use ssdp::{header::ST, FieldMap};
use upnp::discovery;

use futures::Future;
use hyper::rt;

#[allow(unused_variables)]
fn main() {
    let sonos = ST::Target(FieldMap::URN(
        "schemas-upnp-org:device:ZonePlayer:1".to_string(),
    ));
    let media_renderer = ST::Target(FieldMap::URN(
        "schemas-upnp-org:device:MediaRenderer:1".to_string(),
    ));

    let f = discovery::discover(media_renderer, 2).map(|devices| {
        for device in &devices {
            println!("{} - {}", device.device_type(), device.friendly_name());
        }
    });

    rt::run(f.map_err(|e| eprintln!("{}", e)));
}
