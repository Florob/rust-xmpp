#[crate_id = "xmpp#0.1"];
#[crate_type = "lib" ];
#[forbid(non_camel_case_types)];

extern mod extra;
extern mod xml;
extern mod openssl = "github.com/sfackler/rust-openssl";

use std::str;
use std::util;
use std::io::net::addrinfo::get_host_addresses;
use std::io::net::ip::SocketAddr;
use std::io::net::tcp::TcpStream;
use std::io::{Buffer, Stream};
use std::io::BufferedStream;
use extra::base64;
use extra::base64::ToBase64;
use openssl::ssl::{SslContext, SslStream, Sslv23};

trait ReadString {
    fn read_str(&mut self) -> ~str;
}

impl<S: Stream> ReadString for BufferedStream<S> {
    fn read_str(&mut self) -> ~str {
        let (result, last) = {
            let available = self.fill();
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
            (str::from_utf8(available.slice_to(last)).to_owned(), last)
        };
        self.consume(last);
        return result;
    }
}

enum XmppSocket {
    Tcp(BufferedStream<TcpStream>),
    Tls(BufferedStream<SslStream<TcpStream>>),
    NoSock
}

impl Reader for XmppSocket {
    fn read(&mut self, buf: &mut [u8]) -> Option<uint> {
        match *self {
            Tcp(ref mut stream) => stream.read(buf),
            Tls(ref mut stream) => stream.read(buf),
            NoSock => fail!("No socket yet")
        }
    }
}

impl Writer for XmppSocket {
    fn write(&mut self, buf: &[u8]) {
        match *self {
            Tcp(ref mut stream) => stream.write(buf),
            Tls(ref mut stream) => stream.write(buf),
            NoSock => fail!("No socket yet")
        }
    }

    fn flush(&mut self) {
        match *self {
            Tcp(ref mut stream) => stream.flush(),
            Tls(ref mut stream) => stream.flush(),
            NoSock => fail!("No socket yet")
        }
    }
}

impl ReadString for XmppSocket {
    fn read_str(&mut self) -> ~str {
        match *self {
            Tcp(ref mut stream) => stream.read_str(),
            Tls(ref mut stream) => stream.read_str(),
            NoSock => fail!("No socket yet")
        }
    }
}

struct XmppHandler {
    priv username: ~str,
    priv password: ~str,
    priv domain: ~str,
    priv socket: XmppSocket
}

pub struct XmppStream {
    priv parser: xml::Parser,
    priv builder: xml::ElementBuilder,
    priv handler: XmppHandler
}

impl XmppStream {
    pub fn new(user: ~str, domain: ~str, password: ~str) -> XmppStream {
        XmppStream {
            parser: xml::Parser::new(),
            builder: xml::ElementBuilder::new(),
            handler: XmppHandler {
                username: user,
                password: password,
                domain: domain,
                socket: NoSock
            }
        }
    }

    pub fn connect(&mut self) {
        let addresses = get_host_addresses(self.handler.domain).expect("Failed to resolve domain");

        let mut stream = None;
        for ip in addresses.iter() {
            let addr = SocketAddr { ip: *ip, port: 5222 };
            stream = TcpStream::connect(addr);
            if stream.is_some() { break; }
        }
        let stream = stream.expect("Failed to connect");

        self.handler.socket = Tcp(BufferedStream::new(stream));

        self.handler.start_stream();
    }

    pub fn handle(&mut self) {
        let mut closed = false;
        while !closed {
            let string = {
                let socket = &mut self.handler.socket;
                socket.read_str()
            };
            self.parser.parse_str(string, |event| {
                match event {
                    Ok(xml::StartTag(xml::StartTag {
                        name: ~"stream",
                        ns: Some(~"http://etherx.jabber.org/streams"),
                        prefix: prefix, ..
                    })) => {
                        println!("Got stream start");
                        match prefix {
                            Some(prefix) => {
                                self.builder = xml::ElementBuilder::new();
                                self.builder.set_default_ns(~"jabber:client");
                                self.builder.define_prefix(prefix,
                                                           ~"http://etherx.jabber.org/streams");
                            }
                            None => ()
                        }
                    }
                    Ok(xml::EndTag(xml::EndTag {
                        name: ~"stream",
                        ns: Some(~"http://etherx.jabber.org/streams"), ..
                    })) => {
                        println!("Stream closed");

                        let socket = &mut self.handler.socket;
                        socket.write(bytes!("</stream:stream>"));
                        socket.flush();
                        closed = true;
                    }
                    Ok(event) => match self.builder.push_event(event) {
                        Ok(Some(ref e)) => { self.handler.handle_stanza(e); }
                        Ok(None) => (),
                        Err(e) => println!("{}", e),
                    },
                    Err(e) => println!("Line: {} Column: {} Msg: {}", e.line, e.col, e.msg),
                }
            });
        }
    }
}

