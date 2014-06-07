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
