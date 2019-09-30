use err_derive::Error;
use std::fmt;

#[derive(Error, Debug)]
pub enum Error {
    #[error(display = "{}", _0)]
    UPnPError(#[error(cause)] UPnPError),
    #[error(display = "invalid url: {}", _0)]
    InvalidUrl(#[cause] isahc::http::uri::InvalidUri),
    #[error(display = "invalid utf8: {}", _0)]
    InvalidUtf8(#[error(cause)] std::str::Utf8Error),
    #[error(display = "error reading response: {}", _0)]
    IO(#[error(cause)] std::io::Error),
    #[error(display = "failed to parse xml: {}", _0)]
    XmlError(#[cause] roxmltree::Error),
    #[error(display = "failed to parse Control Point response")]
    ParseError(&'static str),
    #[error(display = "Invalid response: {}", _0)]
    InvalidResponse(Box<dyn std::error::Error + Send + Sync + 'static>),
    #[error(display = "An error occurred trying to connect to device: {}", _0)]
    NetworkError(#[error(cause)] isahc::Error),
    #[error(display = "The control point responded with status code: {}", _0)]
    HttpErrorCode(isahc::http::StatusCode),
    #[error(display = "An error occurred trying to discover devices: {}", _0)]
    SSDPError(#[error(cause)] ssdp_client::Error),
    #[error(display = "`{}` contains no `{}` element ", _0, _1)]
    XMLMissingElement(String, String),
    #[error(display = "element `{}`'s text is empty", _0)]
    XMLMissingText(String),
}

#[derive(Debug)]
pub struct UPnPError {
    fault_code: String,
    fault_string: String,
    err_code: u16,
}

impl std::error::Error for UPnPError {}
impl fmt::Display for UPnPError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{} {}: {}",
            self.fault_string,
            self.err_code,
            self.err_code_description()
        )
    }
}

impl UPnPError {
    pub fn err_code_description(&self) -> &str {
        match self.err_code {
            401 => "No action by that name at this service.",
            402 => "Invalid Arguments",
            403 => "(deprecated error code)",
            501 => "Action failed",
            600 => "Argument value invalid",
            601 => "Argument Value Out of Range",
            602 => "Optional Action Not Implemented",
            603 => "Out of Memory",
            604 => "Human Intervention Required",
            605 => "String Argument Too Long",
            606..=612 => "(error code reserved for UPnP DeviceSecurity)",
            613..=699 => "Common action error. Defined by UPnP Forum Technical Committee.",
            700..=799 => "Action-specific error defined by UPnP Forum working committee.",
            800..=899 => "Action-specific error for non-standard actions. Defined by UPnP vendor.",
            _ => "Invalid Error Code",
        }
    }

    pub(crate) fn from_fault_node(node: roxmltree::Node) -> Error {
        let mut fault_code = None;
        let mut fault_string = None;
        let mut err_code = None;

        for child in node.descendants() {
            match child.tag_name().name() {
                "faultcode" => fault_code = child.text(),
                "faultstring" => fault_string = child.text(),
                "errorCode" => err_code = child.text(),
                _ => (),
            }
        }

        if let Some(fault_code) = fault_code {
            if let Some(err_code) = err_code.and_then(|x| x.parse::<u16>().ok()) {
                if let Some(fault_string) = fault_string {
                    Error::UPnPError(UPnPError {
                        fault_code: fault_code.to_string(),
                        fault_string: fault_string.to_string(),
                        err_code,
                    })
                } else {
                    Error::ParseError("`fault` element contains no `faulcode`")
                }
            } else {
                Error::ParseError("`fault` element contains no `errCode` or it was malformed")
            }
        } else {
            Error::ParseError("`fault` element contains no `faultcode`")
        }
    }
}
