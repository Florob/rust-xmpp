// rust-xmpp
// Copyright (c) 2014-2015 Florian Zeitz
//
// This project is MIT licensed.
// Please see the COPYING file for more information.

use xml;
use crate::ns;

use super::{Stanza, StanzaType};

#[derive(Copy, Clone)]
pub enum MessageType {
    Normal,
    Headline,
    Chat,
    Groupchat,
    Error
}

impl StanzaType for MessageType {
    fn attr_string(&self) -> Option<&'static str> {
        Some(match *self {
            MessageType::Normal => "normal",
            MessageType::Headline => "headline",
            MessageType::Chat => "chat",
            MessageType::Groupchat => "groupchat",
            MessageType::Error => "error"
        })
    }
}

#[derive(Clone)]
pub struct Message { elem: xml::Element }

impl_Stanza!("message", Message, MessageType,
    |ty: &str| {
        match ty {
            "normal" => Some(MessageType::Normal),
            "headline" => Some(MessageType::Headline),
            "chat" => Some(MessageType::Chat),
            "groupchat" => Some(MessageType::Groupchat),
            "error" => Some(MessageType::Error),
            _ => None
        }
    }
, Some(MessageType::Normal));

impl Message {
    pub fn new(ty: MessageType, id: String) -> Message {
        Message {
            elem: xml::Element::new("message".into(), Some(ns::JABBER_CLIENT.into()),
                                    vec![("type".into(), None, ty.attr_string().unwrap().into()),
                                         ("id".into(), None, id)])
        }
    }
}
