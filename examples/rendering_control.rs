use upnp::{Device, Error};

macro_rules! map(
    { $($key:expr => $value:expr),+ } => { {
        let mut m = ::std::collections::HashMap::new();
        $(m.insert($key, $value);)+
        m
    }};
);

#[tokio::main]
async fn main() -> Result<(), Error> {
    let uri: hyper::Uri = "http://192.168.2.49:1400/xml/device_description.xml"
        .parse()
        .unwrap();
    let service = "urn:schemas-upnp-org:service:RenderingControl:1"
        .parse()
        .unwrap();

    let device = Device::from_url(uri).await?;
    let service = device.description().find_service(&service).unwrap();

    let args = map! { "InstanceID" => "0", "Channel" => "Master" };
    let response = service
        .action(device.uri().to_owned(), "GetVolume", args)
        .await?;

    let volume = response.get("CurrentVolume").unwrap();

    println!("{}", volume);

    Ok(())
}
