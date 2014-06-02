#![crate_id = "xmpp#0.1"]
#![crate_type = "lib" ]

extern crate serialize;
extern crate xml;
extern crate openssl;

use std::mem;
use std::io::net::tcp::TcpStream;
use std::io::BufferedStream;
use std::io::IoResult;
use serialize::base64;
use serialize::base64::ToBase64;
use openssl::ssl::{SslContext, SslStream, Sslv23};

use read_str::ReadString;
use auth::Authenticator;
use auth::PlainAuth;

mod read_str;
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
                        println!("Got stream start");
                        match *prefix {
                            Some(ref prefix) => {
                                *builder = xml::ElementBuilder::new();
                                builder.set_default_ns(ns::JABBER_CLIENT.to_string());
                                builder.define_prefix(prefix.clone(), ns::STREAMS.to_string());
                            }
                            None => ()
                        }
                    }
                    Ok(xml::EndTag(xml::EndTag {
                        name: ref name,
                        ns: Some(ref ns), ..
                    })) if name.as_slice() == "stream" && ns.as_slice() == ns::STREAMS => {
                        println!("Stream closed");

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
        try!(self.socket.write(start.as_bytes()));
        self.socket.flush()
    }

    fn close_stream(&mut self) -> IoResult<()> {
        try!(self.socket.write(bytes!("</stream:stream>")));
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
                let starttls = stanza.child_with_name_and_ns("starttls",
                    Some(ns::FEATURE_TLS.to_string()));
                if starttls.is_some() {
                    let socket = &mut self.socket;
                    try!(socket.write(format!("<starttls xmlns='{}'/>",
                                              ns::FEATURE_TLS).as_bytes()));
                    return socket.flush();
                }

                // Auth mechanisms
                let mechs = stanza.child_with_name_and_ns("mechanisms",
                    Some(ns::FEATURE_SASL.to_string()));
                if mechs.is_some() {
                    return self.handle_mechs(mechs.unwrap());
                }

                // Bind
                let bind = stanza.child_with_name_and_ns("bind",
                    Some(ns::FEATURE_BIND.to_string()));
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
            } if name.as_slice() == "success" && ns.as_slice() == ns::FEATURE_SASL => {
                return self.start_stream();
            }
            _ => ()
        }
        Ok(())
    }

    fn handle_mechs(&mut self, mechs: &xml::Element) -> IoResult<()> {
        let mechs = mechs.children_with_name_and_ns("mechanism",
                                                    Some(ns::FEATURE_SASL.to_string()));

        for mech in mechs.iter() {
            let mech = mech.content_str();
            match mech.as_slice() {
                "PLAIN" => {
                    let auth: PlainAuth = Authenticator::new(self.username.as_slice(),
                                                             self.password.as_slice(), None);
                    self.authenticator = Some(box auth as Box<Authenticator>);
                }
                _ => continue
            }

            let data = self.authenticator.get_ref().initial().as_slice().to_base64(base64::STANDARD);

            let socket = &mut self.socket;
            try!(socket.write(format!("<auth mechanism='{}' xmlns='{}'>{}</auth>",
                                      mech, ns::FEATURE_SASL, data).as_bytes()));
            return socket.flush();
        }

        Ok(())
    }

    fn handle_bind(&mut self) -> IoResult<()> {
        try!(self.socket.write(format!("<iq type='set' id='bind'>\
                                         <bind xmlns='{}'/>\
                                       </iq>", ns::FEATURE_BIND).as_bytes()));
        self.socket.flush()
    }
}
