pub use self::plain::PlainAuth;
pub use self::scram::ScramAuth;

pub mod plain;
pub mod scram;

pub trait Authenticator {
    fn new(authcid: &str, passwd: &str, authzid: Option<&str>) -> Self;

    fn initial(&mut self) -> Vec<u8>;
    fn continuation(&mut self, _data: &[u8]) -> Result<Vec<u8>, &'static str> {
        Ok(Vec::new())
    }
}
