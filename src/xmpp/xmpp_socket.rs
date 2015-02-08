// rust-xmpp
// Copyright (c) 2014-2015 Florian Zeitz
//
// This project is MIT licensed.
// Please see the COPYING file for more information.

use std::mem;
use std::old_io::BufferedStream;
use std::old_io::{IoResult, IoError, OtherIoError};
use std::old_io::net::tcp::TcpStream;
use openssl::ssl::{SslContext, SslStream, SslMethod};

use read_str::ReadString;

pub enum XmppSocket {
    Tcp(BufferedStream<TcpStream>),
    Tls(BufferedStream<SslStream<TcpStream>>),
    NoSock
}

impl XmppSocket {
    pub fn starttls(&mut self) -> IoResult<()> {
        let socket = mem::replace(self, XmppSocket::NoSock);
        if let XmppSocket::Tcp(sock) = socket {
            let ctx = match SslContext::new(SslMethod::Sslv23) {
                Ok(ctx) => ctx,
                Err(_) => return Err(IoError {
                    kind: OtherIoError,
                    desc: "Could not create SSL context",
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
            *self = XmppSocket::Tls(BufferedStream::new(ssl));
        } else {
            panic!("No socket, or TLS already negotiated");
        }
        Ok(())
    }
}

impl Writer for XmppSocket {
    fn write_all(&mut self, buf: &[u8]) -> IoResult<()> {
        match *self {
            XmppSocket::Tcp(ref mut stream) => stream.write_all(buf),
            XmppSocket::Tls(ref mut stream) => stream.write_all(buf),
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
