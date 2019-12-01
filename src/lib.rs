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
//! The following code searches for devices that have a `RenderingControl` service
//! and print their names along with their current volume.
//!
//! ```rust,no_run
//! use futures::prelude::*;
//! use std::time::Duration;
//! use upnp::ssdp::URN;
//!
//! const RENDERING_CONTROL: URN = URN::service("schemas-upnp-org", "RenderingControl", 1);
//!
//! #[async_std::main]
//! async fn main() -> Result<(), upnp::Error> {
//!     let devices = upnp::discover(&RENDERING_CONTROL.into(), Duration::from_secs(3)).await?;
//!     pin_utils::pin_mut!(devices);
//!
//!     while let Some(device) = devices.next().await {
//!         let device = device?;
//!
//!         let service = device
//!             .find_service(&RENDERING_CONTROL)
//!             .expect("searched for RenderingControl, got something else");
//!
//!         let args = "<InstanceID>0</InstanceID><Channel>Master</Channel>";
//!         let response = service.action(device.url(), "GetVolume", args).await?;
//!
//!         let volume = response.get("CurrentVolume").unwrap();
//!
//!         println!("'{}' is at volume {}", device.friendly_name(), volume);
//!     }
//!
//!     Ok(())
//! }
//! ```
// doc include when it gets stable

mod device;
mod discovery;
mod error;
/// Service Control Protocol Description.
pub mod scpd;
mod service;
mod utils;

pub use device::{Device, DeviceSpec};
pub use discovery::discover;
pub use error::Error;
pub use service::Service;

#[doc(no_inline)]
pub use http::Uri;
pub use ssdp_client as ssdp;

pub(crate) type Result<T, E = Error> = std::result::Result<T, E>;
