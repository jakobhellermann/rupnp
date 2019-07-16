use crate::device::Device;
use crate::error::Error;
use log::trace;
use std::time::Duration;

use ssdp::search::SearchTarget;

pub async fn discover(
    search_target: SearchTarget,
    timeout: Duration,
) -> Result<Vec<Device>, Error> {
    trace!("start ssdp search");
    let ips = ssdp::search(search_target, timeout).await?;
    trace!("ssdp search finished");

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
