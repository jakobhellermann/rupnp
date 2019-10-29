use async_std::prelude::*;
use std::time::Duration;
use upnp::ssdp::URN;

const RENDERING_CONTROL: URN = URN::service("schemas-upnp-org", "RenderingControl", 1);

fn main() {
    if let Err(e) = async_std::task::block_on(discovery()) {
        eprintln!("{}", e);
    }
}

async fn discovery() -> Result<(), upnp::Error> {
    let devices = upnp::discover(&RENDERING_CONTROL.into(), Duration::from_secs(3)).await?;
    pin_utils::pin_mut!(devices);

    while let Some(device) = devices.next().await {
        let device = device?;

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
