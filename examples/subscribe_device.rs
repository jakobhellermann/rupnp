use async_std::{
    io,
    net::{TcpListener, TcpStream},
    task,
};
use futures::prelude::*;
use upnp::Device;

fn main() {
    if let Err(e) = task::block_on(subscribe()) {
        eprintln!("{}", e);
    }
}

async fn subscribe() -> Result<(), upnp::Error> {
    let addr: std::net::SocketAddr = ([192, 168, 2, 91], 3000).into();
    let addr_str = format!("http://{}", addr);

    let url = "http://192.168.2.49:1400/xml/device_description.xml"
        .parse()
        .unwrap();
    let service = "urn:schemas-upnp-org:service:AVTransport:1"
        .parse()
        .unwrap();

    let device = Device::from_url(url).await?;
    let service = device.find_service(&service).unwrap();
    service.subscribe(device.url(), &addr_str).await?;

    let listener = TcpListener::bind(addr)
        .await?;
    println!("Listening on {}", listener.local_addr().unwrap());

    let mut incoming = listener.incoming();
    while let Some(stream) = incoming.next().await {
        let stream = stream?;
        task::spawn(async {
            process(stream).await.unwrap();
        });
    }

    Ok(())
}

async fn process(stream: TcpStream) -> io::Result<()> {
    println!("Accepted from: {}", stream.peer_addr()?);

    let (reader, _) = &mut (&stream, &stream);

    io::copy(reader, &mut io::stdout()).await?;

    Ok(())
}
