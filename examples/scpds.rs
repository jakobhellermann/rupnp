use upnp::{Device, SCPD};

use futures::Future;
use hyper::rt;

fn main() {
    let uri: hyper::Uri = "http://192.168.2.29:1400/xml/device_description.xml"
        .parse()
        .unwrap();
    let device = upnp::Device::from_url(uri);

    let f = parse_scpds(device);

    rt::run(f.map_err(|e| eprintln!("{}", e)));
}

fn parse_scpds(
    device: impl Future<Item = Device, Error = failure::Error>,
) -> impl Future<Item = (), Error = failure::Error> {
    device
        .map(|device| {
            let scpd_futures: Vec<_> = device
                .services()
                .iter()
                .map(|service| {
                    let ip: hyper::Uri = format!("{}{}", device.ip(), service.scpd_url())
                        .parse()
                        .unwrap();
                    println!("{}", ip);
                    SCPD::from_url(ip, service.service_type().to_string())
                })
                .collect();
            scpd_futures
        })
        .and_then(futures::future::join_all)
        .map(|scpds| {
            for scpd in &scpds {
                for _action in scpd.actions() {}
            }
        })
}
