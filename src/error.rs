use std::fmt;
use xmltree::Element;
use err_derive::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error(display = "{}", _0)]
    UPnPError(#[error(cause)] UPnPError),
    #[error(display = "errored to parse Control Point response")]
    ParseError,
    #[error(display = "err")]
    SerdeError(std::sync::Mutex<serde_xml_rs::Error>),
    #[error(display = "Invalid response: {}", _0)]
    InvalidResponse(Box<dyn std::error::Error + Send + Sync + 'static>),
    #[error(display = "An error occurred trying to connect to device: {}", _0)]
    NetworkError(#[error(cause)] hyper::Error),
    #[error(display = "An error occurred trying to discover devices: {}", _0)]
    Error(#[error(cause)] ssdp_client::Error),
    #[error(display = "Invalid Arguments: {}", _0)]
    InvalidArguments(Box<dyn std::error::Error + Send + Sync + 'static>),
}

impl From<serde_xml_rs::Error> for Error {
    fn from(err: serde_xml_rs::Error) -> Self {
        Error::SerdeError(std::sync::Mutex::new(err))
    }
}

impl From<ssdp_client::Error> for Error {
    fn from(error: ssdp_client::Error) -> Error {
        Error::Error(error)
    }
}

impl From<hyper::Error> for Error {
    fn from(error: hyper::Error) -> Error {
        Error::NetworkError(error)
    }
}

#[derive(Error, Debug)]
pub struct UPnPError {
    fault_code: String,
    fault_string: String,
    err_code: u16,
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
}
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

fn element_to_string(element: &Element) -> Result<String, Error> {
    element.text.to_owned().ok_or(Error::ParseError)
}

pub fn parse(fault: &Element) -> Result<UPnPError, Error> {
    let fault_code = element_to_string(fault.get_child("faultcode").ok_or(Error::ParseError)?)?;
    let fault_string = element_to_string(fault.get_child("faultstring").ok_or(Error::ParseError)?)?;

    let err_code = fault
        .get_child("detail")
        .ok_or(Error::ParseError)?
        .get_child("UPnPError")
        .ok_or(Error::ParseError)?
        .get_child("errorCode")
        .ok_or(Error::ParseError)?;

    if let Some(err_code) = &err_code.text {
        let err_code = err_code.parse().map_err(|_| Error::ParseError)?;
        Ok(UPnPError {
            fault_code,
            fault_string,
            err_code,
        })
    } else {
        Err(Error::ParseError)
    }
}
