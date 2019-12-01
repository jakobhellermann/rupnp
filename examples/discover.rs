use futures::prelude::*;
use std::time::Duration;
use upnp::ssdp::SearchTarget;

#[async_std::main]
async fn main() -> Result<(), upnp::Error> {
    let devices = upnp::discover(&SearchTarget::RootDevice, Duration::from_secs(3)).await?;
    pin_utils::pin_mut!(devices);

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
