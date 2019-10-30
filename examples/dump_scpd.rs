use upnp::http::Uri;
use upnp::{Device, DeviceSpec, Error};

fn main() {
    let url = Uri::from_static("http://192.168.2.49:1400/xml/device_description.xml");

    if let Err(e) = async_std::task::block_on(dump_scpd(url)) {
        eprintln!("{}", e);
    }
}

async fn dump_scpd(url: Uri) -> Result<(), Error> {
    let device = Device::from_url(url).await?;
    print(&device, device.url(), 0)
}

fn print(spec: &DeviceSpec, url: &Uri, indent_lvl: usize) -> Result<(), Error> {
    let space = "  ".repeat(indent_lvl);

    println!("{} {}", space, spec.device_type());

    for service in spec.services() {
        println!("{} - {}", space, service.service_type());

        let scpd = async_std::task::block_on(service.scpd(&url))?;

        let space = "  ".repeat(indent_lvl + 2);
        for state_var in scpd.state_variables() {
            println!("{}- SV: {}", space, state_var);
        }
        for action in scpd.actions() {
            println!("{}- AC: {}", space, action);
        }
    }

    for device in spec.devices() {
        print(device, url, indent_lvl + 1)?;
    }

    Ok(())
}
