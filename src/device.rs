use crate::shared::{SpecVersion, Value};
use getset::Getters;
use serde::Deserialize;

use futures::{Future, Stream};

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
struct DeviceDescription {
    spec_version: SpecVersion,
    device: Device,
}

#[derive(Deserialize, Debug, Getters)]
#[serde(rename_all = "camelCase")]
pub struct Device {
    #[serde(default = "String::new")]
    #[get = "pub"]
    ip: String,
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
    device_list: Value<Vec<Device>>,
    #[serde(rename = "presentationURL")]
    #[get = "pub"]
    presentation_url: Option<String>,
}

impl Device {
    pub fn services(&self) -> &Vec<Service> {
        &self.service_list.value
    }
    pub fn devices(&self) -> &Vec<Device> {
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

#[derive(Deserialize, Debug, Getters, Clone)]
#[serde(rename_all = "camelCase")]
#[get = "pub"]
pub struct Service {
    service_type: String,
    service_id: String,
    #[serde(rename = "SCPDURL")]
    scpd_url: String,
    #[serde(rename = "controlURL")]
    control_url: String,
    #[serde(rename = "eventSubURL")]
    event_sub_url: String,
}
impl Service {
    pub fn action(
        &self,
        ip: &str,
        action: &str,
        payload: &str,
    ) -> impl Future<Item = String, Error = failure::Error> {
        let client = hyper::Client::new();

        let body = format!(
            r#"
            <s:Envelope xmlns:s="http://schemas.xmlsoap.org/soap/envelope/"
                s:encodingStyle="http://schemas.xmlsoap.org/soap/encoding/">
                <s:Body>
                    <u:{action} xmlns:u="{service}">
                        {payload}
                    </u:{action}>
                </s:Body>
            </s:Envelope>"#,
            service = self.service_type(),
            action = action,
            payload = payload
        );

        let mut req = hyper::Request::new(hyper::Body::from(body));
        *req.method_mut() = hyper::Method::POST;
        *req.uri_mut() = format!("{}{}", ip, self.control_url()).parse().unwrap();
        req.headers_mut().insert(
            hyper::header::CONTENT_TYPE,
            hyper::header::HeaderValue::from_static("xml"),
        );
        req.headers_mut().insert(
            "SOAPAction",
            format!("\"{}#{}\"", self.service_type(), action)
                .parse()
                .unwrap(),
        );

        client
            .request(req)
            .and_then(|res| res.into_body().concat2())
            .map_err(failure::Error::from)
            .map(|body| String::from_utf8_lossy(body.as_ref()).to_string())
    }
}

impl Device {
    pub fn from_url(uri: hyper::Uri) -> impl Future<Item = Self, Error = failure::Error> {
        let client = hyper::Client::new();

        let ip = format!(
            "{}://{}",
            uri.scheme_str().unwrap(),
            uri.authority_part().unwrap()
        );

        client
            .get(uri)
            .and_then(|response| response.into_body().concat2())
            .map_err(failure::Error::from)
            .map(|body| {
                let device_description: DeviceDescription =
                    serde_xml_rs::from_reader(&body[..]).unwrap();
                assert!(
                    device_description.spec_version.major() == 1,
                    format!(
                        "unable to parse spec version {}.{}",
                        device_description.spec_version.major(),
                        device_description.spec_version.minor()
                    )
                );
                device_description.device
            })
            .map(move |mut device| {
                device.ip = ip;
                device
            })
    }

    fn visit_devices<'a, F, T>(&'a self, f: F) -> Option<T>
    where
        F: Fn(&'a Device) -> Option<T> + Copy,
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
            return None;
        })
    }

    pub fn find_service(&self, service_type: &str) -> Option<&Service> {
        self.visit_services(|s| {
            if s.service_type == service_type {
                return Some(s);
            }
            return None;
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

    pub fn find_device(&self, device_type: &str) -> Option<&Device> {
        self.visit_devices(|device| {
            if device.device_type == device_type {
                return Some(device);
            }
            return None;
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

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
struct UPnPError {
    faultcode: String,
    faultstring: String,
    #[serde(default = "Default::default")]
    err_code: u16
}
impl UPnPError {
    fn err_code_description(&self) -> &str {
        match self.err_code {
            401 => "No action by that name at this service.",
            402 => "Invalid Arguments",
            403 => "(deprecated error code)",
            501 => "Action failed",
            600 => "Argument value invalid",
            601 => "Argument Value Out of Range",
            602 => "Optional Action Not Implemented",
            603 => "Out of Memory",
            604 => "Human Intervention Required",
            605 => "String Argument Too Long",
            606..=612 => "(error code reserved for UPnP DeviceSecurity)",
            613..=699 => "Common action error. Defined by UPnP Forum Technical Committee.",
            700..=799 => "Action-specific error defined by UPnP Forum working committee.",
            800..=899 => "Action-specific error for non-standard actions. Defined by UPnP vendor.",
            _ => "Invalid Error Code"
        }
    }
}
impl std::fmt::Display for UPnPError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{} Error {} ({}): {}", self.faultstring, self.err_code, self.faultcode, self.err_code_description())
    }
}
impl std::error::Error for UPnPError {}

pub fn parse_error<T>(response: &str) -> Result<T, failure::Error> {
    let fault_start = response.find("<s:Body>").ok_or(failure::err_msg("malformed error reponse"))? + 8;
    let fault_end = response.rfind("</s:Body>").unwrap();

    let body = response[fault_start..fault_end].replace("s:", "").replace(" xmlns=\"urn:schemas-upnp-org:control-1-0\"", "");

    let mut fault: UPnPError = serde_xml_rs::from_reader(body.as_bytes()).unwrap();

    let errcode_start = body.find("<errorCode>").unwrap() + 11;
    let errcode_end = body.rfind("</errorCode>").unwrap();

    fault.err_code = body[errcode_start..errcode_end].parse().unwrap();

    Err(failure::Error::from(fault))
}

pub fn urn_to_name(urn: &str) -> String {
    let mut x = urn.rsplitn(3, ':');
    format!("{name}{version}", version=x.next().unwrap(), name=x.next().unwrap())
}