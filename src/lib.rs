#![feature(generators, stmt_expr_attributes, proc_macro_hygiene)]

pub mod device;
mod discovery;
pub mod error;
pub mod scpd;
pub mod service;
mod shared;

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
