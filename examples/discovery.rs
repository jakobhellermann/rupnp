#![feature(async_await, await_macro, futures_api)]

use ssdp::{header::ST, FieldMap};

use upnp::discovery;
use upnp::Device;
use upnp::Error;

use futures::prelude::*;

#[allow(unused_variables)]
fn main() {
    hyper::rt::run(
        async_main()
            .map_err(|e| eprintln!("{}", e))
            .boxed()
            .compat(),
    )
}

async fn async_main() -> Result<(), Error> {
    let sonos = ST::Target(FieldMap::URN(
        "schemas-upnp-org:device:ZonePlayer:1".to_string(),
    ));

    let devices: Vec<Device> = await!(discovery::discover(sonos, 2))?;
    for device in &devices {
        println!("{} - {}", device.device_type(), device.friendly_name());
    }
    Ok(())
}
