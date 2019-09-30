#![feature(stmt_expr_attributes, proc_macro_hygiene)]

use futures_async_stream::for_await;
use std::time::Duration;

fn main() {
    if let Err(e) = async_std::task::block_on(discovery()) {
        eprintln!("{}", e);
    }
}

async fn discovery() -> Result<(), upnp::Error> {
    let search_target = "urn:schemas-upnp-org:device:ZonePlayer:1".parse().unwrap();

    #[for_await]
    for device in upnp::discover(search_target, Duration::from_secs(1)).await? {
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
