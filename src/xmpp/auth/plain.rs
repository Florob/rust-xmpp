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
    pub fn new(authcid: &str, passwd: &str, authzid: Option<&str>) -> PlainAuth {
        PlainAuth {
            authcid: authcid.to_string(),
            passwd: passwd.to_string(),
            authzid: authzid.map(|x| x.to_string())
        }
    }
}

impl Authenticator for PlainAuth {
    fn initial(&mut self) -> Vec<u8> {
        let mut data: Vec<u8> = Vec::new();
        for authzid in self.authzid.iter() {
            data.push_all(authzid.as_bytes());
        }
        data.push(0);
        data.push_all(self.authcid.as_bytes());
        data.push(0);
        data.push_all(self.passwd.as_bytes());
        data
    }
}
