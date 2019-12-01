use crate::Device;
use crate::{Error, Result};
use futures_util::stream::{Stream, StreamExt, TryStreamExt};
use ssdp_client::SearchTarget;
use std::time::Duration;

/// Discover UPnP devices on the network.
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
