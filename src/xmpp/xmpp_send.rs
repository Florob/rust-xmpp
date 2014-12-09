// rust-xmpp
// Copyright (c) 2014 Florian Zeitz
//
// This project is MIT licensed.
// Please see the COPYING file for more information.

use xml::Element;
use std::str::CowString;
use std::borrow::IntoCow;

pub trait XmppSend {
    fn xmpp_str<'a>(&'a self) -> CowString<'a>;
}

impl<'s> XmppSend for &'s str {
    fn xmpp_str<'a>(&'a self) -> CowString<'a> {
        self.into_cow()
    }
}

impl XmppSend for String {
    fn xmpp_str<'a>(&'a self) -> CowString<'a> {
        self.as_slice().into_cow()
    }
}

impl XmppSend for Element {
    fn xmpp_str<'a>(&'a self) -> CowString<'a> {
        (format!("{}", *self)).into_cow()
    }
}
