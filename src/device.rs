use crate::{find_in_xml, HttpResponseExt};
use crate::{Error, Service};
use isahc::http::Uri;
use roxmltree::{Document, Node};
use ssdp_client::URN;

#[derive(Debug)]
pub struct Device {
    url: Uri,
    device_spec: DeviceSpec,
}
impl Device {
    pub fn url(&self) -> &Uri {
        &self.url
    }

    #[rustfmt::skip]
    pub async fn from_url(url: Uri) -> Result<Self, Error> {
        let body = isahc::get_async(&url)
            .await?
            .err_if_not_200()?
            .body_mut()
            .text_async()
            .await?;

        let document = Document::parse(&body)?;
        let device = crate::find_root(&document, "device", "Device Description")?;
        let device_spec = DeviceSpec::from_xml(device)?;

        Ok(Self { url, device_spec })
    }
}
impl std::ops::Deref for Device {
    type Target = DeviceSpec;

    fn deref(&self) -> &Self::Target {
        &self.device_spec
    }
}

#[derive(Debug)]
pub struct DeviceSpec {
    device_type: URN,
    friendly_name: String,

    devices: Vec<DeviceSpec>,
    services: Vec<Service>,
    /*pub manufacturer: String,
    pub manufacturer_url: Option<String>,
    pub model_description: Option<String>,
    pub model_number: Option<String>,
    pub model_url: Option<String>,
    pub serial_number: Option<String>,
    pub udn: String,
    pub upc: Option<String>,
    //pub icon_list: Value<Vec<Icon>>,
    //pub service_list: Value<Vec<Service>>,
    //pub device_list: Value<Vec<DeviceSpec>>,
    pub presentation_url: Option<String>,*/
}

impl DeviceSpec {
    fn from_xml<'a, 'input: 'a>(node: Node<'a, 'input>) -> Result<Self, Error> {
        #[allow(non_snake_case)]
        let (device_type, friendly_name, services, devices) =
            find_in_xml! { node => deviceType, friendlyName, serviceList, ?deviceList };

        let devices = match devices {
            Some(d) => d
                .children()
                .filter(Node::is_element)
                .map(DeviceSpec::from_xml)
                .collect::<Result<_, _>>()?,
            None => Vec::new(),
        };
        let services = services
            .children()
            .filter(Node::is_element)
            .map(Service::from_xml)
            .collect::<Result<_, _>>()?;

        Ok(Self {
            device_type: crate::parse_node_text(device_type)?,
            friendly_name: crate::parse_node_text(friendly_name)?,
            devices,
            services,
        })
    }

    pub fn device_type(&self) -> &URN {
        &self.device_type
    }
    pub fn friendly_name(&self) -> &str {
        &self.friendly_name
    }

    pub fn devices(&self) -> &Vec<DeviceSpec> {
        &self.devices
    }
    pub fn services(&self) -> &Vec<Service> {
        &self.services
    }

    pub fn services_iter(&self) -> impl Iterator<Item = &Service> {
        self.services().iter().chain(self.devices().iter().flat_map(
            |device| -> Box<dyn Iterator<Item = &Service>> { Box::new(device.services_iter()) },
        ))
    }
    pub fn find_service(&self, service_type: &URN) -> Option<&Service> {
        self.services_iter()
            .find(|s| s.service_type() == service_type)
    }

    pub fn devices_iter(&self) -> impl Iterator<Item = &DeviceSpec> {
        self.devices().iter().chain(self.devices().iter().flat_map(
            |device| -> Box<dyn Iterator<Item = &DeviceSpec>> { Box::new(device.devices_iter()) },
        ))
    }
    pub fn find_device(&self, device_type: &URN) -> Option<&DeviceSpec> {
        self.devices_iter().find(|d| &d.device_type == device_type)
    }

    pub fn print(&self) {
        fn print_inner(device: &DeviceSpec, indentation: usize) {
            let i = "  ".repeat(indentation);

            println!("{}{}", i, &device.device_type);
            for service in device.services() {
                println!("{}  - {}", i, service.service_type());
            }
            for device in device.devices() {
                print_inner(device, indentation + 1);
            }
        }

        print_inner(&self, 0);
    }
}
