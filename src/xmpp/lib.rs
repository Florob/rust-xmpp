#![crate_name = "xmpp"]
#![crate_type = "lib"]

#![feature(macro_rules)]

extern crate serialize;

extern crate xml;
extern crate openssl;

use std::mem;
use std::io::net::tcp::TcpStream;
use std::io::BufferedStream;
use std::io::IoResult;
use serialize::base64;
use serialize::base64::{FromBase64, ToBase64};
use openssl::ssl::{SslContext, SslStream, Sslv23};

use read_str::ReadString;
use xmpp_send::XmppSend;
use auth::Authenticator;
use auth::{PlainAuth, ScramAuth};

mod read_str;
mod xmpp_send;
mod auth;
pub mod ns;

enum XmppSocket {
    Tcp(BufferedStream<TcpStream>),
    Tls(BufferedStream<SslStream<TcpStream>>),
    NoSock
}

impl Writer for XmppSocket {
    fn write(&mut self, buf: &[u8]) -> IoResult<()> {
        match *self {
            Tcp(ref mut stream) => stream.write(buf),
            Tls(ref mut stream) => stream.write(buf),
            NoSock => fail!("No socket yet")
        }
    }

    fn flush(&mut self) -> IoResult<()> {
        match *self {
            Tcp(ref mut stream) => stream.flush(),
            Tls(ref mut stream) => stream.flush(),
            NoSock => fail!("No socket yet")
        }
    }
}

impl ReadString for XmppSocket {
    fn read_str(&mut self) -> IoResult<String> {
        match *self {
            Tcp(ref mut stream) => stream.read_str(),
            Tls(ref mut stream) => stream.read_str(),
            NoSock => fail!("Tried to read string before socket exists")
        }
    }
}

struct XmppHandler {
    username: String,
    password: String,
    domain: String,
    socket: XmppSocket,
    authenticator: Option<Box<Authenticator>>
}

pub struct XmppStream {
    parser: xml::Parser,
    builder: xml::ElementBuilder,
    handler: XmppHandler
}

impl XmppStream {
    pub fn new(user: &str, domain: &str, password: &str) -> XmppStream {
        XmppStream {
            parser: xml::Parser::new(),
            builder: xml::ElementBuilder::new(),
            handler: XmppHandler {
                username: user.to_string(),
                password: password.to_string(),
                domain: domain.to_string(),
                socket: NoSock,
                authenticator: None
            }
        }
    }

    pub fn connect(&mut self) -> IoResult<()> {
        let stream = {
            let address = self.handler.domain.as_slice();
            try!(TcpStream::connect(address, 5222))
        };

        self.handler.socket = Tcp(BufferedStream::new(stream));
        self.handler.start_stream()
    }

    pub fn handle(&mut self) -> IoResult<()> {
        let mut closed = false;
        while !closed {
            let string = {
                let socket = &mut self.handler.socket;
                try!(socket.read_str())
            };
            let builder = &mut self.builder;
            let handler = &mut self.handler;
            self.parser.feed_str(string.as_slice());
            for event in self.parser {
                match event {
                    Ok(xml::StartTag(xml::StartTag {
                        name: ref name,
                        ns: Some(ref ns),
                        prefix: ref prefix, ..
                    })) if name.as_slice() == "stream" && ns.as_slice() == ns::STREAMS => {
                        println!("In: Stream start");
                        match *prefix {
                            Some(ref prefix) => {
                                *builder = xml::ElementBuilder::new();
                                builder.set_default_ns(ns::JABBER_CLIENT);
                                builder.define_prefix(prefix.as_slice(), ns::STREAMS);
                            }
                            None => ()
                        }
                    }
                    Ok(xml::EndTag(xml::EndTag {
                        name: ref name,
                        ns: Some(ref ns), ..
                    })) if name.as_slice() == "stream" && ns.as_slice() == ns::STREAMS => {
                        println!("In: Stream end");
                        try!(handler.close_stream());
                        closed = true;
                    }
                    Ok(event) => {
                        match builder.push_event(event) {
                            Ok(Some(ref e)) => { try!(handler.handle_stanza(e)); }
                            Ok(None) => (),
                            Err(e) => println!("{}", e),
                        }
                    }
                    Err(e) => println!("Line: {} Column: {} Msg: {}", e.line, e.col, e.msg),
                }
            }
        }
        Ok(())
    }
}

impl XmppHandler {
    fn start_stream(&mut self) -> IoResult<()> {
        let start = format!("<?xml version='1.0'?>\n\
                             <stream:stream xmlns:stream='{}' xmlns='{}' version='1.0' to='{}'>",
                             ns::STREAMS, ns::JABBER_CLIENT, self.domain);
        self.send(start)
    }

