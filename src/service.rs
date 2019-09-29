use crate::error::{Error, UPnPError};
use roxmltree::Document;
use serde::Deserialize;
use ssdp_client::search::URN;
use std::collections::HashMap;
use isahc::http::Uri;
use isahc::prelude::*;

#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Service {
    #[serde(deserialize_with = "crate::shared::deserialize_urn")]
    service_type: URN<'static>,
    service_id: String,
    #[serde(rename = "SCPDURL")]
    scpd_endpoint: String,
    #[serde(rename = "controlURL")]
    control_endpoint: String,
    #[serde(rename = "eventSubURL")]
    event_sub_endpoint: String,
}

fn url_with_path(url: &Uri, path: &str) -> Uri {
    let mut builder = Uri::builder();
    if let Some(authority) = url.authority_part() {
        builder.authority(authority.clone());
    }
    if let Some(scheme) = url.scheme_part() {
        builder.scheme(scheme.clone());
    }
    builder
        .path_and_query(path)
        .build()
        .expect("infallible")
}

impl Service {
    pub fn service_type(&self) -> &URN<'static> {
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

    pub async fn action(
        &self,
        url: &Uri,
        action: &str,
        arguments: HashMap<&str, &str>,
    ) -> Result<HashMap<String, String>, Error> {
        let mut payload = String::with_capacity(
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
        }
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

        let mut response = Request::post(self.control_url(url))
            .header("CONTENT-TYPE", "xml")
            .header(
                "SOAPAction",
                format!("\"{}#{}\"", &self.service_type, action),
            )
            .body(body).unwrap()
            .send_async()
            .await?;

        if response.status() != 200 {
            return Err(Error::HttpErrorCode(response.status()));
        }

        let doc = response.text_async().await?;
        let document = Document::parse(&doc)?;

        let body = document
            .root()
            .first_children()
            .find(|x| x.has_tag_name("Body"))
            .ok_or(Error::ParseError(
                "upnp response doesn't contain a `Body` element",
            ))?;

        match body.first_element_child().ok_or(Error::ParseError(
            "the upnp responses `Body` element has no children",
        ))? {
            fault if fault.tag_name().name() == "Fault" => Err(UPnPError::from_fault_node(fault)),
            res if res.tag_name().name().starts_with(action) => res
                .children()
                .map(|node| -> Result<(String, String), Error> {
                    if let Some(text) = node.text() {
                        Ok((node.tag_name().name().to_string(), text.to_string()))
                    } else {
                        Err(Error::ParseError(
                            "upnp response element has no text attached",
                        ))
                    }
                })
                .collect(),
            _ => Err(Error::ParseError(
                "upnp response contains neither `fault` nor `${ACTION}Response` element",
            )),
        }
    }

    pub async fn subscribe(&self, url: &Uri, callback: &str) -> Result<(), Error> {
        let response = Request::builder()
            .uri(self.event_sub_url(url))
            .method("SUBSCRIBE")
            .header("CALLBACK", format!("<{}>", callback))
            .header("NT", "upnp:event")
            .header("TIMEOUT", "Second-300")
            .body(()).unwrap()
            .send_async().await?;

        if response.status() != 200 {
            return Err(Error::HttpErrorCode(response.status()));
        }

        dbg!(response.body());

        Ok(())
    }
}
