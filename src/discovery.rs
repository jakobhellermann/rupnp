use crate::{Device, Error, Result};
use futures_util::stream::{Stream, StreamExt, TryStreamExt};
use ssdp_client::SearchTarget;
use std::time::Duration;

/// Discovers UPnP devices on the network.
///
/// # Example usage:
/// ```rust,norun
/// use futures::prelude::*;
/// use std::time::Duration;
/// use upnp::ssdp::SearchTarget;
///
/// # async fn discover() -> Result<(), upnp::Error> {
/// let devices = upnp::discover(&SearchTarget::RootDevice, Duration::from_secs(3)).await?;
/// pin_utils::pin_mut!(devices);
///
/// while let Some(device) = devices.next().await {
///     let device = device?;
///     println!(
///         "{} - {} @ {}",
///         device.device_type(),
///         device.friendly_name(),
///         device.url()
///     );
/// }
///
/// # Ok(())
/// # }
/// ```
// TODO: doc include once stable
pub async fn discover(
    search_target: &SearchTarget,
    timeout: Duration,
) -> Result<impl Stream<Item = Result<Device>>> {
    Ok(ssdp_client::search(search_target, timeout, 3)
        .await?
        .map(|res| match res {
            Ok(search_response) => Ok(search_response.location().to_string().parse()?),
            Err(e) => Err(Error::SSDPError(e)),
        })
        .and_then(Device::from_url))
}