impl XmppHandler {
    fn start_stream(&mut self) {
        let socket = &mut self.socket;
        socket.write(bytes!("<?xml version='1.0'?>\n\
                             <stream:stream xmlns:stream='http://etherx.jabber.org/streams' \
                             xmlns='jabber:client' version='1.0' "));
        let to = format!("to='{}'>", self.domain);
        socket.write(to.as_bytes());
        socket.flush();
    }

    fn handle_stanza(&mut self, stanza: &xml::Element) {
        println!("In: {}", *stanza)
        match stanza {
            &xml::Element {
                name: ~"features",
                ns: Some(~"http://etherx.jabber.org/streams"), ..
            } => {
                // StartTLS
                let starttls = stanza.child_with_name_and_ns("starttls",
                    Some(~"urn:ietf:params:xml:ns:xmpp-tls"));
                if starttls.is_some() {
                    let socket = &mut self.socket;
                    socket.write(bytes!("<starttls xmlns='urn:ietf:params:xml:ns:xmpp-tls'/>"));
                    socket.flush();
                    return;
                }

                // Auth mechanisms
                let mechs = stanza.child_with_name_and_ns("mechanisms",
                    Some(~"urn:ietf:params:xml:ns:xmpp-sasl"));
                if mechs.is_some() {
                    self.handle_mechs(mechs.unwrap());
                    return;
                }

                // Bind
                let bind = stanza.child_with_name_and_ns("bind",
                    Some(~"urn:ietf:params:xml:ns:xmpp-bind"));
                if bind.is_some() {
                    self.handle_bind();
                    return;
                }
            }

            &xml::Element {
                name: ~"proceed",
                ns: Some(~"urn:ietf:params:xml:ns:xmpp-tls"), ..
            } => {
                let socket = util::replace(&mut self.socket, NoSock);
                match socket {
                    Tcp(sock) => {
                        let ctx = SslContext::new(Sslv23);
                        let ssl = SslStream::new(&ctx, sock.unwrap());
                        self.socket = Tls(BufferedStream::new(ssl));
                        self.start_stream();
                    }
                    _ => fail!("No socket, or TLS already negotiated")
                }
            }

            &xml::Element {
                name: ~"success",
                ns: Some(~"urn:ietf:params:xml:ns:xmpp-sasl"), ..
            } => {
                self.start_stream();
            }
            _ => ()
        }
    }

    fn handle_mechs(&mut self, mechs: &xml::Element) {
        let mechs = mechs.children_with_name_and_ns("mechanism",
                                                    Some(~"urn:ietf:params:xml:ns:xmpp-sasl"));

        for mech in mechs.iter() {
            if mech.content_str() == ~"PLAIN" {
                let mut data: ~[u8] = ~[0];
                data.push_all(self.username.as_bytes());
                data.push(0);
                data.push_all(self.password.as_bytes());
                let data = data.to_base64(base64::STANDARD);

                let socket = &mut self.socket;
                socket.write(bytes!("<auth mechanism='PLAIN' \
                                      xmlns='urn:ietf:params:xml:ns:xmpp-sasl'>"));
                socket.write(data.as_bytes());
                socket.write(bytes!("</auth>"));
                socket.flush();
                break;
            }
        }
    }

    fn handle_bind(&mut self) {
        let socket = &mut self.socket;
        socket.write(bytes!("<iq type='set' id='bind'>\
                                 <bind xmlns='urn:ietf:params:xml:ns:xmpp-bind'/>\
                             </iq>"));
        socket.flush();
    }
}
