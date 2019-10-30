#![feature(external_doc, generators, proc_macro_hygiene)]
#![warn(
    nonstandard_style,
    rust_2018_idioms,
    future_incompatible,
    missing_debug_implementations
)]

//! An asynchronous library for finding UPnP control points, performing actions on them
//! and reading their service descriptions.
//! UPnP stand for `Universal Plug and Play` and is widely used for routers, WiFi-enabled speakers
//! and media servers.
//!
//! # Example usage:
//! ```rust,no_run
#![doc(include = "../examples/search_and_action.rs")]
//! ```

mod device;
mod discovery;
mod error;

/// Service Control Protocol Description.
pub mod scpd;
mod service;

pub use device::{Device, DeviceSpec};
pub use discovery::discover;
pub use error::Error;
pub use service::Service;

pub use http;
pub use ssdp_client as ssdp;

pub(crate) type Result<T> = std::result::Result<T, Error>;

trait HttpResponseExt: Sized {
    fn err_if_not_200(self) -> Result<Self>;
}
impl HttpResponseExt for crate::http::Response<isahc::Body> {
    fn err_if_not_200(self) -> Result<Self> {
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

pub(crate) fn parse_node_text<T, E>(node: Node<'_, '_>) -> Result<T>
where
    T: std::str::FromStr<Err = E>,
    E: std::error::Error + Send + Sync + 'static,
{
    node.text()
        .unwrap_or_default()
        .parse()
        .map_err(Error::invalid_response)
}

pub(crate) fn find_root<'a, 'input: 'a>(
    document: &'input Document<'_>,
    element: &str,
    docname: &str,
) -> Result<Node<'a, 'input>> {
    document
        .descendants()
        .filter(Node::is_element)
        .find(|n| n.tag_name().name().eq_ignore_ascii_case(element))
        .ok_or_else(|| Error::XMLMissingElement(docname.to_string(), element.to_string()))
}
