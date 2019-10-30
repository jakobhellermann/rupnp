#![feature(generators, proc_macro_hygiene, stmt_expr_attributes)]

use futures_async_stream::for_await;
use std::collections::HashMap;
use upnp::http::Uri;
use upnp::ssdp::URN;
use upnp::Device;

fn main() {
    if let Err(e) = async_std::task::block_on(subscribe()) {
        eprintln!("{}", e);
    }
}

async fn subscribe() -> Result<(), upnp::Error> {
    let url = Uri::from_static("http://192.168.2.49:1400/xml/device_description.xml");
    let urn = URN::service("schemas-upnp-org", "ZoneGroupTopology", 1);

    let device = Device::from_url(url).await?;
    let service = device.find_service(&urn).unwrap();

    let (sid, stream) = service.subscribe(device.url(), 10).await?;

    #[for_await]
    for state_vars in stream {
        handle(state_vars?);

        service.renew_subscription(device.url(), &sid, 10).await?;
    }

    Ok(())
}

fn handle(state_vars: HashMap<String, String>) {
    println!("Change {{");
    for (key, value) in state_vars {
        if value.len() > 64 {
            println!("  {}: ...", key);
        } else {
            println!("  {}: {}", key, value);
        }
    }
    println!("}}");
}
