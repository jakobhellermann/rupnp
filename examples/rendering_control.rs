use futures::Future;
use hyper::rt;
use upnp::Error;

fn main() {
    let uri: hyper::Uri = "http://192.168.2.49:1400/xml/device_description.xml"
        .parse()
        .unwrap();

    let f = upnp::Device::from_url(uri)
        .map_err(Error::NetworkError)
        .and_then(|device| {
            let service = device
                .find_service("schemas-upnp-org:service:RenderingControl:1")
                .unwrap();

            service.action(
                &device.ip(),
                "GetVolume",
                "<InstanceID>0</InstanceID><Channel>Master</Channel>",
            )
        })
        .map(|response| {
            println!("{:?}", response.get_child("CurrentVolume").unwrap().text);
        });

    rt::run(f.map_err(|e| eprintln!("{}", e)));
}
