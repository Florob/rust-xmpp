// rust-xmpp
// Copyright (c) 2014 Florian Zeitz
//
// This project is MIT licensed.
// Please see the COPYING file for more information.

use xml::Element;
use std::str::{MaybeOwned, Slice, Owned};

pub trait XmppSend {
    fn xmpp_str<'a>(&'a self) -> MaybeOwned<'a>;
}

impl<'s> XmppSend for &'s str {
    fn xmpp_str<'a>(&'a self) -> MaybeOwned<'a> {
        Slice(*self)
    }
}

impl XmppSend for String {
    fn xmpp_str<'a>(&'a self) -> MaybeOwned<'a> {
        Slice(self.as_slice())
    }
}

impl XmppSend for Element {
    fn xmpp_str<'a>(&'a self) -> MaybeOwned<'a> {
        Owned(format!("{}", *self))
    }
}
