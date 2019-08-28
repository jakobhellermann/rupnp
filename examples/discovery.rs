#![feature(stmt_expr_attributes, proc_macro_hygiene)]

use futures_async_stream::for_await;
#[allow(unused_imports)]
use ssdp_client::search::SearchTarget;
use std::time::Duration;

#[tokio::main]
async fn main() -> Result<(), upnp::Error> {
    //let search_target = SearchTarget::RootDevice;
    let search_target = "urn:schemas-upnp-org:device:ZonePlayer:1".parse().unwrap();

    #[for_await]
    for device in upnp::discover(search_target, Duration::from_secs(1)).await? {
        let device = device?;
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
