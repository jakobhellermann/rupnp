#![feature(async_await, await_macro, futures_api)]

use futures::prelude::*;

use upnp::{Device, Error};

#[allow(unused_variables)]
fn main() {
    hyper::rt::run(
        async_main()
            .map_ok(|v| println!("{:?}", v))
            .map_err(|e| eprintln!("{}", e))
            .boxed()
            .compat(),
    )
}

async fn async_main() -> Result<u8, Error> {
    let uri: hyper::Uri = "http://192.168.2.49:1400/xml/device_description.xml"
        .parse()
        .unwrap();

    let device = await!(Device::from_url(uri).map_err(Error::NetworkError))?;

    let service = device
        .find_service("schemas-upnp-org:service:RenderingControl:1")
        .unwrap();

    let mut response = await!(service.action(
        &device.ip(),
        "GetVolume",
        "<InstanceID>0</InstanceID><Channel>Master</Channel>",
    ))?;

    let volume = response
        .take_child("CurrentVolume")
        .unwrap()
        .text
        .ok_or(Error::ParseError)?;
    volume
        .parse()
        .map_err(|e| Error::InvalidResponse(failure::Error::from(e)))
}
