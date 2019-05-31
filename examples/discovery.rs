#![feature(async_await, await_macro)]
#![recursion_limit = "128"]

use ssdp::{header::ST, FieldMap};

use upnp::discovery;
use upnp::Device;

#[runtime::main(runtime_tokio::Tokio)]
async fn main() -> Result<(), upnp::Error> {
    let sonos = ST::Target(FieldMap::URN(
        "schemas-upnp-org:device:ZonePlayer:1".to_string(),
    ));

    let devices: Vec<Device> = discovery::discover(sonos, 2).await?;
    for device in &devices {
        println!("{} - {}", device.device_type(), device.friendly_name());
    }
    Ok(())
}
