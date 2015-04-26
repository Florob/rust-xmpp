// rust-xmpp
// Copyright (c) 2014-2015 Florian Zeitz
//
// This project is MIT licensed.
// Please see the COPYING file for more information.

use xml;
use ns;

pub use self::iq::Iq;
pub use self::iq::IqType;
pub use self::message::Message;
pub use self::message::MessageType;
pub use self::presence::Presence;
pub use self::presence::PresenceType;

pub enum ErrorType {
    Auth,
    Cancel,
    Continue,
    Modify,
    Wait
}

impl ErrorType {
    fn attr_string(&self) -> &'static str {
        match *self {
            ErrorType::Auth => "auth",
            ErrorType::Cancel => "cancel",
            ErrorType::Continue => "continue",
            ErrorType::Modify => "modify",
            ErrorType::Wait => "wait"
        }
    }
}

pub enum DefinedCondition {
    BadRequest,
    Conflict,
    FeatureNotImplemented,
    Forbidden,
    Gone(String),
    InternalServerError,
    ItemNotFound,
    JidMalformed,
    NotAcceptable,
    NotAllowed,
    NotAuthorized,
    PolicyViolation,
    RecipientUnavailable,
    Redirect(String),
    RegistrationRequired,
    RemoteServerNotFound,
    RemoteServerTimeout,
    ResourceConstraint,
    ServiceUnavailable,
    SubscriptionRequired,
    UndefinedCondition,
    UnexpectedRequest
}

impl DefinedCondition {
    fn element(&self) -> xml::Element {
        fn make_cond(name: &str) -> xml::Element {
            xml::Element::new(name.into(), Some(ns::STANZAS.into()), vec![])
        }
        match *self {
            DefinedCondition::BadRequest => make_cond("bad-request"),
            DefinedCondition::Conflict => make_cond("conflict"),
            DefinedCondition::FeatureNotImplemented => make_cond("feature-not-implemented"),
            DefinedCondition::Forbidden => make_cond("forbidden"),
            DefinedCondition::Gone(ref g) => {
                let mut gone = make_cond("gone");
                gone.text(g.clone());
                gone
            }
            DefinedCondition::InternalServerError => make_cond("internal-server-error"),
            DefinedCondition::ItemNotFound => make_cond("item-not-found"),
            DefinedCondition::JidMalformed => make_cond("jid-malformed"),
            DefinedCondition::NotAcceptable => make_cond("not-acceptable"),
            DefinedCondition::NotAllowed => make_cond("not-allowed"),
            DefinedCondition::NotAuthorized => make_cond("not-authorized"),
            DefinedCondition::PolicyViolation => make_cond("policy-violation"),
            DefinedCondition::RecipientUnavailable => make_cond("recipient-unavailable"),
            DefinedCondition::Redirect(ref r) => {
                let mut redirect = make_cond("redirect");
                redirect.text(r.clone());
                redirect
            }
            DefinedCondition::RegistrationRequired => make_cond("registration-required"),
            DefinedCondition::RemoteServerNotFound => make_cond("remote-server-not-found"),
            DefinedCondition::RemoteServerTimeout => make_cond("remote-server-timeout"),
            DefinedCondition::ResourceConstraint => make_cond("resource-constraint"),
            DefinedCondition::ServiceUnavailable => make_cond("service-unavailable"),
            DefinedCondition::SubscriptionRequired => make_cond("subscription-required"),
            DefinedCondition::UndefinedCondition => make_cond("undefined-condition"),
            DefinedCondition::UnexpectedRequest => make_cond("unexpected-request")
        }
    }
}

pub trait StanzaType {
    fn attr_string(&self) -> Option<&'static str>;
}

pub trait Stanza {
    type Ty: StanzaType;

    fn from_element(e: xml::Element) -> Result<Self, xml::Element>;
    fn as_element(&self) -> &xml::Element;
    fn into_inner(self) -> xml::Element;

    fn to(&self) -> Option<&str>;
    fn from(&self) -> Option<&str>;
    fn id(&self) -> Option<&str>;
    fn stanza_type(&self) -> Option< <Self as Stanza>::Ty>;

    fn set_to(&mut self, to: Option<String>);
    fn set_from(&mut self, from: Option<String>);
    fn set_id(&mut self, id: Option<String>);
    fn set_stanza_type(&mut self, ty: <Self as Stanza>::Ty);

