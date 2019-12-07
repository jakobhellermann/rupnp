use futures::prelude::*;
use std::time::Duration;
use upnp::ssdp::{SearchTarget, URN};

const RENDERING_CONTROL: URN = URN::service("schemas-upnp-org", "RenderingControl", 1);

#[async_std::main]
async fn main() -> Result<(), upnp::Error> {
    let search_target = SearchTarget::URN(RENDERING_CONTROL);
    let devices = upnp::discover(&search_target, Duration::from_secs(3)).await?;
    pin_utils::pin_mut!(devices);

    while let Some(device) = devices.try_next().await? {
        let service = device
            .find_service(&RENDERING_CONTROL)
            .expect("searched for RenderingControl, got something else");

        let args = "<InstanceID>0</InstanceID><Channel>Master</Channel>";
        let response = service.action(device.url(), "GetVolume", args).await?;

        let volume = response.get("CurrentVolume").unwrap();

        println!("'{}' is at volume {}", device.friendly_name(), volume);
    }

    Ok(())
}
