use upnp::Device;

use futures::Future;
use hyper::rt;

fn main() {
    let uri: hyper::Uri = "http://192.168.2.29:1400/xml/device_description.xml"
        .parse()
        .unwrap();
    let device = upnp::Device::from_url(uri);

    let f = visit_services(device);

    rt::run(f.map_err(|e| eprintln!("{}", e)));
}

#[allow(unused)]
fn visit_devices(
    device: impl Future<Item = Device, Error = failure::Error>,
) -> impl Future<Item = (), Error = failure::Error> {
    device
        .map(|device| {
            device.visit_devices(|d| {
                println!("{}:\t{}", d.device_type(), d.friendly_name());
                None
            })
        })
        .map(|_: Option<()>| ())
}

#[allow(unused)]
fn visit_services(
    device: impl Future<Item = Device, Error = failure::Error>,
) -> impl Future<Item = (), Error = failure::Error> {
    device
        .map(|device| {
            device.visit_services(|s| {
                println!("{}", s.service_type());
                None
            })
        })
        .map(|_: Option<()>| ())
}
