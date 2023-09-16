use crate::{Device, Error, Result};
use futures_util::stream::{Stream, StreamExt, TryStreamExt};
use ssdp_client::SearchTarget;
use std::time::Duration;

/// Discovers UPnP devices on the network.
///
/// # Example usage:
/// ```rust,no_run
/// use futures::prelude::*;
/// use std::time::Duration;
/// use rupnp::ssdp::SearchTarget;
///
/// # async fn discover() -> Result<(), rupnp::Error> {
/// let devices = rupnp::discover(&SearchTarget::RootDevice, Duration::from_secs(3)).await?;
/// pin_utils::pin_mut!(devices);
///
/// while let Some(device) = devices.try_next().await? {
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
    Ok(ssdp_client::search(search_target, timeout, 3, None)
        .await?
        .map_err(Error::SSDPError)
        .map(|res| Ok(res?.location().parse()?))
        .and_then(Device::from_url))
}
