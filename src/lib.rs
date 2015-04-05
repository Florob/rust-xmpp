// rust-xmpp
// Copyright (c) 2014-2015 Florian Zeitz
//
// This project is MIT licensed.
// Please see the COPYING file for more information.

#![crate_name = "xmpp"]
#![crate_type = "lib"]

// These are unsable for now
#![feature(unicode)]

extern crate unicode;
extern crate rustc_serialize;
extern crate openssl;
extern crate xml;

use std::io;
use std::io::{Write, BufStream};
use std::net::TcpStream;
use rustc_serialize::base64;
use rustc_serialize::base64::{FromBase64, ToBase64};

use auth::Authenticator;
use auth::{PlainAuth, ScramAuth};
use non_stanzas::{AuthStart, AuthResponse, StreamStart, StreamEnd, StartTls};
use read_str::ReadString;
use xmpp_send::XmppSend;
use xmpp_socket::XmppSocket;

mod auth;
mod non_stanzas;
mod read_str;
mod xmpp_send;
mod xmpp_socket;
pub mod ns;
pub mod stanzas;

struct XmppHandler {
    username: String,
    password: String,
    domain: String,
    socket: XmppSocket,
    authenticator: Option<Box<Authenticator + 'static>>
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
                socket: XmppSocket::NoSock,
                authenticator: None
            }
        }
    }

    pub fn connect(&mut self) -> io::Result<()> {
        let stream = {
            let address = &self.handler.domain[..];
            try!(TcpStream::connect(&(address, 5222)))
        };

        self.handler.socket = XmppSocket::Tcp(BufStream::new(stream));
        self.handler.start_stream()
    }

    pub fn handle(&mut self) -> io::Result<()> {
        loop {
            let string = {
                let socket = &mut self.handler.socket;
                try!(socket.read_str())
            };
            let builder = &mut self.builder;
            let handler = &mut self.handler;
            self.parser.feed_str(&string);
            for event in &mut self.parser {
                match event {
                    Ok(xml::Event::ElementStart(xml::StartTag {
                        ref name,
                        ns: Some(ref ns),
                        ref prefix, ..
                    })) if *name == "stream" && *ns == ns::STREAMS => {
                        println!("In: Stream start");
                        match *prefix {
                            Some(ref prefix) => {
                                *builder = xml::ElementBuilder::new();
                                builder.set_default_ns(ns::JABBER_CLIENT.to_string());
                                builder.define_prefix(prefix.clone(), ns::STREAMS.to_string());
                            }
                            None => {
                                *builder = xml::ElementBuilder::new();
                                builder.set_default_ns(ns::STREAMS.to_string());
                            }
                        }
                    }
                    Ok(xml::Event::ElementEnd(xml::EndTag {
                        ref name,
                        ns: Some(ref ns), ..
                    })) if *name == "stream" && *ns == ns::STREAMS => {
                        println!("In: Stream end");
                        try!(handler.close_stream());
                        return Ok(())
                    }
                    event => {
                        match builder.handle_event(event) {
                            Some(Ok(e)) => { try!(handler.handle_stanza(e)); }
                            Some(Err(e)) => println!("{}", e),
                            None => (),
                        }
                    }
                }
            }
        }
    }
}

impl XmppHandler {
    fn start_stream(&mut self) -> io::Result<()> {
        let stream_start = StreamStart { to: &self.domain };
        println!("Out: {}", stream_start);
        try!(stream_start.xmpp_send(&mut self.socket));
        self.socket.flush()
    }

    fn close_stream(&mut self) -> io::Result<()> {
        self.send(StreamEnd)
    }

    fn send<T: XmppSend>(&mut self, data: T) -> io::Result<()> {
        println!("Out: {}", data);
        try!(data.xmpp_send(&mut self.socket));
        self.socket.flush()
    }

    fn handle_stanza(&mut self, stanza: xml::Element) -> io::Result<()> {
        println!("In: {}", stanza);
        if stanza.ns.as_ref().map(|x| &x[..]) == Some(ns::STREAMS) && stanza.name == "features" {
            return self.handle_features(stanza);
        }
        if stanza.ns.as_ref().map(|x| &x[..]) == Some(ns::FEATURE_TLS) {
            return self.handle_starttls(stanza);
        }
        if stanza.ns.as_ref().map(|x| &x[..]) == Some(ns::FEATURE_SASL) {
            return self.handle_sasl(stanza);
        }
        Ok(())
    }

    fn handle_features(&mut self, features: xml::Element) -> io::Result<()> {
        // StartTLS
        if features.get_child("starttls", Some(ns::FEATURE_TLS)).is_some() {
            return self.send(StartTls);
        }

        // Auth mechanisms
        if let Some(mechs) = features.get_child("mechanisms", Some(ns::FEATURE_SASL)) {
            return self.handle_mechs(mechs);
        }

        // Bind
        if features.get_child("bind", Some(ns::FEATURE_BIND)).is_some() {
            return self.handle_bind();
        }

        Ok(())
    }

    fn handle_starttls(&mut self, starttls: xml::Element) -> io::Result<()> {
        if starttls.name == "proceed" {
            try!(self.socket.starttls());
            return self.start_stream();
        }
        Ok(())
    }

    fn handle_mechs(&mut self, mechs: &xml::Element) -> io::Result<()> {
        let mechs = mechs.get_children("mechanism", Some(ns::FEATURE_SASL));

        for mech in mechs {
            let mech = mech.content_str();
            let auth: Box<Authenticator> = match &mech[..] {
                "SCRAM-SHA-1" => Box::new(ScramAuth::new(self.username.clone(),
                                                         self.password.clone(), None)),
                "PLAIN" => Box::new(PlainAuth::new(self.username.clone(),
                                                   self.password.clone(), None)),
                _ => continue
            };
            self.authenticator = Some(auth);

            let result = {
                let auth = self.authenticator.as_mut().unwrap();
                auth.initial().to_base64(base64::STANDARD)
            };

            return self.send(AuthStart { mech: &mech, data: &result });
        }

        Ok(())
    }

    fn handle_sasl(&mut self, sasl: xml::Element) -> io::Result<()> {
        if sasl.name == "challenge" {
            let challenge = match sasl.content_str().from_base64() {
                Ok(c) => c,
                Err(_) => return Ok(())
            };

            let result = {
                let auth = self.authenticator.as_mut().unwrap();
                match auth.continuation(&challenge) {
                    Ok(r) => r,
                    Err(e) => {
                        println!("{}", e);
                        return Ok(());
                    }
                }
            };

            let data = result.to_base64(base64::STANDARD);
            return self.send(AuthResponse { data: &data });
        }

        if sasl.name == "success" {
            let success = match sasl.content_str().from_base64() {
                Ok(c) => c,
                Err(_) => return Ok(())
            };
            {
                let auth = self.authenticator.as_mut().unwrap();
                match auth.continuation(&success) {
                    Ok(_) => (),
                    Err(e) => {
                        println!("{}", e);
                        return Ok(());
                    }
                }
            }
            return self.start_stream();
        }

        Ok(())
    }

    fn handle_bind(&mut self) -> io::Result<()> {
        let mut bind_iq = xml::Element::new("iq".to_string(), Some(ns::JABBER_CLIENT.to_string()),
                                            vec![("type".to_string(), None, "set".to_string()),
                                                 ("id".to_string(), None, "bind".to_string())]);
        bind_iq.tag(xml::Element::new("bind".to_string(),
                                      Some(ns::FEATURE_BIND.to_string()), vec![]));
        self.send(bind_iq)
    }
}
