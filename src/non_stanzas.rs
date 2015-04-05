// rust-xmpp
// Copyright (c) 2015 Florian Zeitz
//
// This project is MIT licensed.
// Please see the COPYING file for more information.

use std::fmt;
use ns;
use xmpp_send::XmppSend;

#[derive(Debug)]
pub struct StreamStart<'a> {
    pub to: &'a str
}

impl<'a> fmt::Display for StreamStart<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "<?xml version='1.0'?>\n\
               <stream:stream xmlns:stream='{}' xmlns='{}' version='1.0' to='{}'>",
               ns::STREAMS, ns::JABBER_CLIENT, self.to)
    }
}

impl<'a> XmppSend for StreamStart<'a> {}

#[derive(Debug)]
pub struct StreamEnd;

impl fmt::Display for StreamEnd {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "</stream:stream>")
    }
}

impl XmppSend for StreamEnd {}

#[derive(Debug)]
pub struct StartTls;

impl fmt::Display for StartTls {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "<starttls xmlns='{}'/>", ns::FEATURE_TLS)
    }
}

impl XmppSend for StartTls {}

#[derive(Debug)]
pub struct AuthStart<'a> {
    pub mech: &'a str,
    pub data: &'a str
}

impl<'a> fmt::Display for AuthStart<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "<auth mechanism='{}' xmlns='{}'>{}</auth>",
               self.mech, ns::FEATURE_SASL, self.data)
    }
}

impl<'a> XmppSend for AuthStart<'a> {}

#[derive(Debug)]
pub struct AuthResponse<'a> {
    pub data: &'a str
}

impl<'a> fmt::Display for AuthResponse<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "<response xmlns='{}'>{}</response>", ns::FEATURE_SASL, self.data)
    }
}

impl<'a> XmppSend for AuthResponse<'a> {}
