use ssdp::header::{HeaderMut, HeaderRef, Location, Man, MX, ST};
use ssdp::message::{Multicast, SearchRequest};

use futures::{future, Future};
use crate::device::Device;

pub fn discover_ips(search_target: ST, timeout: u8) -> Result<Vec<hyper::Uri>, failure::Error> {
    let mut request = SearchRequest::new();
    request.set(Man);
    request.set(MX(timeout));
    request.set(search_target);

    let mut responses = Vec::new();
    for (msg, _src) in request
        .multicast()
        .map_err(|e| failure::err_msg(format!("error sending multicast: {}", e)))?
    {
        let location: &Location = msg.get().ok_or(failure::err_msg(
            "UPnP Response does not contain LOCATION header",
        ))?;

        responses.push(location.parse()?);
    }

    Ok(responses)
}

pub fn discover(search_target: ST, timeout: u8) -> Result<impl Future<Item = Vec<Device>, Error = failure::Error>, failure::Error> {
    Ok(future::join_all(
        discover_ips(search_target, timeout)?
            .into_iter()
            .map(Device::from_url)
    ))
}