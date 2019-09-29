use crate::service::Service;
use crate::shared::{SpecVersion, Value};
use crate::Error;
use crate::HttpResponseExt;
use isahc::http::Uri;
use serde::Deserialize;
use ssdp_client::search::URN;

#[derive(Debug)]
pub struct Device {
    url: Uri,
    device_spec: DeviceSpec,
}
impl Device {
    pub fn url(&self) -> &Uri {
        &self.url
    }

    pub async fn from_url(url: Uri) -> Result<Self, Error> {
        let body = isahc::get_async(&url)
            .await?
            .err_if_not_200()?
            .body_mut()
            .text_async()
            .await?;

        let device_description: DeviceDescription = serde_xml_rs::from_reader(body.as_bytes())?;

        assert!(
            device_description.spec_version.major() == 1,
            "can only parse spec version 1.x"
        );

        Ok(Device {
            url,
            device_spec: device_description.device,
        })
    }
}

impl std::ops::Deref for Device {
    type Target = DeviceSpec;

    fn deref(&self) -> &Self::Target {
        &self.device_spec
    }
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
struct DeviceDescription {
    spec_version: SpecVersion,
    device: DeviceSpec,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct DeviceSpec {
    #[serde(deserialize_with = "crate::shared::deserialize_urn")]
    pub device_type: URN<'static>,
    pub friendly_name: String,
    pub manufacturer: String,
    #[serde(rename = "manufacturerURL")]
    pub manufacturer_url: Option<String>,
    pub model_description: Option<String>,
    pub model_number: Option<String>,
    #[serde(rename = "modelURL")]
    pub model_url: Option<String>,
    pub serial_number: Option<String>,
    #[serde(rename = "UDN")]
    pub udn: String,
    #[serde(rename = "UPC")]
    pub upc: Option<String>,
    #[serde(default = "Default::default")]
    pub icon_list: Value<Vec<Icon>>,
    #[serde(default = "Default::default")]
    pub service_list: Value<Vec<Service>>,
    #[serde(default = "Default::default")]
    pub device_list: Value<Vec<DeviceSpec>>,
    #[serde(rename = "presentationURL")]
    pub presentation_url: Option<String>,
}

impl DeviceSpec {
    pub fn services(&self) -> &Vec<Service> {
        &self.service_list.value
    }
    pub fn devices(&self) -> &Vec<DeviceSpec> {
        &self.device_list.value
    }
    pub fn icons(&self) -> &Vec<Icon> {
        &self.icon_list.value
    }
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Icon {
    pub mimetype: String,
    pub width: u32,
    pub height: u32,
    pub depth: u32,
    pub url: String,
}

impl DeviceSpec {
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
