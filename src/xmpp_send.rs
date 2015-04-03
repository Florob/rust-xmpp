// rust-xmpp
// Copyright (c) 2014 Florian Zeitz
//
// This project is MIT licensed.
// Please see the COPYING file for more information.

use xml::Element;
use std::borrow::Cow;

pub trait XmppSend<'a> {
    fn xmpp_str(self) -> Cow<'a, str>;
}

impl<'a> XmppSend<'a> for &'a str {
    fn xmpp_str(self) -> Cow<'a, str> {
        Cow::from(self)
    }
}

impl XmppSend<'static> for String {
    fn xmpp_str(self) -> Cow<'static, str> {
        Cow::from(self)
    }
}

impl<'a> XmppSend<'static> for &'a Element {
    fn xmpp_str(self) -> Cow<'static, str> {
        Cow::from(self.to_string())
    }
}
