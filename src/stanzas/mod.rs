// rust-xmpp
// Copyright (c) 2014-2015 Florian Zeitz
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

pub trait StanzaType {
    fn attr_string(&self) -> Option<&'static str>;
}

pub trait Stanza {
    type Ty: StanzaType;

    fn from_element(e: xml::Element) -> Result<Self, xml::Element>;
    fn as_element(&self) -> &xml::Element;
    fn into_inner(self) -> xml::Element;

    fn to(&self) -> Option<&str>;
    fn from(&self) -> Option<&str>;
    fn id(&self) -> Option<&str>;
    fn stanza_type(&self) -> Option< <Self as Stanza>::Ty>;

    fn set_to(&mut self, to: Option<String>);
    fn set_from(&mut self, from: Option<String>);
    fn set_id(&mut self, id: Option<String>);
    fn set_stanza_type(&mut self, ty: <Self as Stanza>::Ty);
}

macro_rules! impl_Stanza(
    ($name: expr, $kind: ident, $ty: ty, $ty_some: expr, $ty_none: expr) => (
        impl Stanza for $kind {
            type Ty = $ty;
            fn from_element(e: xml::Element) -> ::std::result::Result<$kind, xml::Element> {
                match e.ns {
                    Some(ref ns) if *ns == ns::JABBER_CLIENT
                                    || *ns == ns::JABBER_SERVER => (),
                    _ => return Err(e)
                }

                if e.name == $name {
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

            fn to(&self) -> Option<&str> {
                self.elem.get_attribute("to", None)
            }

            fn from(&self) -> Option<&str> {
                self.elem.get_attribute("from", None)
            }

            fn id(&self) -> Option<&str> {
                self.elem.get_attribute("id", None)
            }

            fn stanza_type(&self) -> Option<$ty> {
                match self.elem.get_attribute("type", None) {
                    Some(ty) => ($ty_some)(ty),
                    None => $ty_none
                }
            }

            fn set_to(&mut self, to: Option<String>) {
                if let Some(to) = to {
                    self.set_attribute("to".into(), None, to);
                } else {
                    self.remove_attribute("to", None);
                }
            }

            fn set_from(&mut self, from: Option<String>) {
                if let Some(from) = from {
                    self.set_attribute("from".into(), None, from);
                } else {
                    self.remove_attribute("from", None);
                }
            }

            fn set_id(&mut self, id: Option<String>) {
                if let Some(id) = id {
                    self.set_attribute("id".into(), None, id);
                } else {
                    self.remove_attribute("id", None);
                }
            }

            fn set_stanza_type(&mut self, ty: <Self as Stanza>::Ty) {
                if let Some(ty) = ty.attr_string() {
                    self.set_attribute("type".into(), None, ty.into());
                } else {
                    self.remove_attribute("type", None);
                }
            }
        }

        impl ::std::ops::Deref for $kind {
            type Target = xml::Element;
            fn deref(&self) -> &xml::Element {
                &self.elem
            }
        }

        impl ::std::ops::DerefMut for $kind {
            fn deref_mut(&mut self) -> &mut xml::Element {
                &mut self.elem
            }
        }

        impl ::std::fmt::Display for $kind {
            fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
                self.elem.fmt(f)
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
            Some(ref ns) if *ns == ns::JABBER_CLIENT || *ns == ns::JABBER_SERVER => (),
            _ => return Err(e)
        }

        match &e.name[..] {
            "iq" => Stanza::from_element(e).map(AStanza::IqStanza),
            "message" => Stanza::from_element(e).map(AStanza::MessageStanza),
            "presence" => Stanza::from_element(e).map(AStanza::PresenceStanza),
            _ => Err(e)
        }
    }
}
