// rust-xmpp
// Copyright (c) 2014-2015 Florian Zeitz
//
// This project is MIT licensed.
// Please see the COPYING file for more information.

use std::{fmt, io};
use xml;

pub trait XmppSend: fmt::Display {
    fn xmpp_send<W: io::Write>(&self, w: &mut W) -> io::Result<()> {
        write!(w, "{}", self)
    }
}

impl XmppSend for xml::Element {}