    fn error_reply(&self, ty: ErrorType, cond: DefinedCondition, text: Option<String>) -> Self;
}

macro_rules! impl_Stanza(
    ($name: expr, $kind: ident, $ty: ty, $ty_some: expr, $ty_none: expr) => (
        impl Stanza for $kind {
            type Ty = $ty;
            fn from_element(e: xml::Element) -> ::std::result::Result<$kind, xml::Element> {
                match e.ns {
                    Some(ref ns) if *ns == ns::JABBER_CLIENT
                                    || *ns == ns::JABBER_SERVER => (),
                    _ => return Err(e)
                }

                if e.name == $name {
                    Ok($kind { elem: e })
                } else {
                    Err(e)
                }
            }

            fn as_element(&self) -> &xml::Element {
                &self.elem
            }

            fn into_inner(self) -> xml::Element {
                self.elem
            }

            fn to(&self) -> Option<&str> {
                self.elem.get_attribute("to", None)
            }

            fn from(&self) -> Option<&str> {
                self.elem.get_attribute("from", None)
            }

            fn id(&self) -> Option<&str> {
                self.elem.get_attribute("id", None)
            }

            fn stanza_type(&self) -> Option<$ty> {
                match self.elem.get_attribute("type", None) {
                    Some(ty) => ($ty_some)(ty),
                    None => $ty_none
                }
            }

            fn set_to(&mut self, to: Option<String>) {
                if let Some(to) = to {
                    self.set_attribute("to".into(), None, to);
                } else {
                    self.remove_attribute("to", None);
                }
            }

            fn set_from(&mut self, from: Option<String>) {
                if let Some(from) = from {
                    self.set_attribute("from".into(), None, from);
                } else {
                    self.remove_attribute("from", None);
                }
            }

            fn set_id(&mut self, id: Option<String>) {
                if let Some(id) = id {
                    self.set_attribute("id".into(), None, id);
                } else {
                    self.remove_attribute("id", None);
                }
            }

            fn set_stanza_type(&mut self, ty: <Self as Stanza>::Ty) {
                if let Some(ty) = ty.attr_string() {
                    self.set_attribute("type".into(), None, ty.into());
                } else {
                    self.remove_attribute("type", None);
                }
            }

            fn error_reply(&self, ty: ::stanzas::ErrorType, cond: ::stanzas::DefinedCondition,
                           text: Option<String>) -> $kind
            {
                let to = self.from().map(|x| x.into());
                let id = self.id().unwrap_or("").into();
                let ty = ty.attr_string().into();

                let mut reply = $kind {
                    elem: xml::Element::new($name.into(), Some(ns::JABBER_CLIENT.into()),
                                            vec![("type".into(), None, "error".into()),
                                                 ("id".into(), None, id)])
                };
                {
                    let error = reply.tag(xml::Element::new("error".into(),
                                                            Some(ns::JABBER_CLIENT.into()),
                                                            vec![("type".into(), None, ty)]))
                                     .tag_stay(cond.element());
                    if let Some(text) = text {
                        error.tag(xml::Element::new("text".into(),
                                                    Some(ns::STANZAS.into()), vec![]))
                             .text(text);
                    }
                }
                reply.set_to(to);
                reply
            }
        }

        impl ::std::ops::Deref for $kind {
            type Target = xml::Element;
            fn deref(&self) -> &xml::Element {
                &self.elem
            }
        }

        impl ::std::ops::DerefMut for $kind {
            fn deref_mut(&mut self) -> &mut xml::Element {
                &mut self.elem
            }
        }

        impl ::std::fmt::Display for $kind {
            fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
                self.elem.fmt(f)
            }
        }
    );
);


// Has to be after impl_Stanza!
mod iq;
mod message;
mod presence;

pub enum AStanza {
    IqStanza(Iq),
    MessageStanza(Message),
    PresenceStanza(Presence)
}

impl AStanza {
    pub fn from_element(e: xml::Element) -> Result<AStanza, xml::Element> {
        match e.ns {
            Some(ref ns) if *ns == ns::JABBER_CLIENT || *ns == ns::JABBER_SERVER => (),
            _ => return Err(e)
        }

        match &e.name[..] {
            "iq" => Stanza::from_element(e).map(AStanza::IqStanza),
            "message" => Stanza::from_element(e).map(AStanza::MessageStanza),
            "presence" => Stanza::from_element(e).map(AStanza::PresenceStanza),
            _ => Err(e)
        }
    }
}
