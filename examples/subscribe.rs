#![feature(async_await, await_macro)]

use futures::prelude::*;
use futures01::{Future, Stream};

use hyper::rt;
use hyper::{service::service_fn_ok, Server};
use hyper::{Body, Request, Response};

fn main() {
    rt::run(subscribe().map_err(|e| eprintln!("{}", e)).boxed().compat());
    rt::run(server().map_err(|e| eprintln!("{}", e)));
}

async fn subscribe() -> Result<(), upnp::Error> {
    let uri: hyper::Uri = "http://192.168.2.49:1400/xml/device_description.xml"
        .parse()
        .unwrap();

    let device = await!(upnp::Device::from_url(uri))?;
    let service = device
        .find_service("schemas-upnp-org:service:AVTransport:1")
        .unwrap();

    await!(service.subscribe(&device.ip(), "http://192.168.2.91:3000"))
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

fn server() -> impl futures01::Future<Item = (), Error = hyper::Error> {
    let addr = ([192, 168, 2, 91], 3000).into();

    Server::bind(&addr).serve(|| service_fn_ok(callback))
}
