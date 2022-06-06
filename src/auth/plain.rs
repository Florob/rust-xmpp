// rust-xmpp
// Copyright (c) 2014 Florian Zeitz
//
// This project is MIT licensed.
// Please see the COPYING file for more information.

use super::Authenticator;

pub struct PlainAuth {
    authcid: String,
    authzid: Option<String>,
    passwd: String,
}

impl PlainAuth {
    pub fn new(authcid: String, passwd: String, authzid: Option<String>) -> PlainAuth {
        PlainAuth {
            authcid,
            passwd,
            authzid,
        }
    }
}

impl Authenticator for PlainAuth {
    fn initial(&mut self) -> Result<Vec<u8>, &'static str> {
        let mut data: Vec<u8> = Vec::new();
        if let Some(ref authzid) = self.authzid {
            data.extend(authzid.bytes());
        }
        data.push(0);
        data.extend(self.authcid.bytes());
        data.push(0);
        data.extend(self.passwd.bytes());
        Ok(data)
    }
}
