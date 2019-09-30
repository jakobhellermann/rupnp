#![feature(const_generics, generators, stmt_expr_attributes, proc_macro_hygiene)]
#![allow(incomplete_features)]

pub mod device;
mod discovery;
pub mod error;
pub mod scpd;
pub mod service;

pub use device::Device;
pub use error::Error;
pub use scpd::datatypes::Bool;
pub use scpd::SCPD;
pub use service::Service;

pub use discovery::discover;
pub use ssdp_client;

trait HttpResponseExt: Sized {
    fn err_if_not_200(self) -> Result<Self, Error>;
}
impl HttpResponseExt for isahc::http::Response<isahc::Body> {
    fn err_if_not_200(self) -> Result<Self, Error> {
        if self.status() != 200 {
            Err(Error::HttpErrorCode(self.status()))
        } else {
            Ok(self)
        }
    }
}

use roxmltree::{Document, Node};

#[macro_export]
#[doc(hidden)]
macro_rules! find_in_xml {
    ( $node:expr => $( $($var:ident)? $(?$var_opt:ident)? ),+ ) => { {
        let node = $node;
        $(
            $(let mut $var = None;)?
            $(let mut $var_opt = None;)?
        )*
        for child in node.children().filter(Node::is_element) {
            match child.tag_name().name() {
                $(
                    $(stringify!($var) => $var = Some(child),)?
                    $(stringify!($var_opt) => $var_opt = Some(child),)?
                )*
                _ => (),
            }
        }

        $(
            $(let $var = $var.ok_or_else(|| Error::XMLMissingElement(
                    node.tag_name().name().to_string(),
                    stringify!($var).to_string(),
                ))?;)?
        )*

        ($(
            $($var)?
            $($var_opt)?
        ),*)
    } }
}

pub(crate) fn parse_node_text<T, E>(node: Node) -> Result<T, Error>
where
    T: std::str::FromStr<Err = E>,
    E: std::error::Error + Send + Sync + 'static,
{
    node.text()
        .ok_or_else(|| Error::XMLMissingText(node.tag_name().name().to_string()))
        .and_then(|s| s.parse().map_err(|e| Error::InvalidResponse(Box::new(e))))
}

pub(crate) fn find_root<'a, 'input: 'a>(
    document: &'input Document,
    element: &str,
    docname: &str,
) -> Result<Node<'a, 'input>, Error> {
    document
        .descendants()
        .filter(Node::is_element)
        .find(|n| n.tag_name().name().eq_ignore_ascii_case(element))
        .ok_or_else(|| Error::XMLMissingElement(docname.to_string(), element.to_string()))
}
