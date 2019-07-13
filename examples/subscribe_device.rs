#![feature(async_await, await_macro)]
#![recursion_limit = "128"]

use futures::compat::Future01CompatExt;
use futures01::{Future, Stream};
use hyper::rt;
use hyper::{service::service_fn_ok, Server};
use hyper::{Body, Request, Response};
use upnp::Device;

#[runtime::main(runtime_tokio::Tokio)]
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
        .serve(|| service_fn_ok(callback))
        .map_err(upnp::Error::NetworkError)
        .compat()
        .await?;

    Ok(())
}

fn callback(req: Request<Body>) -> Response<Body> {
    rt::spawn(
        req.into_body()
            .concat2()
            .map(|body| {
                let body = String::from_utf8_lossy(&body);
                println!("{}", body);
            })
            .map_err(|e| eprintln!("{}", e)),
    );

    Response::default()
}
