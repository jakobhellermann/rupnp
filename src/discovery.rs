use crate::device::Device;
use crate::error::Error;
use futures::{future, Future};
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

pub fn discover(search_target: ST, timeout: u8) -> impl Future<Item = Vec<Device>, Error = Error> {
    let ips = match discover_ips(search_target, timeout) {
        Ok(item) => future::ok(item),
        Err(err) => future::err(err),
    };

    ips.map(|devices| {
        devices
            .into_iter()
            .map(|device| Device::from_url(device).map_err(Error::NetworkError))
    })
    .and_then(future::join_all)
}
