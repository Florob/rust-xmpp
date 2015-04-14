// rust-xmpp
// Copyright (c) 2014 Florian Zeitz
//
// This project is MIT licensed.
// Please see the COPYING file for more information.

use xml;
use ns;

use std::fmt;

use super::Stanza;

#[derive(Copy, Clone)]
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

impl fmt::Display for PresenceType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str(match *self {
            PresenceType::Error => "error",
            PresenceType::Probe => "probe",
            PresenceType::Subscribe => "subscribe",
            PresenceType::Subscribed => "subscribed",
            PresenceType::Unavailable => "unavailable",
            PresenceType::Unsubscribe => "unsubscribe",
            PresenceType::Unsubscribed => "unsubscribed",
            PresenceType::Available => "available"
        })
    }
}

#[derive(Clone)]
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
, Some(PresenceType::Available));

impl Presence {
    pub fn new(ty: PresenceType, id: String) -> Presence {
        let elem = if let PresenceType::Available = ty {
            xml::Element::new("presence".into(), Some(ns::JABBER_CLIENT.into()),
                              vec![("id".into(), None, id)])
        } else {
            xml::Element::new("presence".into(), Some(ns::JABBER_CLIENT.into()),
                              vec![("type".into(), None, ty.to_string()),
                                   ("id".into(), None, id)])
        };
        Presence { elem: elem }
    }
}
