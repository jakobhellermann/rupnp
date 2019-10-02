use crate::error::{Error, UPnPError};
use crate::SCPD;
use crate::{find_in_xml, HttpResponseExt};
use isahc::http::Uri;
use isahc::prelude::*;
use roxmltree::{Document, Node};
use ssdp_client::search::URN;
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct Service {
    service_type: URN,
    service_id: String,
    scpd_endpoint: String,
    control_endpoint: String,
    event_sub_endpoint: String,
}

impl Service {
    pub(crate) fn from_xml(node: Node) -> Result<Self, Error> {
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

    pub fn service_type(&self) -> &URN {
        &self.service_type
    }

    pub fn service_id(&self) -> &str {
        &self.service_id
    }

    pub fn control_url(&self, url: &Uri) -> Uri {
        url_with_path(url, &self.control_endpoint)
    }
    pub fn scpd_url(&self, url: &Uri) -> Uri {
        url_with_path(url, &self.scpd_endpoint)
    }
    pub fn event_sub_url(&self, url: &Uri) -> Uri {
        url_with_path(url, &self.event_sub_endpoint)
    }

    pub async fn scpd(&self, url: &Uri) -> Result<SCPD, Error> {
        Ok(SCPD::from_url(&self.scpd_url(url), self.service_type().clone()).await?)
    }

    pub async fn action(
        &self,
        url: &Uri,
        action: &str,
        //arguments: HashMap<&str, &str>,
        payload: &str,
    ) -> Result<HashMap<String, String>, Error> {
        /*let mut payload = String::with_capacity(
            arguments
                .iter()
                .map(|(k, v)| 2 * k.len() + v.len() + 5)
                .sum(),
        );
        for (k, v) in &arguments {
            payload.push('<');
            payload.push_str(k);
            payload.push('>');
            payload.push_str(v);
            payload.push_str("</");
            payload.push_str(k);
            payload.push('>');
        }*/
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

    pub async fn subscribe(&self, url: &Uri, callback: &str) -> Result<(), Error> {
        let response = Request::builder()
            .uri(self.event_sub_url(url))
            .method("SUBSCRIBE")
            .header("CALLBACK", format!("<{}>", callback))
            .header("NT", "upnp:event")
            .header("TIMEOUT", "Second-300")
            .body(())
            .unwrap()
            .send_async()
            .await?;

        if response.status() != 200 {
            return Err(Error::HttpErrorCode(response.status()));
        }

        println!("{:?}", response.body());

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
