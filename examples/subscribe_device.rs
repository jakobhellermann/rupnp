#![feature(async_await)]

use futures::prelude::*;
use hyper::service::{make_service_fn, service_fn};
use hyper::{Body, Request, Response, Server};
use upnp::Device;

#[hyper::rt::main]
async fn main() -> Result<(), upnp::Error> {
    let addr: std::net::SocketAddr = ([192, 168, 2, 91], 3000).into();
    let addr_str = format!("http://{}", addr);

    let uri: hyper::Uri = "http://192.168.2.49:1400/xml/device_description.xml"
        .parse()
        .unwrap();
    let device = Device::from_url(uri).await?;
    let service = device
        .description()
        .find_service("schemas-upnp-org:service:AVTransport:1")
        .unwrap();
    service.subscribe(device.ip().to_owned(), &addr_str).await?;

    Server::bind(&addr)
        .serve(make_service_fn(|_| {
            async { Ok::<_, hyper::Error>(service_fn(callback)) }
        }))
        .map_err(upnp::Error::NetworkError)
        .await?;

    Ok(())
}

async fn callback(req: Request<Body>) -> Result<Response<Body>, hyper::Error> {
    let body = req.into_body().try_concat().await?;
    let body = String::from_utf8_lossy(body.as_ref());
    println!("{}", body);

    Ok(Response::default())
}
