#![crate_id = "xmpp#0.1"]
#![crate_type = "lib" ]

extern crate serialize;
extern crate xml;
extern crate openssl;

use std::str;
use std::mem;
use std::io::net::tcp::TcpStream;
use std::io::{Buffer, Stream};
use std::io::BufferedStream;
use std::io::IoResult;
use serialize::base64;
use serialize::base64::ToBase64;
use openssl::ssl::{SslContext, SslStream, Sslv23};

trait ReadString {
    fn read_str(&mut self) -> IoResult<String>;
}

impl<S: Stream> ReadString for BufferedStream<S> {
    fn read_str(&mut self) -> IoResult<String> {
        let (result, last) = {
            let available = try!(self.fill_buf());
            let len = available.len();
            let mut last = if len < 3 { 0 } else { len - 3 };
            while last < len {
                let width = str::utf8_char_width(available[last]);
                if width == 0 {
                    last += 1;
                    continue;
                }
                if last+width <= len {
                    last += width;
                } else {
                    break;
                }
            }
            (str::from_utf8(available.slice_to(last)).unwrap().to_string(), last)
        };
        self.consume(last);

        Ok(result)
    }
}

enum XmppSocket {
    Tcp(BufferedStream<TcpStream>),
    Tls(BufferedStream<SslStream<TcpStream>>),
    NoSock
}

impl Reader for XmppSocket {
    fn read(&mut self, buf: &mut [u8]) -> IoResult<uint> {
        match *self {
            Tcp(ref mut stream) => stream.read(buf),
            Tls(ref mut stream) => stream.read(buf),
            NoSock => fail!("No socket yet")
        }
    }
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
    socket: XmppSocket
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
                socket: NoSock
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
                    })) if name.as_slice() == "stream" && ns.as_slice() == "http://etherx.jabber.org/streams" => {
                        println!("Got stream start");
                        match *prefix {
                            Some(ref prefix) => {
                                *builder = xml::ElementBuilder::new();
                                builder.set_default_ns("jabber:client".to_string());
                                builder.define_prefix(prefix.clone(), "http://etherx.jabber.org/streams".to_string());
                            }
                            None => ()
                        }
                    }
                    Ok(xml::EndTag(xml::EndTag {
                        name: ref name,
                        ns: Some(ref ns), ..
                    })) if name.as_slice() == "stream" && ns.as_slice() == "http://etherx.jabber.org/streams" => {
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
                             <stream:stream xmlns:stream='http://etherx.jabber.org/streams' \
                             xmlns='jabber:client' version='1.0' to='{}'>", self.domain);
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
            } if name.as_slice() == "features" && ns.as_slice() == "http://etherx.jabber.org/streams" => {
                // StartTLS
                let starttls = stanza.child_with_name_and_ns("starttls",
                    Some("urn:ietf:params:xml:ns:xmpp-tls".to_string()));
                if starttls.is_some() {
                    let socket = &mut self.socket;
                    try!(socket.write(bytes!("<starttls xmlns='urn:ietf:params:xml:ns:xmpp-tls'/>")));
                    return socket.flush();
                }

                // Auth mechanisms
                let mechs = stanza.child_with_name_and_ns("mechanisms",
                    Some("urn:ietf:params:xml:ns:xmpp-sasl".to_string()));
                if mechs.is_some() {
                    return self.handle_mechs(mechs.unwrap());
                }

                // Bind
                let bind = stanza.child_with_name_and_ns("bind",
                    Some("urn:ietf:params:xml:ns:xmpp-bind".to_string()));
                if bind.is_some() {
                    return self.handle_bind();
                }
            }

            &xml::Element {
                name: ref name,
                ns: Some(ref ns), ..
            } if name.as_slice() == "proceed" && ns.as_slice() == "urn:ietf:params:xml:ns:xmpp-tls" => {
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
            } if name.as_slice() == "success" && ns.as_slice() == "urn:ietf:params:xml:ns:xmpp-sasl" => {
                return self.start_stream();
            }
            _ => ()
        }
        Ok(())
    }

    fn handle_mechs(&mut self, mechs: &xml::Element) -> IoResult<()> {
        let mechs = mechs.children_with_name_and_ns("mechanism",
                                                    Some("urn:ietf:params:xml:ns:xmpp-sasl".to_string()));

        for mech in mechs.iter() {
            if mech.content_str().as_slice() == "PLAIN" {
                let mut data: Vec<u8> = vec![0];
                data.push_all(self.username.as_bytes());
                data.push(0);
                data.push_all(self.password.as_bytes());
                let data = data.as_slice().to_base64(base64::STANDARD);

                let socket = &mut self.socket;
                try!(socket.write(bytes!("<auth mechanism='PLAIN' \
                                           xmlns='urn:ietf:params:xml:ns:xmpp-sasl'>")));
                try!(socket.write(data.as_bytes()));
                try!(socket.write(bytes!("</auth>")));
                return socket.flush();
            }
        }

        Ok(())
    }

    fn handle_bind(&mut self) -> IoResult<()> {
        try!(self.socket.write(bytes!("<iq type='set' id='bind'>\
                                         <bind xmlns='urn:ietf:params:xml:ns:xmpp-bind'/>\
                                       </iq>")));
        self.socket.flush()
    }
}
