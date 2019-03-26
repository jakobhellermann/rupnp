use upnp::Device;

use futures::Future;
use hyper::rt;

fn main() {
    let uri: hyper::Uri = "http://192.168.2.29:1400/xml/device_description.xml"
        .parse()
        .unwrap();
    let device = upnp::Device::from_url(uri);

    let f = rendering_control(device);

    rt::run(f.map_err(|e| eprintln!("{}", e)));
}

fn rendering_control(
    device: impl Future<Item = Device, Error = failure::Error>,
) -> impl Future<Item = (), Error = failure::Error> {
    device
        .and_then(|device| {
            let service = device
                .find_service("urn:schemas-upnp-org:service:RenderingControl:1")
                .unwrap();
            service.action(
                &device.ip(),
                "GetVolume",
                "<InstanceID>0</InstanceID><Channel>Master</Channel>",
            )
        })
        .map(|response| {
            println!("{}", response);
        })
}
