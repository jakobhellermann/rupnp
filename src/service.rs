use crate::error::{Error, UPnPError};
use futures::prelude::*;
use roxmltree::Document;
use serde::Deserialize;
use ssdp_client::search::URN;
use std::collections::HashMap;
use surf::http::Method;

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

impl Service {
    pub fn service_type(&self) -> &URN<'static> {
        &self.service_type
    }

    pub fn service_id(&self) -> &str {
        &self.service_id
    }

    pub fn control_url(&self, url: &surf::url::Url) -> surf::url::Url {
        let mut control_url = url.clone();
        control_url.set_path(&self.control_endpoint);
        control_url
    }
    pub fn scpd_url(&self, url: &surf::url::Url) -> surf::url::Url {
        let mut scpd_url = url.clone();
        scpd_url.set_path(&self.scpd_endpoint);
        scpd_url
    }
    pub fn event_sub_url(&self, url: &surf::url::Url) -> surf::url::Url {
        let mut event_sub_url = url.clone();
        event_sub_url.set_path(&self.event_sub_endpoint);
        event_sub_url
    }

    pub async fn action(
        &self,
        url: &surf::url::Url,
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

        let response = surf::post(self.control_url(url))
            .body_string(body)
            .set_header("CONTENT-TYPE", "xml")
            .set_header(
                "SOAPAction",
                format!("\"{}#{}\"", &self.service_type, action),
            )
            .recv_string()
            .map_err(Error::NetworkError)
            .await?;

        let document = Document::parse(&response)?;

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

    pub async fn subscribe(&self, url: &surf::url::Url, callback: &str) -> Result<(), Error> {
        let mut response = surf::Request::new(
            Method::from_bytes(b"SUSBSCRIBE").unwrap(),
            self.event_sub_url(url),
        )
        .set_header("CALLBACK", format!("<{}>", callback))
        .set_header("NT", "upnp:event")
        .set_header("TIMEOUT", "Second-300")
        .await
        .map_err(Error::NetworkError)?;

        if response.status() != 200 {
            return Err(Error::HttpErrorCode(response.status()));
        }

        let body = response.body_string().await.map_err(Error::NetworkError)?;
        dbg!(body);

        unimplemented!()
    }
}
