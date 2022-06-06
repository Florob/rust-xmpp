// rust-xmpp
// Copyright (c) 2014 Florian Zeitz
//
// This project is MIT licensed.
// Please see the COPYING file for more information.

pub use self::anon::AnonAuth;
pub use self::plain::PlainAuth;
pub use self::scram::ScramAuth;

pub mod anon;
pub mod plain;
pub mod scram;

pub trait Authenticator {
    fn initial(&mut self) -> Result<Vec<u8>, &'static str>;
    fn continuation(&mut self, _data: &[u8]) -> Result<Vec<u8>, &'static str> {
        Ok(Vec::new())
    }
}
