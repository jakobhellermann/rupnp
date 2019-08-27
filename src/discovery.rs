use crate::{Device, Error};
use futures_async_stream::for_await;
use ssdp_client::search::SearchTarget;
use std::time::Duration;

pub async fn discover(
    search_target: SearchTarget<'_>,
    timeout: Duration,
) -> Result<Vec<Device>, Error> {
    let mut devices = Vec::new();

    #[for_await]
    for ip in ssdp_client::search(search_target, timeout, 3).await? {
        let uri: hyper::Uri = ip?
            .location()
            .parse()
            .map_err(|e| Error::InvalidResponse(Box::new(e)))?;

        devices.push(Device::from_url(uri).await?);
    }
    Ok(devices)
}
