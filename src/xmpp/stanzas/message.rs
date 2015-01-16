// rust-xmpp
// Copyright (c) 2014 Florian Zeitz
//
// This project is MIT licensed.
// Please see the COPYING file for more information.

use xml;
use ns;

use super::Stanza;

#[derive(Copy)]
pub enum MessageType {
    Normal,
    Headline,
    Chat,
    Groupchat,
    Error
}

pub struct Message { elem: xml::Element }

impl_Stanza!("message", Message, MessageType,
    |: ty: &str| {
        match ty {
            "headline" => Some(MessageType::Headline),
            "chat" => Some(MessageType::Chat),
            "groupchat" => Some(MessageType::Groupchat),
            "error" => Some(MessageType::Error),
            _ => None
        }
    }
, Some(MessageType::Normal));
