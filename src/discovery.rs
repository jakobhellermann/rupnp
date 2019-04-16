use crate::device::Device;
use crate::error::Error;

use ssdp::header::{HeaderMut, HeaderRef, Location, Man, MX, ST};
use ssdp::message::{Multicast, SearchRequest};

pub fn discover_ips(search_target: ST, timeout: u8) -> Result<Vec<hyper::Uri>, Error> {
    let mut request = SearchRequest::new();
    request.set(Man);
    request.set(MX(timeout));
    request.set(search_target);

    let mut responses = Vec::new();
    for (msg, _src) in request.multicast()? {
        let location: &Location = msg.get().ok_or(Error::ParseError)?;

        responses.push(location.parse().map_err(|_| Error::ParseError)?);
    }

    Ok(responses)
}

pub async fn discover(search_target: ST, timeout: u8) -> Result<Vec<Device>, Error> {
    let ips = discover_ips(search_target, timeout)?;

    let mut devices = Vec::with_capacity(ips.len());
    for ip in ips {
        devices.push(await!(Device::from_url(ip)).map_err(Error::NetworkError)?);
    }

    Ok(devices)
}
