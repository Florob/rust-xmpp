extern crate xmpp;
use xmpp::XmppStream;
use xmpp::stanzas::{Stanza, Presence, PresenceType};

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
        let (opt_response, send_presence) = match stream.handle() {
            xmpp::Event::StreamClosed => break,
            xmpp::Event::Message(msg) => {
                let mut response = msg.clone();
                let to = response.from().map(|x| x.into());
                response.set_to(to);
                response.set_from(None);
                (Some(response), false)
            }
            xmpp::Event::Bound(_jid) => {
                (None, true)
            }
            _ => continue
        };
        match opt_response {
            Some(response) => stream.send(response).unwrap(),
            None => ()
        }
        if send_presence {
            stream.send(Presence::new(PresenceType::Available, "".to_owned())).unwrap();
        }
    }
}
