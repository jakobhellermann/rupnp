use crate::{
    find_in_xml,
    utils::{self, HttpResponseExt},
    Result, Service,
};
use http::Uri;
use roxmltree::{Document, Node};
use ssdp_client::URN;

#[derive(Debug)]
/// A UPnP Device.
/// It stores its `Uri` and a [`DeviceSpec`], which contains information like the device type and
/// its list of inner devices and services.
pub struct Device {
    url: Uri,
    device_spec: DeviceSpec,
}
impl Device {
    pub fn url(&self) -> &Uri {
        &self.url
    }

    /// Creates a UPnP device from the given url.
    /// The url should point to the `/device_description.xml` or similar of the device.
    /// If you dont have access to the concrete location, have a look at [`discover`][fn.discover.html] instead.
    pub async fn from_url(url: Uri) -> Result<Self> {
        let body = isahc::get_async(&url)
            .await?
            .err_if_not_200()?
            .body_mut()
            .text_async()
            .await?;

        let document = Document::parse(&body)?;
        let device = utils::find_root(&document, "device", "Device Description")?;
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

/// Information about a device, like its 'friendly_name', device type etc.
/// Also includes its list of subdevices and services.
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
    fn from_xml<'a, 'input: 'a>(node: Node<'a, 'input>) -> Result<Self> {
        #[allow(non_snake_case)]
        let (device_type, friendly_name, services, devices) =
            find_in_xml! { node => deviceType, friendlyName, serviceList, ?deviceList };

        let devices = match devices {
            Some(d) => d
                .children()
                .filter(Node::is_element)
                .map(DeviceSpec::from_xml)
                .collect::<Result<_>>()?,
            None => Vec::new(),
        };
        let services = services
            .children()
            .filter(Node::is_element)
            .map(Service::from_xml)
            .collect::<Result<_>>()?;

        Ok(Self {
            device_type: utils::parse_node_text(device_type)?,
            friendly_name: utils::parse_node_text(friendly_name)?,
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

    /// Returns a list of this devices subdevices.
    /// Note that this does not recurse, if you want that behaviour use
    /// [devices_iter](struct.DeviceSpec.html#method.devices_iter) instead.
    pub fn devices(&self) -> &Vec<DeviceSpec> {
        &self.devices
    }

    /// Returns a list of this devices services.
    /// Note that this does not recurse, if you want that behaviour use
    /// [services_iter](struct.DeviceSpec.html#method.services_iter) instead.
    pub fn services(&self) -> &Vec<Service> {
        &self.services
    }

    /// Returns an Iterator of all services that can be used from this device.
    pub fn services_iter(&self) -> impl Iterator<Item = &Service> {
        self.services().iter().chain(self.devices().iter().flat_map(
            |device| -> Box<dyn Iterator<Item = &Service>> { Box::new(device.services_iter()) },
        ))
    }
    pub fn find_service(&self, service_type: &URN) -> Option<&Service> {
        self.services_iter()
            .find(|s| s.service_type() == service_type)
    }

    /// Returns an Iterator of all devices that can be used from this device.
    pub fn devices_iter(&self) -> impl Iterator<Item = &DeviceSpec> {
        self.devices().iter().chain(self.devices().iter().flat_map(
            |device| -> Box<dyn Iterator<Item = &DeviceSpec>> { Box::new(device.devices_iter()) },
        ))
    }
    pub fn find_device(&self, device_type: &URN) -> Option<&DeviceSpec> {
        self.devices_iter().find(|d| &d.device_type == device_type)
    }
}
