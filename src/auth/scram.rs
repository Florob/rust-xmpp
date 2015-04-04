// rust-xmpp
// Copyright (c) 2014 Florian Zeitz
//
// This project is MIT licensed.
// Please see the COPYING file for more information.

use std::str;

use super::Authenticator;
use openssl::crypto::rand::rand_bytes;
use openssl::crypto::hash::Type::SHA1;
use openssl::crypto::hash::hash;
use openssl::crypto::hmac::hmac;
use openssl::crypto::pkcs5::pbkdf2_hmac_sha1;
use rustc_serialize::base64;
use rustc_serialize::base64::{FromBase64, ToBase64};

macro_rules! check (
    ($e:expr, $s:expr) => (match $e { Some(s) => s, None => return Err($s) })
);

enum State {
    Initial,
    WaitFirst(String, String),
    WaitFinal(Vec<u8>),
    Finished
}

pub struct ScramAuth {
    authcid: String,
    authzid: Option<String>,
    passwd: String,
    state: State
}

fn gen_nonce() -> Vec<u8> {
    let mut nonce = rand_bytes(64);

    for c in nonce.iter_mut() {
        // Restrict output to printable ASCII, excluding '~'
        *c = ( *c % (('~' as u8) - ('!' as u8)) ) + ('!' as u8);
        // Map occurences of ',' to '~'
        if *c == (',' as u8) { *c = '~' as u8 }
    }
    nonce
}

fn parse_server_first<'a>(data: &'a str) -> Result<(&'a str, Vec<u8>, u16), &'static str> {
    let mut nonce = None;
    let mut salt = None;
    let mut iter: Option<u16> = None;
    for  sub in data.split(',') {
        if sub.starts_with("r=") {
            nonce = Some(&sub[2..]);
        } else if sub.starts_with("s=") {
            salt = match sub[2..].from_base64().ok() {
                None => return Err("SCRAM: Invalid base64 encoding for salt"),
                s => s
            };
        } else if sub.starts_with("i=") {
            iter = match sub[2..].parse().ok() {
                None => return Err("SCRAM: Iteration count is not a number"),
                it => it,
            };
        } else if sub.starts_with("m=") {
            return Err("SCRAM: Unsupported mandatory extension found");
        }
    }

    let nonce = check!(nonce, "SCRAM: No nonce found");
    let salt = check!(salt, "SCRAM: No salt found");
    let iter = check!(iter, "SCRAM: No iteration count found");

    Ok((nonce, salt, iter))
}

impl ScramAuth {
    pub fn new(authcid: String, passwd: String, authzid: Option<String>) -> ScramAuth {
        ScramAuth {
            authcid: authcid,
            passwd: passwd,
            authzid: authzid,
            state: State::Initial
        }
    }

    fn handle_server_first(&mut self, data: &[u8]) -> Result<Vec<u8>, &'static str> {
        let data = check!(str::from_utf8(data).ok(), "SCRAM: Server sent non-UTF-8 data");
        let (nonce, salt, iter) = try!(parse_server_first(data));

        {
            let cnonce = match self.state {
                State::WaitFirst(ref c, _) => c,
                _ => unreachable!()
            };

            if !nonce.starts_with(&cnonce[..]) {
                return Err("SCRAM: Server replied with invalid nonce")
            }
        }

        let gs2header = if let Some(ref authzid) = self.authzid {
            format!("n,a={},", authzid).as_bytes().to_base64(base64::STANDARD)
        } else {
            b"n,,".to_base64(base64::STANDARD)
        };

        let mut result: Vec<u8> = Vec::new();
        // Add c=<base64(GS2Header+channelBindingData)>
        result.extend("c=".bytes());
        result.extend(gs2header.bytes());
        // Add r=<nonce>
        result.extend(",r=".bytes());
        result.extend(nonce.bytes());

        // SaltedPassword := Hi(Normalize(password), salt, i)
        let salted_passwd = pbkdf2_hmac_sha1(&self.passwd, &salt, iter as usize, 20);

        /*
         * AuthMessage := client-first-message-bare + "," +
         *		  server-first-message + "," +
         *		  client-final-message-without-proof
         */
        let mut auth_message = Vec::new();
        {
            let client_first_message_bare = match self.state {
                State::WaitFirst(_, ref c) => c,
                _ => unreachable!()
            };
            auth_message.extend(client_first_message_bare.bytes());
        }
        auth_message.push(b',');
        auth_message.extend(data.bytes());
        auth_message.push(b',');
        auth_message.extend(result.iter().cloned());

        // ClientKey := HMAC(SaltedPassword, "Client Key")
        let client_key = hmac(SHA1, &salted_passwd, b"Client Key");

        // StoredKey := H(ClientKey)
        let stored_key = hash(SHA1, &client_key);

        // ClientSignature := HMAC(StoredKey, AuthMessage)
        let client_signature = hmac(SHA1, &stored_key, &auth_message);
        // ServerKey := HMAC(SaltedPassword, "Server Key")
        let server_key = hmac(SHA1, &salted_passwd, b"Server Key");
        // ServerSignature := HMAC(ServerKey, AuthMessage)
        let server_signature = hmac(SHA1, &server_key, &auth_message);
        // ClientProof := ClientKey XOR ClientSignature
        let client_proof: Vec<u8> = client_key.iter().zip(client_signature.iter()).map(|(x, y)| {
            *x ^ *y
        }).collect();

        // Add p=<base64(ClientProof)>
        result.extend(",p=".bytes());
        result.extend(client_proof.to_base64(base64::STANDARD).bytes());

        self.state = State::WaitFinal(server_signature);

        Ok(result)
    }

    fn handle_server_final(&mut self, data: &[u8]) -> Result<Vec<u8>, &'static str> {
        let data = check!(str::from_utf8(data).ok(), "SCRAM: Server sent non-UTF-8 data");
        if !data.starts_with("v=") { return Err("SCRAM: Server didn't sent a verifier") }

        let verifier = check!(data[2..].from_base64().ok(),
                              "SCRAM: Server sent verifier with invalid base64 encoding");

        {
            let server_signature = match self.state {
                State::WaitFinal(ref s) => s,
                _ => unreachable!()
            };
            if *server_signature != verifier { return Err("SCRAM: Server sent invalid verifier"); }
        }

        self.state = State::Finished;

        Ok(Vec::new())
    }
}

impl Authenticator for ScramAuth {
    fn initial(&mut self) -> Vec<u8> {
        let gs2header = match self.authzid {
            Some(ref a) => format!("n,a={},", a),
            None => "n,,".to_string()
        };

        let cnonce = String::from_utf8(gen_nonce()).ok().expect("Generated an invalid nonce");

        let client_first_message_bare = format!("n={},r={}", self.authcid, cnonce);

        let mut ret = Vec::new();
        ret.extend(gs2header.bytes());
        ret.extend(client_first_message_bare.bytes());

        self.state = State::WaitFirst(cnonce, client_first_message_bare);

        ret
    }

    fn continuation(&mut self, data: &[u8]) -> Result<Vec<u8>, &'static str> {
        match self.state {
            State::Initial => {
                Ok(self.initial())
            }
            State::WaitFirst(..) => {
                self.handle_server_first(data)
            }
            State::WaitFinal(_) => {
                self.handle_server_final(data)
            }
            State::Finished => {
                Ok(Vec::new())
            }
        }
    }
}
