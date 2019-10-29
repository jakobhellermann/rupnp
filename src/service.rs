use crate::error::{Error, UPnPError};
use crate::scpd::SCPD;
use crate::{find_in_xml, HttpResponseExt};
use isahc::http::Uri;
use isahc::prelude::*;
use roxmltree::{Document, Node};
use ssdp_client::search::URN;
use std::collections::HashMap;

/// A UPnP Service is the description of endpoints on a device for performing actions and reading
/// the service definition.
/// For a list of actions and state variables the service provides, take a look at [scpd](struct.Service.html#method.scpd).
#[derive(Debug, Clone)]
pub struct Service {
    service_type: URN,
    service_id: String,
    scpd_endpoint: String,
    control_endpoint: String,
    event_sub_endpoint: String,
}

impl Service {
    pub(crate) fn from_xml(node: Node<'_, '_>) -> Result<Self, Error> {
        #[allow(non_snake_case)]
        let (service_type, service_id, scpd_endpoint, control_endpoint, event_sub_endpoint) =
            find_in_xml! { node => serviceType, serviceId, SCPDURL, controlURL, eventSubURL };

        Ok(Self {
            service_type: crate::parse_node_text(service_type)?,
            service_id: crate::parse_node_text(service_id)?,
            scpd_endpoint: crate::parse_node_text(scpd_endpoint)?,
            control_endpoint: crate::parse_node_text(control_endpoint)?,
            event_sub_endpoint: crate::parse_node_text(event_sub_endpoint)?,
        })
    }

    /// Returns the [URN](ssdp_client::URN) of this service.
    pub fn service_type(&self) -> &URN {
        &self.service_type
    }

    /// Returns the `Service Identifier`.
    pub fn service_id(&self) -> &str {
        &self.service_id
    }

    pub(crate) fn control_url(&self, url: &Uri) -> Uri {
        url_with_path(url, &self.control_endpoint)
    }
    pub(crate) fn scpd_url(&self, url: &Uri) -> Uri {
        url_with_path(url, &self.scpd_endpoint)
    }
    pub(crate) fn event_sub_url(&self, url: &Uri) -> Uri {
        url_with_path(url, &self.event_sub_endpoint)
    }

    /// Fetches the [`SCPD`](scpd/struct.SCPD.html) of this service.
    pub async fn scpd(&self, url: &Uri) -> Result<SCPD, Error> {
        Ok(SCPD::from_url(&self.scpd_url(url), self.service_type().clone()).await?)
    }

    /// Execute some UPnP Action on this service.
    /// The URL is usually obtained by the device this service was found on.
    /// The payload is xml-formatted data.
    ///
    /// # Example usage:
    ///
    /// ```rust,no_run
    /// # use ssdp_client::URN;
    /// # async fn rendering_control_example() -> Result<(), Box<dyn std::error::Error>> {
    /// # let some_url = unimplemented!();
    /// use upnp::ssdp::URN;
    /// use upnp::Device;
    ///
    /// let urn = URN::service("schemas-upnp-org", "RenderingControl", 1);
    ///
    /// let device = Device::from_url( some_url ).await?;
    /// let service = device.find_service(&urn)
    ///     .expect("service exists");
    ///
    /// let args = "<InstanceID>0</InstanceID><Channel>Master</Channel>";
    /// let response = service.action(device.url(), "GetVolume", args).await?;
    ///
    /// let volume = response
    ///     .get("CurrentVolume")
    ///     .expect("exists");
    ///
    /// println!("Volume: {}", volume);
    /// # Ok(())
    /// # }
    /// ```
    pub async fn action(
        &self,
        url: &Uri,
        action: &str,
        payload: &str,
    ) -> Result<HashMap<String, String>, Error> {
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
            service = &self.service_type,
            action = action,
            payload = payload
        );

        let doc = Request::post(self.control_url(url))
            .header("CONTENT-TYPE", "xml")
            .header(
                "SOAPAction",
                format!("\"{}#{}\"", &self.service_type, action),
            )
            .body(body)
            .unwrap()
            .send_async()
            .await?
            .err_if_not_200()?
            .text_async()
            .await?;

        let document = Document::parse(&doc)?;
        let body = crate::find_root(&document, "Body", "UPnP Response")?;

        let first_child = body.first_element_child().ok_or(Error::ParseError(
            "the upnp responses `Body` element has no children",
        ))?;

        if first_child.tag_name().name().eq_ignore_ascii_case("Fault") {
            Err(UPnPError::from_fault_node(first_child)?.into())
        } else if first_child.tag_name().name().starts_with(action) {
            Ok(first_child
                .children()
                .filter(Node::is_element)
                .filter_map(|node| -> Option<(String, String)> {
                    if let Some(text) = node.text() {
                        Some((node.tag_name().name().to_string(), text.to_string()))
                    } else {
                        None
                    }
                })
                .collect::<HashMap<_, _>>())
        } else {
            Err(Error::ParseError(
                "upnp response contains neither `fault` nor `${ACTION}Response` element",
            ))
        }
    }

    /// Subscribe for state variable changes.
    /// The control point will make `NOTIFY` requests to the given callback uri.
    ///
    /// # Example usage using async-std:
    /// ```rust,no_run
    #[doc(include = "../examples/subscribe_device.rs")]
    /// ```
    pub async fn subscribe(&self, url: &Uri, callback: &str) -> Result<(), Error> {
        let _response = Request::builder()
            .uri(self.event_sub_url(url))
            .method("SUBSCRIBE")
            .header("CALLBACK", format!("<{}>", callback))
            .header("NT", "upnp:event")
            .header("TIMEOUT", "Second-300")
            .body(())
            .unwrap()
            .send_async()
            .await?
            .err_if_not_200()?;
        Ok(())
    }
}

fn url_with_path(url: &Uri, path: &str) -> Uri {
    let mut builder = Uri::builder();
    if let Some(authority) = url.authority_part() {
        builder.authority(authority.clone());
    }
    if let Some(scheme) = url.scheme_part() {
        builder.scheme(scheme.clone());
    }
    builder.path_and_query(path).build().expect("infallible")
}
