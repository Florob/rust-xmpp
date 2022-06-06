// rust-xmpp
// Copyright (c) 2014-2015 Florian Zeitz
//
// This project is MIT licensed.
// Please see the COPYING file for more information.

use xml;
use crate::ns;

use super::{Stanza, StanzaType};

#[derive(Copy, Clone, Debug)]
pub enum IqType {
    Set,
    Get,
    Result,
    Error
}

impl StanzaType for IqType {
    fn attr_string(&self) -> Option<&'static str> {
        Some(match *self {
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
                                    vec![("type".into(), None, ty.attr_string().unwrap().into()),
                                         ("id".into(), None, id)])
        }
    }

    pub fn get_xmpp_bind_jid(&self) -> Option<String> {
        let ns = Some(ns::FEATURE_BIND);
        self.elem
            .get_child("bind", ns)
            .and_then(|bind| bind.get_child("jid", ns))
            .map(|jid| jid.content_str())
    }
}
