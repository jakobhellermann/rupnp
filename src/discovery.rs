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
    return discover_with_fields(search_target, timeout, &[]).await;
}

/// Discovers UPnP devices on the network and saves extra_fields in device descriptions
///
/// # Example usage:
/// ```rust,no_run
/// use futures::prelude::*;
/// use std::time::Duration;
/// use rupnp::ssdp::SearchTarget;
///
/// # async fn discover_with_fields() -> Result<(), rupnp::Error> {
/// let devices = rupnp::discover_with_fields(&SearchTarget::RootDevice, Duration::from_secs(3), &["manufacturer", "manufacturerURL"]).await?;
/// pin_utils::pin_mut!(devices);
///
/// while let Some(device) = devices.try_next().await? {
///     println!(
///         "{} - {} @ {}",
///         device.device_type(),
///         device.get_extra_element("manufacturer").unwrap_or_default(),
///         device.get_extra_element("manufacturerURL").unwrap_or_default()
///     );
/// }
///
/// # Ok(())
/// # }
/// ```
pub async fn discover_with_fields<'a>(
    search_target: &SearchTarget,
    timeout: Duration,
    extra_fields: &'a [&'a str],
) -> Result<impl Stream<Item = Result<Device>> + 'a> {
    Ok(ssdp_client::search(search_target, timeout, 3, None)
        .await?
        .map_err(Error::SSDPError)
        .map(|res| Ok(res?.location().parse()?))
        .and_then(move |url| Device::from_url_and_fields(url, extra_fields)))
}
