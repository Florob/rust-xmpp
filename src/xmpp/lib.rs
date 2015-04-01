// rust-xmpp
// Copyright (c) 2014-2015 Florian Zeitz
//
// This project is MIT licensed.
// Please see the COPYING file for more information.

#![crate_name = "xmpp"]
#![crate_type = "lib"]

// These are unsable for now
#![feature(collections)]
#![feature(io)]
#![feature(unicode)]
#![feature(into_cow)]

extern crate unicode;
extern crate rustc_serialize as serialize;
extern crate openssl;
extern crate xml;

use std::io;
use std::io::{Write, BufStream};
use std::net::TcpStream;
use serialize::base64;
use serialize::base64::{FromBase64, ToBase64};

use read_str::ReadString;
use xmpp_send::XmppSend;
use xmpp_socket::XmppSocket;
use auth::Authenticator;
use auth::{PlainAuth, ScramAuth};

mod read_str;
mod xmpp_send;
mod xmpp_socket;
mod auth;
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
        'main: loop {
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
                        break 'main;
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
        Ok(())
    }
}

impl XmppHandler {
    fn start_stream(&mut self) -> io::Result<()> {
        let start = format!("<?xml version='1.0'?>\n\
                             <stream:stream xmlns:stream='{}' xmlns='{}' version='1.0' to='{}'>",
                             ns::STREAMS, ns::JABBER_CLIENT, self.domain);
        self.send(start)
    }

    fn close_stream(&mut self) -> io::Result<()> {
        self.send("</stream:stream>")
    }

    fn send<'a, T: XmppSend<'a>>(&mut self, data: T) -> io::Result<()> {
        let data = data.xmpp_str();
        println!("Out: {}", data);
        try!(self.socket.write_all(data.as_bytes()));
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
        let starttls = features.get_child("starttls", Some(ns::FEATURE_TLS));
        if starttls.is_some() {
            return self.send(format!("<starttls xmlns='{}'/>", ns::FEATURE_TLS));
        }

        // Auth mechanisms
        let mechs = features.get_child("mechanisms", Some(ns::FEATURE_SASL));
        if mechs.is_some() {
            return self.handle_mechs(mechs.unwrap());
        }

        // Bind
        let bind = features.get_child("bind", Some(ns::FEATURE_BIND));
        if bind.is_some() {
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
            let auth = match &mech[..] {
                "SCRAM-SHA-1" => {
                    Box::new(ScramAuth::new(&self.username,
                                            &self.password, None)) as Box<Authenticator>
                }
                "PLAIN" => {
                    Box::new(PlainAuth::new(&self.username,
                                            &self.password, None)) as Box<Authenticator>
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
            return self.send(format!("<response xmlns='{}'>{}</response>",
                                     ns::FEATURE_SASL, data));
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
        self.send(format!("<iq type='set' id='bind'>\
                               <bind xmlns='{}'/>\
                           </iq>", ns::FEATURE_BIND))
    }
}
