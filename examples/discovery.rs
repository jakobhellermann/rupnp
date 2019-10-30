#![feature(generators, proc_macro_hygiene, stmt_expr_attributes)]

use futures_async_stream::for_await;
use std::time::Duration;
use upnp::ssdp::SearchTarget;

fn main() {
    if let Err(e) = async_std::task::block_on(discovery()) {
        eprintln!("{}", e);
    }
}

async fn discovery() -> Result<(), upnp::Error> {
    let devices = upnp::discover(&SearchTarget::RootDevice, Duration::from_secs(3)).await?;

    #[for_await]
    for device in devices {
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
