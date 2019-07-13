#![feature(async_await, await_macro)]
#![recursion_limit = "128"]

use ssdp::search::SearchTarget;
use std::time::Duration;
use upnp::discovery;
use upnp::Device;

#[runtime::main(runtime_tokio::Tokio)]
async fn main() -> Result<(), upnp::Error> {
    //let sonos = SearchTarget::RootDevice;
    let sonos = SearchTarget::URN("schemas-upnp-org:device:ZonePlayer:1".to_string());

    let devices: Vec<Device> = discovery::discover(sonos, Duration::from_secs(1)).await?;
    for device in &devices {
        let spec = device.description();
        println!(
            "{} - {} @ {}",
            spec.device_type(),
            spec.friendly_name(),
            device.ip()
        );
    }
    Ok(())
}
