#![feature(async_await, await_macro)]
#![recursion_limit = "128"]

use ssdp::search::SearchTarget;
use upnp::discovery;
use upnp::Device;
use std::time::Duration;

#[runtime::main(runtime_tokio::Tokio)]
async fn main() -> Result<(), upnp::Error> {
    let sonos = SearchTarget::URN("schemas-upnp-org:device:ZonePlayer:1");

    let devices: Vec<Device> = discovery::discover(sonos, Duration::from_secs(1)).await?;
    for device in &devices {
        println!("{} - {}", device.device_type(), device.friendly_name());
    }
    Ok(())
}
