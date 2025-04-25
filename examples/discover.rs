use futures::prelude::*;
use rupnp::ssdp::SearchTarget;
use std::time::Duration;

#[tokio::main]
async fn main() -> Result<(), rupnp::Error> {
    let devices = rupnp::discover(&SearchTarget::RootDevice, Duration::from_secs(3), None).await?;
    pin_utils::pin_mut!(devices);

    while let Some(device) = devices.try_next().await? {
        println!(
            "{} - {} @ {}",
            device.device_type(),
            device.friendly_name(),
            device.url()
        );
    }

    Ok(())
}
