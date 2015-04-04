// rust-xmpp
// Copyright (c) 2014 Florian Zeitz
//
// This project is MIT licensed.
// Please see the COPYING file for more information.

use super::Authenticator;

pub struct PlainAuth {
    authcid: String,
    authzid: Option<String>,
    passwd: String
}

impl PlainAuth {
    pub fn new(authcid: String, passwd: String, authzid: Option<String>) -> PlainAuth {
        PlainAuth {
            authcid: authcid,
            passwd: passwd,
            authzid: authzid
        }
    }
}

impl Authenticator for PlainAuth {
    fn initial(&mut self) -> Vec<u8> {
        let mut data: Vec<u8> = Vec::new();
        if let Some(ref authzid) = self.authzid {
            data.extend(authzid.bytes());
        }
        data.push(0);
        data.extend(self.authcid.bytes());
        data.push(0);
        data.extend(self.passwd.bytes());
        data
    }
}
