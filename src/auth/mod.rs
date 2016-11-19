// rust-xmpp
// Copyright (c) 2014 Florian Zeitz
//
// This project is MIT licensed.
// Please see the COPYING file for more information.

pub use self::plain::PlainAuth;
pub use self::scram::ScramAuth;
pub use self::anon::AnonAuth;

pub mod plain;
pub mod scram;
pub mod anon;

pub trait Authenticator {
    fn initial(&mut self) -> Vec<u8>;
    fn continuation(&mut self, _data: &[u8]) -> Result<Vec<u8>, &'static str> {
        Ok(Vec::new())
    }
}
