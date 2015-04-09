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
pub enum MessageType {
    Normal,
    Headline,
    Chat,
    Groupchat,
    Error
}

impl fmt::Display for MessageType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str(match *self {
            MessageType::Normal => "normal",
            MessageType::Headline => "headline",
            MessageType::Chat => "chat",
            MessageType::Groupchat => "groupchat",
            MessageType::Error => "error"
        })
    }
}

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
                                    vec![("type".into(), None, ty.to_string()),
                                         ("id".into(), None, id)])
        }
    }
}
