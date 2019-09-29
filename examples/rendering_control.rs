use async_std::task;
use isahc::http::Uri;
use upnp::{Device, Error};

macro_rules! map(
    { $($key:expr => $value:expr),+ } => { {
        let mut m = ::std::collections::HashMap::new();
        $(m.insert($key, $value);)+
        m
    }};
);

fn main() {
    let url = "http://192.168.2.29:1400/xml/device_description.xml"
        .parse()
        .unwrap();

    match task::block_on(get_volume(url)) {
        Err(err) => eprintln!("{}", err),
        Ok(volume) => println!("{}", volume),
    }
}

async fn get_volume(url: Uri) -> Result<u16, Error> {
    let service = "urn:schemas-upnp-org:service:RenderingControl:1"
        .parse()
        .unwrap();

    let device = Device::from_url(url).await?;
    let service = device.find_service(&service).unwrap();

    let args = map! { "InstanceID" => "0", "Channel" => "Master" };
    let response = service.action(device.url(), "GetVolume", args).await?;

    response
        .get("CurrentVolume")
        .unwrap()
        .parse()
        .map_err(|err| Error::InvalidResponse(Box::new(err)))
}
