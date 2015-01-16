// rust-xmpp
// Copyright (c) 2014 Florian Zeitz
//
// This project is MIT licensed.
// Please see the COPYING file for more information.

#![crate_name = "xmpp"]
#![crate_type = "lib"]

extern crate unicode;
extern crate "rustc-serialize" as serialize;
extern crate openssl;
extern crate xml;

use std::mem;
use std::io::net::tcp::TcpStream;
use std::io::BufferedStream;
use std::io::{IoResult, IoError, OtherIoError};
use serialize::base64;
use serialize::base64::{FromBase64, ToBase64};
use openssl::ssl::{SslContext, SslStream, SslMethod};

use read_str::ReadString;
use xmpp_send::XmppSend;
use auth::Authenticator;
use auth::{PlainAuth, ScramAuth};

mod read_str;
mod xmpp_send;
mod auth;
pub mod ns;
pub mod stanzas;

enum XmppSocket {
    Tcp(BufferedStream<TcpStream>),
    Tls(BufferedStream<SslStream<TcpStream>>),
    NoSock
}

impl Writer for XmppSocket {
    fn write(&mut self, buf: &[u8]) -> IoResult<()> {
        match *self {
            XmppSocket::Tcp(ref mut stream) => stream.write(buf),
            XmppSocket::Tls(ref mut stream) => stream.write(buf),
            XmppSocket::NoSock => panic!("No socket yet")
        }
    }

    fn flush(&mut self) -> IoResult<()> {
        match *self {
            XmppSocket::Tcp(ref mut stream) => stream.flush(),
            XmppSocket::Tls(ref mut stream) => stream.flush(),
            XmppSocket::NoSock => panic!("No socket yet")
        }
    }
}

impl ReadString for XmppSocket {
    fn read_str(&mut self) -> IoResult<String> {
        match *self {
            XmppSocket::Tcp(ref mut stream) => stream.read_str(),
            XmppSocket::Tls(ref mut stream) => stream.read_str(),
            XmppSocket::NoSock => panic!("Tried to read string before socket exists")
        }
    }
}

struct XmppHandler<'a> {
    username: String,
    password: String,
    domain: String,
    socket: XmppSocket,
    authenticator: Option<Box<Authenticator + 'a>>
}

pub struct XmppStream<'a> {
    parser: xml::Parser,
    builder: xml::ElementBuilder,
    handler: XmppHandler<'a>
}

impl<'a> XmppStream<'a> {
    pub fn new(user: &str, domain: &str, password: &str) -> XmppStream<'a> {
        XmppStream {
            parser: xml::Parser::new(),
            builder: xml::ElementBuilder::new(),
            handler: XmppHandler {
                username: user.to_string(),
                password: password.to_string(),
                domain: domain.to_string(),
                socket: XmppSocket::NoSock,
                authenticator: None
            }
        }
    }

    pub fn connect(&mut self) -> IoResult<()> {
        let stream = {
            let address = &self.handler.domain[];
            try!(TcpStream::connect((address, 5222)))
        };

        self.handler.socket = XmppSocket::Tcp(BufferedStream::new(stream));
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
            self.parser.feed_str(&string[]);
            for event in self.parser {
                match event {
                    Ok(xml::Event::ElementStart(xml::StartTag {
                        ref name,
                        ns: Some(ref ns),
                        ref prefix, ..
                    })) if &name[] == "stream" && &ns[] == ns::STREAMS => {
                        println!("In: Stream start");
                        match *prefix {
                            Some(ref prefix) => {
                                *builder = xml::ElementBuilder::new();
                                builder.set_default_ns(ns::JABBER_CLIENT);
                                builder.define_prefix(&prefix[], ns::STREAMS);
                            }
                            None => {
                                *builder = xml::ElementBuilder::new();
                                builder.set_default_ns(ns::STREAMS);
                            }
                        }
                    }
                    Ok(xml::Event::ElementEnd(xml::EndTag {
                        ref name,
                        ns: Some(ref ns), ..
                    })) if &name[] == "stream" && &ns[] == ns::STREAMS => {
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

impl<'a> XmppHandler<'a> {
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
        try!(self.socket.write(data.as_bytes()));
        self.socket.flush()
    }

    fn handle_stanza(&mut self, stanza: &xml::Element) -> IoResult<()> {
        println!("In: {}", *stanza);
        match stanza {
            &xml::Element {
                ref name,
                ns: Some(ref ns), ..
            } if &name[] == "features" && &ns[] == ns::STREAMS => {
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
                ref name,
                ns: Some(ref ns), ..
            } if &name[] == "proceed" && &ns[] == ns::FEATURE_TLS => {
                let socket = mem::replace(&mut self.socket, XmppSocket::NoSock);
                match socket {
                    XmppSocket::Tcp(sock) => {
                        let ctx = match SslContext::new(SslMethod::Sslv23) {
                            Ok(ctx) => ctx,
                            Err(_) => return Err(IoError {
                                kind: OtherIoError,
                                desc: "Couldn not create SSL context",
                                detail: None
                            })
                        };
                        let ssl = match SslStream::new(&ctx, sock.into_inner()) {
                            Ok(ssl) => ssl,
                            Err(_) => return Err(IoError {
                                kind: OtherIoError,
                                desc: "Couldn not create SSL stream",
                                detail: None
                            })
                        };
                        self.socket = XmppSocket::Tls(BufferedStream::new(ssl));
                        return self.start_stream();
                    }
                    _ => panic!("No socket, or TLS already negotiated")
                }
            }

            &xml::Element {
                ref name,
                ns: Some(ref ns), ..
            } if &name[] == "challenge" && &ns[] == ns::FEATURE_SASL => {
                let challenge = match stanza.content_str()[].from_base64() {
                    Ok(c) => c,
                    Err(_) => return Ok(())
                };

                let result = {
                    let auth = self.authenticator.as_mut().unwrap();
                    match auth.continuation(&challenge[]) {
                        Ok(r) => r,
                        Err(e) => {
                            println!("{}", e);
                            return Ok(());
                        }
                    }
                };

                let data = result[].to_base64(base64::STANDARD);
                return self.send(format!("<response xmlns='{}'>{}</response>",
                                          ns::FEATURE_SASL, data));
            }

            &xml::Element {
                ref name,
                ns: Some(ref ns), ..
            } if &name[] == "success" && &ns[] == ns::FEATURE_SASL => {
                let success = match stanza.content_str().from_base64() {
                    Ok(c) => c,
                    Err(_) => return Ok(())
                };
                {
                    let auth = self.authenticator.as_mut().unwrap();
                    match auth.continuation(&success[]) {
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
            let auth = match &mech[] {
                "SCRAM-SHA-1" => {
                    Box::new(ScramAuth::new(&self.username[],
                                            &self.password[], None)) as Box<Authenticator>
                }
                "PLAIN" => {
                    Box::new(PlainAuth::new(&self.username[],
                                            &self.password[], None)) as Box<Authenticator>
                }
                _ => continue
            };
            self.authenticator = Some(auth);

            let result = {
                let auth = self.authenticator.as_mut().unwrap();
                auth.initial().to_base64(base64::STANDARD)
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
