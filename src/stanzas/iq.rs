// rust-xmpp
// Copyright (c) 2014 Florian Zeitz
//
// This project is MIT licensed.
// Please see the COPYING file for more information.

use xml;
use ns;

use super::Stanza;

#[derive(Copy, Clone)]
pub enum IqType {
    Set,
    Get,
    Result,
    Error
}

pub struct Iq { elem: xml::Element }

impl_Stanza!("iq", Iq, IqType,
    |ty: &str| {
        match ty {
            "get" => Some(IqType::Get),
            "set" => Some(IqType::Set),
            "result" => Some(IqType::Result),
            "error" => Some(IqType::Error),
            _ => None
        }
    }
, None);
