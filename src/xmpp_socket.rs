// rust-xmpp
// Copyright (c) 2014-2015 Florian Zeitz
//
// This project is MIT licensed.
// Please see the COPYING file for more information.

use std::io;
use std::io::{Write, BufReader};
use std::mem;
use std::net::TcpStream;
use openssl::ssl::{SslConnector, SslStream, SslMethod};

use crate::read_str::ReadString;

pub enum XmppSocket {
    Tcp(BufReader<TcpStream>, TcpStream),
    Tls(BufReader<SslStream<TcpStream>>),
    NoSock
}

impl XmppSocket {
    pub fn starttls(&mut self, domain: &str) -> io::Result<()> {
        let socket = mem::replace(self, XmppSocket::NoSock);
        if let XmppSocket::Tcp(_, sock) = socket {
            let ctx = match SslConnector::builder(SslMethod::tls()) {
                Ok(builder) => builder.build(),
                Err(_) => return Err(io::Error::new(io::ErrorKind::Other,
                                                    "Could not create SSL context"))
            };
            let ssl = match ctx.connect(domain, sock) {
                Ok(ssl) => ssl,
                Err(_) => return Err(io::Error::new(io::ErrorKind::Other,
                                                    "Could not create SSL stream"))
            };
            *self = XmppSocket::Tls(BufReader::new(ssl));
        } else {
            panic!("No socket, or TLS already negotiated");
        }
        Ok(())
    }
}

impl Write for XmppSocket {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        match *self {
            XmppSocket::Tcp(_, ref mut stream) => stream.write(buf),
            XmppSocket::Tls(ref mut stream) => stream.get_mut().write(buf),
            XmppSocket::NoSock => panic!("No socket yet")
        }
    }

    fn flush(&mut self) -> io::Result<()> {
        match *self {
            XmppSocket::Tcp(_, ref mut stream) => stream.flush(),
            XmppSocket::Tls(ref mut stream) => stream.get_mut().flush(),
            XmppSocket::NoSock => panic!("No socket yet")
        }
    }
}

impl ReadString for XmppSocket {
    fn read_str(&mut self) -> io::Result<String> {
        match *self {
            XmppSocket::Tcp(ref mut stream, _) => stream.read_str(),
            XmppSocket::Tls(ref mut stream) => stream.read_str(),
            XmppSocket::NoSock => panic!("Tried to read string before socket exists")
        }
    }
}
