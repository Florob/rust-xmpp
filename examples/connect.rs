extern crate xmpp;
use xmpp::XmppStream;

fn main() {
    let mut stream = XmppStream::new("alice", "localhost", "test");
    match stream.connect() {
        Ok(_) => (),
        Err(e) => {
            println!("{}", e);
            return;
        }
    }
    loop {
        if let xmpp::Event::StreamClosed = stream.handle() {
            break;
        }
    }
}
