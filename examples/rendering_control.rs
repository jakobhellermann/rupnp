#![feature(async_await)]

use upnp::{Device, Error};

#[hyper::rt::main]
async fn main() -> Result<(), Error> {
    let uri: hyper::Uri = "http://192.168.2.49:1400/xml/device_description.xml"
        .parse()
        .unwrap();
    let service = "urn:schemas-upnp-org:service:RenderingControl:1"
        .parse()
        .unwrap();

    let device = Device::from_url(uri).await?;
    let spec = device.description();

    let service = spec.find_service(&service).unwrap();

    let mut response = service
        .action(
            device.uri().to_owned(),
            "GetVolume",
            "<InstanceID>0</InstanceID><Channel>Master</Channel>",
        )
        .await?;

    let volume = response
        .take_child("CurrentVolume")
        .unwrap()
        .text
        .ok_or(Error::ParseError)?;

    let volume: u8 = volume
        .parse()
        .map_err(|e| Error::InvalidResponse(failure::Error::from(e)))?;

    println!("{}", volume);

    Ok(())
}
