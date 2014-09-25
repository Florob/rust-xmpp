// rust-xmpp
// Copyright (c) 2014 Florian Zeitz
//
// This project is MIT licensed.
// Please see the COPYING file for more information.

use xml;
use ns;

use super::Stanza;

pub enum MessageType {
    Normal,
    Headline,
    Chat,
    Groupchat,
    Error
}

pub struct Message { elem: xml::Element }

impl_Stanza!("message", Message, MessageType,
    |ty: &str| {
        match ty {
            "headline" => Some(Headline),
            "chat" => Some(Chat),
            "groupchat" => Some(Groupchat),
            "error" => Some(Error),
            _ => None
        }
    }
, Some(Normal))
