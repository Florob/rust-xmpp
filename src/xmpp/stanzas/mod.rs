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

pub trait Stanza {
    type Ty;

    fn from_element(e: xml::Element) -> Result<Self, xml::Element>;
    fn as_element(&self) -> &xml::Element;
    fn into_inner(self) -> xml::Element;
    fn get_to(&self) -> Option<&str>;
    fn get_from(&self) -> Option<&str>;
    fn get_id(&self) -> Option<&str>;
    fn get_type(&self) -> Option< <Self as Stanza>::Ty>;
}

macro_rules! impl_Stanza(
    ($name: expr, $kind: ident, $ty: ty, $ty_some: expr, $ty_none: expr) => (
        impl Stanza for $kind {
            type Ty = $ty;
            fn from_element(e: xml::Element) -> ::std::result::Result<$kind, xml::Element> {
                match e.ns {
                    Some(ref ns) if &ns[] == ns::JABBER_CLIENT
                                    || &ns[] == ns::JABBER_SERVER => (),
                    _ => return Err(e)
                }

                if &e.name[] == $name {
                    Ok($kind { elem: e })
                } else {
                    Err(e)
                }
            }

            fn as_element(&self) -> &xml::Element {
                &self.elem
            }

            fn into_inner(self) -> xml::Element {
                self.elem
            }

            fn get_to(&self) -> Option<&str> {
                self.elem.get_attribute("to", None)
            }

            fn get_from(&self) -> Option<&str> {
                self.elem.get_attribute("from", None)
            }

            fn get_id(&self) -> Option<&str> {
                self.elem.get_attribute("id", None)
            }

            fn get_type(&self) -> Option<$ty> {
                match self.elem.get_attribute("type", None) {
                    Some(ty) => ($ty_some)(ty),
                    None => $ty_none
                }
            }
        }
    );
);

// Has to be after impl_Stanza!
mod iq;
mod message;
mod presence;

pub enum AStanza {
    IqStanza(Iq),
    MessageStanza(Message),
    PresenceStanza(Presence)
}

impl AStanza {
    pub fn from_element(e: xml::Element) -> Result<AStanza, xml::Element> {
        match e.ns {
            Some(ref ns) if &ns[] == ns::JABBER_CLIENT
                            || &ns[] == ns::JABBER_SERVER => (),
            _ => return Err(e)
        }

        match &e.name[] {
            "iq" => Stanza::from_element(e).and_then(|s| Ok(AStanza::IqStanza(s))),
            "message" => Stanza::from_element(e).and_then(|s| Ok(AStanza::MessageStanza(s))),
            "presence" => Stanza::from_element(e).and_then(|s| Ok(AStanza::PresenceStanza(s))),
            _ => Err(e)
        }
    }
}
