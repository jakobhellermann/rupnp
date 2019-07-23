use crate::{Device, Error};
use ssdp_client::search::SearchTarget;
use std::time::Duration;

pub async fn discover(
    search_target: SearchTarget<'_>,
    timeout: Duration,
) -> Result<Vec<Device>, Error> {
    let ips = ssdp_client::search(search_target, timeout, 3).await?;

    let mut devices = Vec::with_capacity(ips.len());
    for ip in ips {
        let uri: hyper::Uri = ip
            .location()
            .parse()
            .map_err(|_| Error::InvalidResponse(failure::err_msg("invalid location header")))?;
        devices.push(Device::from_url(uri).await?);
    }

    Ok(devices)
}
