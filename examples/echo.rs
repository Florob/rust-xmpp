extern crate xmpp;
use xmpp::XmppStream;
use xmpp::stanzas::Stanza;

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
        let response = match stream.handle() {
            xmpp::Event::StreamClosed => break,
            xmpp::Event::Message(msg) => {
                let mut response = msg.clone();
                let to = response.from().map(|x| x.into());
                response.set_to(to);
                response.set_from(None);
                response
            }
            _ => continue
        };
        stream.send(response);
    }
}
