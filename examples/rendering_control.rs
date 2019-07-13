#![feature(async_await, await_macro)]

use upnp::{Device, Error};

/*




*/

#[runtime::main(runtime_tokio::Tokio)]
async fn main() -> Result<(), Error> {
    let uri: hyper::Uri = "http://192.168.2.49:1400/xml/device_description.xml"
        .parse()
        .unwrap();

    let device = Device::from_url(uri).await?;
    let spec = device.description();

    let service = spec
        .find_service("schemas-upnp-org:service:RenderingControl:1")
        .unwrap();

    let mut response = service
        .action(
            device.ip().to_owned(),
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
