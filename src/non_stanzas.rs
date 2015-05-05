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

#[allow(dead_code)]
#[derive(Debug)]
pub enum DefinedCondition {
    BadFormat,
    BadNamespacePrefix,
    Conflict,
    ConnectionTimeout,
    HostGone,
    HostUnknown,
    ImproperAddressing,
    InternalServerError,
    InvalidFrom,
    InvalidId,
    InvalidNamespace,
    InvalidXml,
    NotAuthorized,
    NotWellFormed,
    PolicyViolation,
    RemoteConnectionFailed,
    Reset,
    ResourceConstraint,
    RestrictedXml,
    SeeOtherHost(String),
    SystemShutdown,
    UndefinedCondition,
    UnsupportedEncoding,
    UnsupportedStanzaType,
    UnsupportedVersion,
}

impl fmt::Display for DefinedCondition {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let name = match *self {
            DefinedCondition::BadFormat => "bad-format",
            DefinedCondition::BadNamespacePrefix => "bad-namespace-prefix",
            DefinedCondition::Conflict => "conflict",
            DefinedCondition::ConnectionTimeout => "connection-timeout",
            DefinedCondition::HostGone => "host-gone",
            DefinedCondition::HostUnknown => "host-unknown",
            DefinedCondition::ImproperAddressing => "improper-addressing",
            DefinedCondition::InternalServerError => "internal-server-error",
            DefinedCondition::InvalidFrom => "invalid-from",
            DefinedCondition::InvalidId => "invalid-id",
            DefinedCondition::InvalidNamespace => "invalid-namespace",
            DefinedCondition::InvalidXml => "invalid-xml",
            DefinedCondition::NotAuthorized => "not-authorized",
            DefinedCondition::NotWellFormed => "not-well-formed",
            DefinedCondition::PolicyViolation => "policy-violation",
            DefinedCondition::RemoteConnectionFailed => "remote-connection-failed",
            DefinedCondition::Reset => "reset",
            DefinedCondition::ResourceConstraint => "resource-constraint",
            DefinedCondition::RestrictedXml => "restricted-xml",
            DefinedCondition::SeeOtherHost(ref host) => {
                return write!(f, "<see-other-host xmlns='{}'>{}</see-other-host>",
                              ns::STREAM_ERRORS, host);
            }
            DefinedCondition::SystemShutdown => "system-shutdown",
            DefinedCondition::UndefinedCondition => "undefined-condition",
            DefinedCondition::UnsupportedEncoding => "unsupported-encoding",
            DefinedCondition::UnsupportedStanzaType => "unsupported-stanza-type",
            DefinedCondition::UnsupportedVersion => "unsupported-version"
        };
        write!(f, "<{} xmlns='{}'/>", name, ns::STREAM_ERRORS)
    }
}

#[derive(Debug)]
pub struct StreamError<'a> {
    pub cond: DefinedCondition,
    pub text: Option<&'a str>
}

impl<'a> fmt::Display for StreamError<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        try!(write!(f, "<stream:error>{}", self.cond));
        if let Some(text) = self.text {
            try!(write!(f, "<text xmlns='{}'>{}</text>", ns::STREAM_ERRORS, text));
        }
        write!(f, "</stream:error>")
    }
}

impl<'a> XmppSend for StreamError<'a> {}
