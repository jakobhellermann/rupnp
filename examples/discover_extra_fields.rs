use futures::prelude::*;
use rupnp::ssdp::SearchTarget;
use std::time::Duration;

const EXTRA: &[&str; 2] = &["manufacturer", "manufacturerURL"];

#[tokio::main]
async fn main() -> Result<(), rupnp::Error> {
    let devices =
        rupnp::discover_with_fields(&SearchTarget::RootDevice, Duration::from_secs(3), EXTRA)
            .await?;
    pin_utils::pin_mut!(devices);

    while let Some(maybe_device) = devices.next().await {
        match maybe_device {
            Ok(device) => println!(
                "{} from {} @ {}",
                device.friendly_name(),
                device.get_extra_element(EXTRA[0]).unwrap_or_default(),
                device.get_extra_element(EXTRA[1]).unwrap_or_default()
            ),
            _ => (),
        };
    }

    Ok(())
}
