use crate::service:: Service;
use crate::shared::{SpecVersion, Value};
use log::trace;
use crate::Error;
use futures::prelude::*;
use getset::Getters;
use serde::Deserialize;

#[derive(Debug)]
pub struct Device {
    uri: hyper::Uri,
    device_spec: DeviceSpec,
}

impl Device {
    pub fn uri(&self) -> &hyper::Uri {
        &self.uri
    }
    pub fn description(&self) -> &DeviceSpec {
        &self.device_spec
    }

    pub async fn from_url(uri: hyper::Uri) -> Result<Self, Error> {
        trace!("begin Device::from_url");
        let client = hyper::Client::new();

        let res = client.get(uri.clone()).await?;
        let body = res.into_body().try_concat().await?;
        trace!("fetched device description");

        let device_description: DeviceDescription = serde_xml_rs::from_reader(&body[..])?;
        trace!("parsed device description");

        assert!(
            device_description.spec_version.major() == 1,
            "can only parse spec version 1.x"
        );

        Ok(Device {
            uri,
            device_spec: device_description.device,
        })
    }
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
struct DeviceDescription {
    spec_version: SpecVersion,
    device: DeviceSpec,
}

#[derive(Deserialize, Debug, Getters)]
#[serde(rename_all = "camelCase")]
pub struct DeviceSpec {
    #[get = "pub"]
    device_type: String,
    #[get = "pub"]
    friendly_name: String,
    #[get = "pub"]
    manufacturer: String,
    #[serde(rename = "manufacturerURL")]
    #[get = "pub"]
    manufacturer_url: Option<String>,
    #[get = "pub"]
    model_description: Option<String>,
    #[get = "pub"]
    model_name: String,
    #[get = "pub"]
    model_number: Option<String>,
    #[serde(rename = "modelURL")]
    #[get = "pub"]
    model_url: Option<String>,
    #[get = "pub"]
    serial_number: Option<String>,
    #[serde(rename = "UDN")]
    #[get = "pub"]
    udn: String,
    #[serde(rename = "UPC")]
    #[get = "pub"]
    upc: Option<String>,
    #[serde(default = "Default::default")]
    icon_list: Value<Vec<Icon>>,
    #[serde(default = "Default::default")]
    service_list: Value<Vec<Service>>,
    #[serde(default = "Default::default")]
    device_list: Value<Vec<DeviceSpec>>,
    #[serde(rename = "presentationURL")]
    #[get = "pub"]
    presentation_url: Option<String>,
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

#[derive(Deserialize, Debug, Getters)]
#[serde(rename_all = "camelCase")]
#[get = "pub"]
pub struct Icon {
    mimetype: String,
    width: u32,
    height: u32,
    depth: u32,
    url: String,
}

impl DeviceSpec {
    fn visit_devices<'a, F, T>(&'a self, f: F) -> Option<T>
    where
        F: Fn(&'a DeviceSpec) -> Option<T> + Copy,
    {
        if let Some(x) = f(&self) {
            return Some(x);
        }

        for device in self.devices() {
            if let Some(x) = device.visit_devices(f) {
                return Some(x);
            }
        }

        None
    }

    fn visit_services<'a, F, T>(&'a self, f: F) -> Option<T>
    where
        F: Fn(&'a Service) -> Option<T> + Copy,
    {
        self.visit_devices(|device| {
            for service in device.services() {
                if let Some(x) = f(service) {
                    return Some(x);
                }
            }
            None
        })
    }

    pub fn find_service(&self, service_type: &str) -> Option<&Service> {
        self.visit_services(|s| {
            if s.service_type() == service_type {
                return Some(s);
            }
            None
        })
    }

    fn get_services_inner<'a>(&'a self, acc: &mut Vec<&'a Service>) {
        for service in self.services() {
            acc.push(service);
        }
        for device in self.devices() {
            device.get_services_inner(acc);
        }
    }
    pub fn get_services<'a>(&'a self) -> Vec<&'a Service> {
        let mut acc = Vec::new();
        self.get_services_inner(&mut acc);
        acc
    }

    pub fn find_device(&self, device_type: &str) -> Option<&DeviceSpec> {
        self.visit_devices(|device| {
            if device.device_type == device_type {
                return Some(device);
            }
            None
        })
    }

    fn print_inner(&self, indentation: usize) {
        let i = "  ".repeat(indentation);

        println!("{}{}", i, self.device_type());
        for service in self.services() {
            println!("{}  - {}", i, service.service_type());
        }
        for device in self.devices() {
            device.print_inner(indentation + 1);
        }
    }

    pub fn print(&self) {
        self.print_inner(0);
    }
}
