use async_std::io;
use async_std::net::TcpListener;
use async_std::prelude::*;

use upnp::ssdp::URN;
use upnp::Device;

fn main() {
    if let Err(e) = async_std::task::block_on(subscribe()) {
        eprintln!("{}", e);
    }
}

async fn subscribe() -> Result<(), upnp::Error> {
    let url = "http://192.168.2.49:1400/xml/device_description.xml"
        .parse()
        .unwrap();
    let urn = URN::service("schemas-upnp-org", "AVTransport", 1);
    let addr = "http://192.168.2.91:3000";

    let device = Device::from_url(url).await?;
    let service = device.find_service(&urn).unwrap();

    let listener = TcpListener::bind(addr.trim_start_matches("http://")).await?;

    service.subscribe(device.url(), addr).await?;

    while let Some(stream) = listener.incoming().next().await {
        io::copy(&mut stream?, &mut io::stdout()).await?;
    }

    Ok(())
}
