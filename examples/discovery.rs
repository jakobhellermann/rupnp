use async_std::prelude::*;
use pin_utils::pin_mut;
use std::time::Duration;

fn main() {
    if let Err(e) = async_std::task::block_on(discovery()) {
        eprintln!("{}", e);
    }
}

async fn discovery() -> Result<(), upnp::Error> {
    // let search_target = "urn:schemas-upnp-org:device:ZonePlayer:1".parse().unwrap();
    let search_target = ssdp_client::SearchTarget::RootDevice;
    let devices = upnp::discover(&search_target, Duration::from_secs(3)).await?;

    pin_mut!(devices);
    while let Some(device) = devices.next().await {
        let device = device?;
        println!(
            "{} - {} @ {}",
            device.device_type(),
            device.friendly_name(),
            device.url()
        );
    }
    Ok(())
}
