// rust-xmpp
// Copyright (c) 2014 Florian Zeitz
//
// This project is MIT licensed.
// Please see the COPYING file for more information.

use xml;
use ns;
pub use self::PresenceType::{Error,Probe,Subscribe,Subscribed,Unavailable,Unsubscribe,Unsubscribed,Available};

use super::Stanza;

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

impl_Stanza!("presence", Presence, PresenceType,
    |ty: &str| {
        match ty {
            "error" => Some(Error),
            "probe" => Some(Probe),
            "subscribe" => Some(Subscribe),
            "subscribed" => Some(Subscribed),
            "unavailable" => Some(Unavailable),
            "unsubscribe" => Some(Unsubscribe),
            "unsubscribed" => Some(Unsubscribed),
            _ => None
        }
    }
, Some(Available))
