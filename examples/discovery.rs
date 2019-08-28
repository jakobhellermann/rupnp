#[allow(unused_imports)]
use ssdp_client::search::SearchTarget;
use std::time::Duration;
use upnp::Device;

#[tokio::main]
async fn main() -> Result<(), upnp::Error> {
    //let search_target = SearchTarget::RootDevice;
    let search_target = "urn:schemas-upnp-org:device:ZonePlayer:1".parse().unwrap();

    let devices: Vec<Device> = upnp::discover(search_target, Duration::from_secs(1)).await?;
    for device in &devices {
        let spec = device.description();
        println!(
            "{} - {} @ {}",
            spec.device_type(),
            spec.friendly_name(),
            device.uri()
        );
    }
    Ok(())
}
