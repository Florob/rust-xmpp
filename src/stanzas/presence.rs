// rust-xmpp
// Copyright (c) 2014-2015 Florian Zeitz
//
// This project is MIT licensed.
// Please see the COPYING file for more information.

use xml;
use ns;

use super::{Stanza, StanzaType};

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

impl StanzaType for PresenceType {
    fn attr_string(&self) -> Option<&'static str> {
        match *self {
            PresenceType::Error => Some("error"),
            PresenceType::Probe => Some("probe"),
            PresenceType::Subscribe => Some("subscribe"),
            PresenceType::Subscribed => Some("subscribed"),
            PresenceType::Unavailable => Some("unavailable"),
            PresenceType::Unsubscribe => Some("unsubscribe"),
            PresenceType::Unsubscribed => Some("unsubscribed"),
            PresenceType::Available => None
        }
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
        let elem = if let Some(ty) = ty.attr_string() {
            xml::Element::new("presence".into(), Some(ns::JABBER_CLIENT.into()),
                              vec![("type".into(), None, ty.into()),
                                   ("id".into(), None, id)])
        } else {
            xml::Element::new("presence".into(), Some(ns::JABBER_CLIENT.into()),
                              vec![("id".into(), None, id)])
        };
        Presence { elem: elem }
    }
}
