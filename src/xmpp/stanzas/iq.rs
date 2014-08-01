// rust-xmpp
// Copyright (c) 2014 Florian Zeitz
//
// This project is MIT licensed.
// Please see the COPYING file for more information.

use xml;
use ns;

use super:: Stanza;

pub enum IqType {
    Set,
    Get,
    Result,
    Error
}

pub struct Iq { elem: xml::Element }

impl Stanza<IqType> for Iq {
    fn from_element(e: xml::Element) -> Result<Iq, xml::Element> {
        match e.ns {
            Some(ref ns) if ns.as_slice() == ns::JABBER_CLIENT
                            || ns.as_slice() == ns::JABBER_SERVER => (),
            _ => return Err(e)
        }

        if e.name.as_slice() == "iq" {
            Ok(Iq { elem: e })
        } else {
            Err(e)
        }
    }

    fn as_element(&self) -> &xml::Element {
        &self.elem
    }

    fn get_to(&self) -> Option<&str> {
        self.elem.get_attribute("to", None).map(|to| to.value.as_slice())
    }

    fn get_from(&self) -> Option<&str> {
        self.elem.get_attribute("from", None).map(|from| from.value.as_slice())
    }

    fn get_id(&self) -> Option<&str> {
        self.elem.get_attribute("id", None).map(|id| id.value.as_slice())
    }

    fn get_type(&self) -> Option<IqType> {
        self.elem.get_attribute("type", None).and_then(|ty| {
            match ty.value.as_slice() {
                "get" => Some(Get),
                "set" => Some(Set),
                "result" => Some(Result),
                "error" => Some(Error),
                _ => None
            }
        })
    }
}
