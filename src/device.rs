use crate::Error;
use ssdp_client::search::URN;
use crate::HttpResponseExt;
use isahc::http::Uri;
use roxmltree::Document;

macro_rules! find_in_xml {
    ( $name:literal $xml:expr => $( $var:ident: $($ty:ty)? $( |optional| $ty_opt:ty)? ,)* ) => { {
        $(let mut $var: Option<&str> = None;)*

        for node in $xml.descendants() {
            match node.tag_name().name() {
                $(stringify!($var) => $var = node.text(),)*
                _ => (),
            }
        }

        $(
            $(
                let $var = $var.ok_or_else(|| Error::MissingXMLElement($name, stringify!($var)))?;
                let $var = $var.parse::<$ty>().map_err(|e| Error::InvalidResponse(Box::new(e)))?;
            )?
            $(
                let $var = match $var {
                    Some(var) => Some(var.parse::<$ty_opt>().map_err(|e| Error::InvalidResponse(Box::new(e)))?),
                    None => None,
                };
            )?
        )*
        Ok::<_, Error>(($($var),*))
    } }
}

#[derive(Debug)]
pub struct Device {
    url: Uri,
    spec_version: (u8, u8),
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

        let device_description = Document::parse(&body)?;

        #[allow(non_snake_case, unused)]
        let (
            major, minor,
            device_type, friendly_name,
            manufacturer, manufacturer_url,
            model_description, model_number, model_url, serial_number,
            udn, upc,
            presentation_url
        ) = find_in_xml! { "Device description" device_description =>
            major: u8, minor: u8,
            deviceType: URN, friendlyName: String,
            manufacturer: String, manufacturerURL: |optional| String, modelDescription: |optional| String, modelNumber: |optional| String, modelURL: |optional| String, serialNumber: |optional| String,
            UDN: String, UPC: |optional| String,
            // todo
            presentationURL: |optional| String,
        }?;

        Ok(Self {
            url,
            spec_version: (major, minor),
            device_spec: DeviceSpec {
                device_type, friendly_name,
                manufacturer, manufacturer_url,
                model_description, model_number, model_url, serial_number,
                udn, upc,
                presentation_url
            },
        })
    }
}

#[derive(Debug)]
pub struct DeviceSpec {
    pub device_type: URN<'static>,
    pub friendly_name: String,
    pub manufacturer: String,
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
    pub presentation_url: Option<String>,
}

impl std::ops::Deref for Device {
    type Target = DeviceSpec;

    fn deref(&self) -> &Self::Target {
        &self.device_spec
    }
}

/*

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
}*/
