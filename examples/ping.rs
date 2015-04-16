extern crate xml;
extern crate xmpp;
use xmpp::XmppStream;
use xmpp::stanzas::{Iq, IqType, Stanza};

const NS_PING: &'static str = "urn:xmpp:ping";

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
        match stream.handle() {
            xmpp::Event::StreamClosed => break,
            xmpp::Event::IqRequest(mut iq) => {
                if let Some(IqType::Get) = iq.stanza_type() {
                    if let Some(_) = iq.get_child("ping", Some(NS_PING)) {
                        let id = if let Some(id) = iq.id() { id.into() } else { continue };
                        let to = iq.from().map(|x| x.into());
                        let mut response = Iq::new(IqType::Result, id);
                        response.set_to(to);
                        response.tag(xml::Element::new("pong".into(), Some(NS_PING.into()),
                                                       vec![]));
                        iq.respond(&response);
                    }
                }
            }
            _ => continue
        }
    }
}
