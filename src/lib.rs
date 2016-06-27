// rust-xmpp
// Copyright (c) 2014-2015 Florian Zeitz
//
// This project is MIT licensed.
// Please see the COPYING file for more information.

#![crate_name = "xmpp"]
#![crate_type = "lib"]

extern crate rustc_serialize;
extern crate openssl;
extern crate xml;

use std::io;
use std::io::{Write, BufReader};
use std::net::TcpStream;
use std::ops::Deref;
use rustc_serialize::base64;
use rustc_serialize::base64::{FromBase64, ToBase64};

use auth::Authenticator;
use auth::{PlainAuth, ScramAuth};
use non_stanzas::{AuthStart, AuthResponse, DefinedCondition, StreamStart, StreamEnd};
use non_stanzas::{StreamError, StartTls};
use read_str::ReadString;
use stanzas::{AStanza, Stanza, IqType};
use xmpp_send::XmppSend;
use xmpp_socket::XmppSocket;

mod auth;
mod non_stanzas;
mod read_str;
mod xmpp_send;
mod xmpp_socket;
pub mod ns;
pub mod stanzas;

pub struct IqGuard<'a> {
    iq: stanzas::Iq,
    responded: bool,
    handler: &'a mut XmppHandler
}

impl<'a> Deref for IqGuard<'a> {
    type Target = stanzas::Iq;
    fn deref(&self) -> &stanzas::Iq { &self.iq }
}

impl<'a> Drop for IqGuard<'a> {
    fn drop(&mut self) {
        if self.responded { return }

        // Don't respond to IQs without an id attribute
        if let None = self.iq.id() { return; };

        let response = self.iq.error_reply(stanzas::ErrorType::Cancel,
                                           stanzas::DefinedCondition::ServiceUnavailable, None);
        let _ = self.handler.send(response);
    }
}

impl<'a> IqGuard<'a> {
    pub fn respond(&mut self, response: &stanzas::Iq) {
        // TODO: Check attributes of provided response
        self.responded = true;
        let _ = self.handler.send(response);
    }
}

pub enum Event<'a> {
    IqRequest(IqGuard<'a>),
    IqResponse(stanzas::Iq),
    Message(stanzas::Message),
    Presence(stanzas::Presence),
    Bound,
    BindError(stanzas::Iq),
    StreamError(xml::Element),
    StreamClosed
}

struct XmppHandler {
    username: String,
    password: String,
    domain: String,
    closed: bool,
    socket: XmppSocket,
    authenticator: Option<Box<Authenticator + 'static>>,
    pending_bind_id: Option<String>
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
                closed: false,
                socket: XmppSocket::NoSock,
                authenticator: None,
                pending_bind_id: None
            }
        }
    }

    pub fn connect(&mut self) -> io::Result<()> {
        let stream = {
            let address = &self.handler.domain[..];
            try!(TcpStream::connect(&(address, 5222)))
        };
        let stream_read = try!(stream.try_clone());

        self.handler.socket = XmppSocket::Tcp(BufReader::new(stream_read), stream);
        self.handler.start_stream()
    }

    pub fn send<T: XmppSend>(&mut self, data: T) -> io::Result<()> {
        self.handler.send(data)
    }

    pub fn handle(&mut self) -> Event {
        let builder = &mut self.builder;
        let handler = &mut self.handler;
        loop {
            let string =  match handler.socket.read_str() {
                Ok(s) => s,
                Err(_) => return Event::StreamClosed
            };
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
                        let _ = handler.close_stream();
                        return Event::StreamClosed;
                    }
                    event => match builder.handle_event(event) {
                        None => (),
                        Some(Ok(e)) => {
                            println!("In: {}", e);
                            let stanza = match stanzas::AStanza::from_element(e) {
                                Ok(s) => s,
                                Err(e) => {
                                    // For IO errors we should return StreamClosed
                                    // in the next iteration
                                    let _ = handler.handle_non_stanza(e);
                                    continue;
                                }
                            };
                            match stanza {
                                AStanza::MessageStanza(msg) => return Event::Message(msg),
                                AStanza::PresenceStanza(pres) => return Event::Presence(pres),
                                AStanza::IqStanza(iq) => {
                                    match iq.stanza_type() {
                                        None => continue,
                                        Some(IqType::Result)
                                            if handler.pending_bind_id.as_ref()
                                            .map(|x| &x[..]) == iq.id() => {
                                                handler.pending_bind_id = None;
                                                return Event::Bound
                                            },
                                        Some(IqType::Error)
                                            if handler.pending_bind_id.as_ref()
                                            .map(|x| &x[..]) == iq.id() => {
                                                handler.pending_bind_id = None;
                                                return Event::BindError(iq)
                                            },
                                        Some(IqType::Result)
                                        | Some(IqType::Error) => return Event::IqResponse(iq),
                                        Some(IqType::Set)
                                        | Some(IqType::Get) => return Event::IqRequest(IqGuard {
                                            iq: iq,
                                            responded: false,
                                            handler: handler
                                        })
                                    }
                                }
                            }
                        }
                        Some(Err(e)) => {
                            println!("{}", e);
                            let _ = handler.send(StreamError {
                                cond: DefinedCondition::InvalidXml,
                                text: None
                            });
                            let _ = handler.close_stream();
                            // Wait for remote to close stream
                            // TODO: Avoid waiting forever
                            continue;
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
        if !self.closed {
            self.closed = true;
            self.send(StreamEnd)
        } else {
            Ok(())
        }
    }

    fn send<T: XmppSend>(&mut self, data: T) -> io::Result<()> {
        println!("Out: {}", data);
        try!(data.xmpp_send(&mut self.socket));
        self.socket.flush()
    }

    fn handle_non_stanza(&mut self, stanza: xml::Element) -> io::Result<()> {
        match stanza.ns.as_ref().map(|x| &x[..]) {
            Some(ns::STREAMS) if stanza.name == "features" => self.handle_features(stanza),
            Some(ns::FEATURE_TLS) => self.handle_starttls(stanza),
            Some(ns::FEATURE_SASL) => self.handle_sasl(stanza),
            _ => Ok(())
        }
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
            let mut auth: Box<Authenticator> = match &mech[..] {
                "SCRAM-SHA-1" => Box::new(ScramAuth::new(self.username.clone(),
                                                         self.password.clone(), None)),
                "PLAIN" => Box::new(PlainAuth::new(self.username.clone(),
                                                   self.password.clone(), None)),
                _ => continue
            };
            let initial = auth.initial().to_base64(base64::STANDARD);
            self.authenticator = Some(auth);

            return self.send(AuthStart { mech: &mech, data: &initial });
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
        let id: String = "bind".into();

        let mut bind_iq = stanzas::Iq::new(stanzas::IqType::Set, id.clone());
        bind_iq.tag(xml::Element::new("bind".into(), Some(ns::FEATURE_BIND.into()), vec![]));
        self.pending_bind_id = Some(id);
        self.send(bind_iq)
    }
}
