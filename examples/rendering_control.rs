use upnp::{http::Uri, ssdp::URN, Device, Error};

fn main() {
    let url = Uri::from_static("http://192.168.2.49:1400/xml/device_description.xml");

    match async_std::task::block_on(get_volume(url)) {
        Err(err) => eprintln!("{}", err),
        Ok(volume) => println!("{}", volume),
    }
}

const RENDERING_CONTROL: URN = URN::service("schemas-upnp-org", "RenderingControl", 1);

async fn get_volume(url: Uri) -> Result<u16, Error> {
    let device = Device::from_url(url).await?;
    let service = device.find_service(&RENDERING_CONTROL).unwrap();

    let args = "<InstanceID>0</InstanceID><Channel>Master</Channel>";
    let response = service.action(device.url(), "GetVolume", args).await?;

    response
        .get("CurrentVolume")
        .unwrap()
        .parse()
        .map_err(Error::invalid_response)
}
