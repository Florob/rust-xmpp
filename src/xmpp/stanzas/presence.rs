// rust-xmpp
// Copyright (c) 2014 Florian Zeitz
//
// This project is MIT licensed.
// Please see the COPYING file for more information.

use xml;
use ns;

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
            "error" => Some(PresenceType::Error),
            "probe" => Some(PresenceType::Probe),
            "subscribe" => Some(PresenceType::Subscribe),
            "subscribed" => Some(PresenceType::Subscribed),
            "unavailable" => Some(PresenceType::Unavailable),
            "unsubscribe" => Some(PresenceType::Unsubscribe),
            "unsubscribed" => Some(PresenceType::Unsubscribed),
            _ => None
        }
    }
, Some(PresenceType::Available))
