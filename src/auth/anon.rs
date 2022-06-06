// rust-xmpp
// Copyright (c) 2014 Florian Zeitz
// Copyright (c) 2016 Astro
//
// This project is MIT licensed.
// Please see the COPYING file for more information.

use super::Authenticator;

pub struct AnonAuth;

impl AnonAuth {
    pub fn new() -> AnonAuth {
        AnonAuth
    }
}

impl Authenticator for AnonAuth {
    fn initial(&mut self) -> Result<Vec<u8>, &'static str> {
        Ok(vec![])
    }
}
