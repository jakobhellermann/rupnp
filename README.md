# upnp
An asynchronous library for finding UPnP control points, performing actions on them
and reading their service descriptions.
UPnP stand for `Universal Plug and Play` and is widely used for routers, WiFi-enabled speakers
and media servers.

Spec:
[http://upnp.org/specs/arch/UPnP-arch-DeviceArchitecture-v2.0.pdf](http://upnp.org/specs/arch/UPnP-arch-DeviceArchitecture-v2.0.pdf)

# Example usage:
The following code searches for devices that have a `RenderingControl` service 
and print their names along with their current volume.
```rust,no_run
use futures::prelude::*;
use std::time::Duration;
use upnp::ssdp::URN;

const RENDERING_CONTROL: URN = URN::service("schemas-upnp-org", "RenderingControl", 1);

#[async_std::main]
async fn main() -> Result<(), upnp::Error> {
    let devices = upnp::discover(&RENDERING_CONTROL.into(), Duration::from_secs(3)).await?;
    pin_utils::pin_mut!(devices);

    while let Some(device) = devices.next().await {
        let device = device?;

        let service = device
            .find_service(&RENDERING_CONTROL)
            .expect("searched for RenderingControl, got something else");

        let args = "<InstanceID>0</InstanceID><Channel>Master</Channel>";
        let response = service.action(device.url(), "GetVolume", args).await?;

        let volume = response.get("CurrentVolume").unwrap();

        println!("'{}' is at volume {}", device.friendly_name(), volume);
    }

    Ok(())
}
```

License
-------

Licensed under either of

 * Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
 * MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

Contribution
------------

Please use [rustfmt](https://github.com/rust-lang/rustfmt) before any pull requests.

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall be dual licensed as above, without any
additional terms or conditions.
