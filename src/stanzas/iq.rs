// rust-xmpp
// Copyright (c) 2014 Florian Zeitz
//
// This project is MIT licensed.
// Please see the COPYING file for more information.

use xml;
use ns;

use std::fmt;

use super::Stanza;

#[derive(Copy, Clone, Debug)]
pub enum IqType {
    Set,
    Get,
    Result,
    Error
}

impl fmt::Display for IqType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str(match *self {
            IqType::Set => "set",
            IqType::Get => "get",
            IqType::Result => "result",
            IqType::Error => "error"
        })
    }
}

#[derive(Clone)]
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

impl Iq {
    pub fn new(ty: IqType, id: String) -> Iq {
        Iq {
            elem: xml::Element::new("iq".into(), Some(ns::JABBER_CLIENT.into()),
                                    vec![("type".into(), None, ty.to_string()),
                                         ("id".into(), None, id)])
        }
    }
}
