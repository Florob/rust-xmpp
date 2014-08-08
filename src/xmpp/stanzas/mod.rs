// rust-xmpp
// Copyright (c) 2014 Florian Zeitz
//
// This project is MIT licensed.
// Please see the COPYING file for more information.

use xml;
use ns;

pub use self::iq::Iq;
pub use self::iq::IqType;
pub use self::message::Message;
pub use self::message::MessageType;
pub use self::presence::Presence;
pub use self::presence::PresenceType;

pub trait Stanza<Type> {
    fn from_element(e: xml::Element) -> Result<Self, xml::Element>;
    fn as_element(&self) -> &xml::Element;
    fn unwrap(self) -> xml::Element;
    fn get_to(&self) -> Option<&str>;
    fn get_from(&self) -> Option<&str>;
    fn get_id(&self) -> Option<&str>;
    fn get_type(&self) -> Option<Type>;
}

macro_rules! impl_Stanza(
    ($name: expr, $kind: ident, $ty: ident, $ty_some: expr, $ty_none: expr) => (
        impl Stanza<$ty> for $kind {
            fn from_element(e: xml::Element) -> Result<$kind, xml::Element> {
                match e.ns {
                    Some(ref ns) if ns.as_slice() == ns::JABBER_CLIENT
                                    || ns.as_slice() == ns::JABBER_SERVER => (),
                    _ => return Err(e)
                }

                if e.name.as_slice() == $name {
                    Ok($kind { elem: e })
                } else {
                    Err(e)
                }
            }

            fn as_element(&self) -> &xml::Element {
                &self.elem
            }

            fn unwrap(self) -> xml::Element {
                self.elem
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

            fn get_type(&self) -> Option<$ty> {
                match self.elem.get_attribute("type", None) {
                    Some(ty) => ($ty_some)(ty),
                    None => $ty_none
                }
            }
        }
    );
)

// Has to be after impl_Stanza!
mod iq;
mod message;
mod presence;

pub enum AStanza {
    Iq(Iq),
    Message(Message),
    Presence(Presence)
}

impl AStanza {
    pub fn from_element(e: xml::Element) -> Result<AStanza, xml::Element> {
        match e.ns {
            Some(ref ns) if ns.as_slice() == ns::JABBER_CLIENT
                            || ns.as_slice() == ns::JABBER_SERVER => (),
            _ => return Err(e)
        }

        match e.name.as_slice() {
            "iq" => Ok(Iq(Stanza::from_element(e).unwrap())),
            "message" => Ok(Message(Stanza::from_element(e).unwrap())),
            "presence" => Ok(Presence(Stanza::from_element(e).unwrap())),
            _ => Err(e)
        }
    }
}
