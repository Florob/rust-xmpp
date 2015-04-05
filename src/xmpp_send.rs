// rust-xmpp
// Copyright (c) 2014-2015 Florian Zeitz
//
// This project is MIT licensed.
// Please see the COPYING file for more information.

use std::fmt;
use std::io;

pub trait XmppSend {
    fn xmpp_send<W: io::Write>(&self, w: &mut W) -> io::Result<()>;
}

impl<T> XmppSend for T where T: fmt::Display {
    fn xmpp_send<W: io::Write>(&self, w: &mut W) -> io::Result<()> {
        write!(w, "{}", self)
    }
}
