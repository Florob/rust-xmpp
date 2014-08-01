// rust-xmpp
// Copyright (c) 2014 Florian Zeitz
//
// This project is MIT licensed.
// Please see the COPYING file for more information.

use xml;
use ns;

use super:: Stanza;

pub enum PresenceType {
    Error,
    Probe,
    Subscribe,
    Subscribed,
    Unavailable,
    Unsubscribe,
    Unsubscribed,
    Available
}

pub struct Presence { elem: xml::Element }

impl Stanza<PresenceType> for Presence {
    fn from_element(e: xml::Element) -> Result<Presence, xml::Element> {
        match e.ns {
            Some(ref ns) if ns.as_slice() == ns::JABBER_CLIENT
                            || ns.as_slice() == ns::JABBER_SERVER => (),
            _ => return Err(e)
        }

        if e.name.as_slice() == "presence" {
            Ok(Presence { elem: e })
        } else {
            Err(e)
        }
    }

    fn as_element(&self) -> &xml::Element {
        &self.elem
    }

    fn get_to(&self) -> Option<&str> {
        self.elem.get_attribute("to", None).map(|to| to.value.as_slice())
    }

    fn get_from(&self) -> Option<&str> {
        self.elem.get_attribute("from", None).map(|from| from.value.as_slice())
    }

    fn get_id(&self) -> Option<&str> {
        self.elem.get_attribute("id", None).map(|id| id.value.as_slice())
    }

    fn get_type(&self) -> Option<PresenceType> {
        match self.elem.get_attribute("type", None) {
            Some(ref ty) => match ty.value.as_slice() {
                "error" => Some(Error),
                "probe" => Some(Probe),
                "subscribe" => Some(Subscribe),
                "subscribed" => Some(Subscribed),
                "unavailable" => Some(Unavailable),
                "unsubscribe" => Some(Unsubscribe),
                "unsubscribed" => Some(Unsubscribed),
                _ => None
            },
            None => Some(Available)
        }
    }
}
