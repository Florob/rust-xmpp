pub use self::plain::PlainAuth;

pub mod plain;

pub trait Authenticator {
    fn new(authcid: &str, passwd: &str, authzid: Option<&str>) -> Self;

    fn initial(&self) -> Vec<u8>;
    fn continuation(&mut self, _data: &[u8])  -> Vec<u8> {
        Vec::new()
    }
}