    fn close_stream(&mut self) -> IoResult<()> {
        self.send("</stream:stream>")
    }

    fn send<T: XmppSend>(&mut self, data: T) -> IoResult<()> {
        let data = data.xmpp_str();
        println!("Out: {}", data);
        try!(self.socket.write(data.as_slice().as_bytes()));
        self.socket.flush()
    }

    fn handle_stanza(&mut self, stanza: &xml::Element) -> IoResult<()> {
        println!("In: {}", *stanza)
        match stanza {
            &xml::Element {
                name: ref name,
                ns: Some(ref ns), ..
            } if name.as_slice() == "features" && ns.as_slice() == ns::STREAMS => {
                // StartTLS
                let starttls = stanza.get_child("starttls", Some(ns::FEATURE_TLS));
                if starttls.is_some() {
                    return self.send(format!("<starttls xmlns='{}'/>", ns::FEATURE_TLS));
                }

                // Auth mechanisms
                let mechs = stanza.get_child("mechanisms", Some(ns::FEATURE_SASL));
                if mechs.is_some() {
                    return self.handle_mechs(mechs.unwrap());
                }

                // Bind
                let bind = stanza.get_child("bind", Some(ns::FEATURE_BIND));
                if bind.is_some() {
                    return self.handle_bind();
                }
            }

            &xml::Element {
                name: ref name,
                ns: Some(ref ns), ..
            } if name.as_slice() == "proceed" && ns.as_slice() == ns::FEATURE_TLS => {
                let socket = mem::replace(&mut self.socket, NoSock);
                match socket {
                    Tcp(sock) => {
                        let ctx = SslContext::new(Sslv23);
                        let ssl = SslStream::new(&ctx, sock.unwrap());
                        self.socket = Tls(BufferedStream::new(ssl));
                        return self.start_stream();
                    }
                    _ => fail!("No socket, or TLS already negotiated")
                }
            }

            &xml::Element {
                name: ref name,
                ns: Some(ref ns), ..
            } if name.as_slice() == "challenge" && ns.as_slice() == ns::FEATURE_SASL => {
                let challenge = match stanza.content_str().as_slice().from_base64() {
                    Ok(c) => c,
                    Err(_) => return Ok(())
                };

                let result = {
                    let auth = self.authenticator.get_mut_ref();
                    match auth.continuation(challenge.as_slice()) {
                        Ok(r) => r,
                        Err(e) => {
                            println!("{}", e);
                            return Ok(());
                        }
                    }
                };

                let data = result.as_slice().to_base64(base64::STANDARD);
                return self.send(format!("<response xmlns='{}'>{}</response>",
                                          ns::FEATURE_SASL, data));
            }

            &xml::Element {
                name: ref name,
                ns: Some(ref ns), ..
            } if name.as_slice() == "success" && ns.as_slice() == ns::FEATURE_SASL => {
                let success = match stanza.content_str().as_slice().from_base64() {
                    Ok(c) => c,
                    Err(_) => return Ok(())
                };
                {
                    let auth = self.authenticator.get_mut_ref();
                    match auth.continuation(success.as_slice()) {
                        Ok(_) => (),
                        Err(e) => {
                            println!("{}", e);
                            return Ok(());
                        }
                    }
                }
                return self.start_stream();
            }
            _ => ()
        }
        Ok(())
    }

    fn handle_mechs(&mut self, mechs: &xml::Element) -> IoResult<()> {
        let mechs = mechs.get_children("mechanism", Some(ns::FEATURE_SASL));

        for mech in mechs.iter() {
            let mech = mech.content_str();
            match mech.as_slice() {
                "SCRAM-SHA-1" => {
                    let auth: ScramAuth = Authenticator::new(self.username.as_slice(),
                                                             self.password.as_slice(), None);
                    self.authenticator = Some(box auth as Box<Authenticator>);
                }
                "PLAIN" => {
                    let auth: PlainAuth = Authenticator::new(self.username.as_slice(),
                                                             self.password.as_slice(), None);
                    self.authenticator = Some(box auth as Box<Authenticator>);
                }
                _ => continue
            }

            let result = {
                let auth = self.authenticator.get_mut_ref();
                auth.initial().as_slice().to_base64(base64::STANDARD)
            };

            return self.send(format!("<auth mechanism='{}' xmlns='{}'>{}</auth>",
                                     mech, ns::FEATURE_SASL, result));
        }

        Ok(())
    }

    fn handle_bind(&mut self) -> IoResult<()> {
        self.send(format!("<iq type='set' id='bind'>\
                               <bind xmlns='{}'/>\
                           </iq>", ns::FEATURE_BIND))
    }
}
