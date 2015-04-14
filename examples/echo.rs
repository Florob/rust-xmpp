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
        let response = match stream.handle() {
            xmpp::Event::StreamClosed => break,
            xmpp::Event::Message(msg) => {
                let mut response = msg.clone();
                if let Some(to) = response.remove_attribute("from", None) {
                    response.set_attribute("to".into(), None, to);
                }
                if let Some(from) = response.remove_attribute("to", None) {
                    response.set_attribute("from".into(), None, from);
                }
                response
            }
            _ => continue
        };
        stream.send(response);
    }
}
