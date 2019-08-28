#![feature(generators, stmt_expr_attributes, proc_macro_hygiene)]

pub mod device;
mod discovery;
pub mod error;
pub mod scpd;
pub mod service;
mod shared;

pub use device::Device;
pub use error::Error;
pub use scpd::SCPD;
pub use service::Service;
pub use scpd::datatypes::Bool;

pub use discovery::discover;
pub use ssdp_client;
