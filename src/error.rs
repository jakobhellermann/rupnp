use std::fmt;

#[derive(Debug)]
pub enum Error {
    UPnPError(UPnPError),
    InvalidUrl(isahc::http::uri::InvalidUri),
    InvalidUtf8(std::str::Utf8Error),
    IO(std::io::Error),
    XmlError(roxmltree::Error),
    ParseError(&'static str),
    InvalidResponse(Box<dyn std::error::Error + Send + Sync + 'static>),
    NetworkError(isahc::Error),
    HttpErrorCode(isahc::http::StatusCode),
    SSDPError(ssdp_client::Error),
    XMLMissingElement(String, String),
    XMLMissingText(String),
}
impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Error::UPnPError(err) => write!(f, "{}", err),
            Error::InvalidUrl(err) => write!(f, "invalid url: {}", err),
            Error::InvalidUtf8(err) => write!(f, "invalid utf8: {}", err),
            Error::IO(err) => write!(f, "error reading response: {}", err),
            Error::XmlError(err) => write!(f, "failed to parse xml: {}", err),
            Error::ParseError(err) => write!(f, "{}", err),
            Error::InvalidResponse(err) => write!(f, "Invalid response {}", err),
            Error::NetworkError(err) => write!(f, "An error occurred trying to connect to device: {}", err),
            Error::HttpErrorCode(code) => write!(f, "The control point responded with status code {}", code),
            Error::SSDPError(err) => write!(f, "error trying to discover devices: {}", err),
            Error::XMLMissingElement(parent, child) => write!(f, "`{}` does not contain an `{}` element or attribute", parent, child),
            Error::XMLMissingText(element) => write!(f, "element `{}`'s text is empty", element),
        }
    }
}
impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Error::UPnPError(err) => Some(err),
            Error::InvalidUrl(err) => Some(err),
            Error::XmlError(err) => Some(err),
            //Error::InvalidResponse(err) => Some(err),
            Error::NetworkError(err) => Some(err),
            Error::SSDPError(err) => Some(err),
            _ => None,
        }
    }
}
impl From<isahc::Error> for Error {
    fn from(err: isahc::Error) -> Self {
        Error::NetworkError(err)
    }
}
impl From<roxmltree::Error> for Error {
    fn from(err: roxmltree::Error) -> Self {
        Error::XmlError(err)
    }
}
impl From<std::io::Error> for Error {
    fn from(err: std::io::Error) -> Self {
        Error::IO(err)
    }
}
impl From<isahc::http::uri::InvalidUri> for Error {
    fn from(err: isahc::http::uri::InvalidUri) -> Self {
        Error::InvalidUrl(err)
    }
}
impl From<ssdp_client::Error> for Error {
    fn from(err: ssdp_client::Error) -> Self {
        Error::SSDPError(err)
    }
}
impl From<UPnPError> for Error {
    fn from(err: UPnPError) -> Self {
        Error::UPnPError(err)
    }
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

    pub(crate) fn from_fault_node(node: roxmltree::Node) -> Result<UPnPError, Error> {
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
        
        let fault_code = fault_code
            .ok_or(Error::ParseError("`fault` element contains no `faultcode`"))?
            .to_string();
        let fault_string = fault_string
            .ok_or(Error::ParseError("`fault` element contains no `errCode` or it wasn't an integer"))?
            .to_string();
        let err_code = err_code
            .and_then(|err_code| err_code.parse().ok())
            .ok_or(Error::ParseError("`fault` element contains no `faultcode`"))?;

        Ok(UPnPError {
            fault_code,
            fault_string,
            err_code,
        })
    }
}
