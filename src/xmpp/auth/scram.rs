// rust-xmpp
// Copyright (c) 2014 Florian Zeitz
//
// This project is MIT licensed.
// Please see the COPYING file for more information.

use std::str;

use super::Authenticator;
use openssl::crypto::rand::rand_bytes;
use openssl::crypto::hmac::HMAC;
use openssl::crypto::hash::Hasher;
use openssl::crypto::hash::HashType::SHA1;
use openssl::crypto::pkcs5::pbkdf2_hmac_sha1;
use serialize::base64;
use serialize::base64::{FromBase64, ToBase64};

pub use self::State::{ Initial,WaitFirst,WaitFinal,Finished};

macro_rules! check(
    ($e:expr, $s:expr) => (match $e { Some(s) => s, None => return Err($s) })
)

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
        // Restrict output to printable ASCII, excludint '~'
        *c = ( *c % (('~' as u8) - ('!' as u8)) ) + ('!' as u8);
        // Map occurences of ',' to '~'
        if *c == (',' as u8) { *c = '~' as u8 }
    }
    nonce
}

fn hmac_sha1(key: &[u8], data: &[u8]) -> Vec<u8> {
    let mut hmac = HMAC(SHA1, key);
    hmac.update(data);
    hmac.finalize()
}

fn sha1(data: &[u8]) -> Vec<u8> {
    let mut sha1 = Hasher::new(SHA1);
    sha1.update(data);
    sha1.finalize()
}

fn parse_server_first(data: &str) -> Result<(String, Vec<u8>, uint), &'static str> {
    let mut nonce = None;
    let mut salt = None;
    let mut iter: Option<uint> = None;
    for  sub in data.split(',') {
        if sub.starts_with("r=") {
            nonce = Some(sub.slice_from(2).to_string());
        } else if sub.starts_with("s=") {
            let b64salt = sub.slice_from(2);
            salt = match b64salt.from_base64().ok() {
                None => return Err("SCRAM: Invalid base64 encoding for salt"),
                s => s
            };
        } else if sub.starts_with("i=") {
            iter = match from_str(sub.slice_from(2)) {
                None => break,
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
    fn handle_server_first(&mut self, data: &[u8]) -> Result<Vec<u8>, &'static str> {
        let data = check!(str::from_utf8(data), "SCRAM: Server sent non-UTF-8 data");
        let (nonce, salt, iter) = try!(parse_server_first(data));

        {
            let cnonce = match self.state {
                WaitFirst(ref c, _) => c,
                _ => unreachable!()
            };

            if !nonce.as_slice().starts_with(cnonce.as_slice()) {
                return Err("SCRAM: Server replied with invalid nonce")
            }
        }

        let gs2header = match self.authzid {
            Some(ref a) => format!("n,a={},", a),
            None => "n,,".to_string()
        }.as_bytes().to_base64(base64::STANDARD);

        let mut result = Vec::new();
        // Add c=<base64(GS2Header+channelBindingData)>
        result.push_all(b"c=");
        result.push_all(gs2header.as_bytes());
        // Add r=<nonce>
        result.push_all(b",r=");
        result.push_all(nonce.as_bytes());

        // SaltedPassword := Hi(Normalize(password), salt, i)
        let salted_passwd = pbkdf2_hmac_sha1(self.passwd.as_slice(), salt.as_slice(), iter, 20);

        /*
         * AuthMessage := client-first-message-bare + "," +
         *		  server-first-message + "," +
         *		  client-final-message-without-proof
         */
        let mut auth_message = Vec::new();
        {
            let client_first_message_bare = match self.state {
                WaitFirst(_, ref c) => c,
                _ => unreachable!()
            };
            auth_message.push_all(client_first_message_bare.as_bytes());
        }
        auth_message.push(',' as u8);
        auth_message.push_all(data.as_bytes());
        auth_message.push(',' as u8);
        auth_message.push_all(result.as_slice());

        // ClientKey := HMAC(SaltedPassword, "Client Key")
        let client_key = hmac_sha1(salted_passwd.as_slice(), b"Client Key");

        // StoredKey := H(ClientKey)
        let stored_key = sha1(client_key.as_slice());

        // ClientSignature := HMAC(StoredKey, AuthMessage)
        let client_signature = hmac_sha1(stored_key.as_slice(), auth_message.as_slice());
        // ServerKey := HMAC(SaltedPassword, "Server Key")
        let server_key = hmac_sha1(salted_passwd.as_slice(), b"Server Key");
        // ServerSignature := HMAC(ServerKey, AuthMessage)
        let server_signature = hmac_sha1(server_key.as_slice(), auth_message.as_slice());
        // ClientProof := ClientKey XOR ClientSignature
        let client_proof: Vec<u8> = client_key.iter().zip(client_signature.iter()).map(|(x, y)| {
            *x ^ *y
        }).collect();

        // Add p=<base64(ClientProof)>
        result.push_all(b",p=");
        result.push_all(client_proof.as_slice().to_base64(base64::STANDARD).as_bytes());

        self.state = WaitFinal(server_signature);

        Ok(result)
    }

    fn handle_server_final(&mut self, data: &[u8]) -> Result<Vec<u8>, &'static str> {
        let data = check!(str::from_utf8(data), "SCRAM: Server sent non-UTF-8 data");
        if !data.starts_with("v=") { return Err("SCRAM: Server didn't sent a verifier") }

        let verifier = check!(data.slice_from(2).from_base64().ok(),
                              "SCRAM: Server sent verifier with invalid base64 encoding");

        {
            let server_signature = match self.state {
                WaitFinal(ref s) => s,
                _ => unreachable!()
            };
            if *server_signature != verifier { return Err("SCRAM: Server sent invalid verifier"); }
        }

        self.state = Finished;

        Ok(Vec::new())
    }
}

impl Authenticator for ScramAuth {
    fn new(authcid: &str, passwd: &str, authzid: Option<&str>) -> ScramAuth {
        ScramAuth {
            authcid: authcid.to_string(),
            passwd: passwd.to_string(),
            authzid: authzid.map(|x| x.to_string()),
            state: Initial
        }
    }

    fn initial(&mut self) -> Vec<u8> {
        let gs2header = match self.authzid {
            Some(ref a) => format!("n,a={},", a),
            None => "n,,".to_string()
        };

        let cnonce = String::from_utf8(gen_nonce()).ok().expect("Generated an invalid nonce");

        let client_first_message_bare = format!("n={},r={}", self.authcid, cnonce);

        let mut ret = Vec::new();
        ret.push_all(gs2header.as_bytes());
        ret.push_all(client_first_message_bare.as_bytes());

        self.state = WaitFirst(cnonce, client_first_message_bare);

        ret
    }

    fn continuation(&mut self, data: &[u8]) -> Result<Vec<u8>, &'static str> {
        match self.state {
            Initial => {
                Ok(self.initial())
            }
            WaitFirst(..) => {
                self.handle_server_first(data)
            }
            WaitFinal(_) => {
                self.handle_server_final(data)
            }
            Finished => {
                Ok(Vec::new())
            }
        }
    }
}
