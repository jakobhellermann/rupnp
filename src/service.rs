use crate::error::{Error, UPnPError};
use futures::prelude::*;
use getset::Getters;
use hyper::header::HeaderValue;
use roxmltree::Document;
use serde::Deserialize;
use ssdp_client::search::URN;
use std::collections::HashMap;

#[derive(Deserialize, Debug, Getters, Clone)]
#[serde(rename_all = "camelCase")]
#[get = "pub"]
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
    pub fn control_url(&self, ip: hyper::Uri) -> hyper::Uri {
        assemble_url(ip, &self.control_endpoint)
    }
    pub fn scpd_url(&self, ip: hyper::Uri) -> hyper::Uri {
        assemble_url(ip, &self.scpd_endpoint)
    }
    pub fn event_sub_url(&self, ip: hyper::Uri) -> hyper::Uri {
        assemble_url(ip, &self.event_sub_endpoint)
    }

    pub async fn action(
        &self,
        ip: hyper::Uri,
        action: &str,
        arguments: HashMap<&str, &str>,
    ) -> Result<HashMap<String, String>, Error> {
        let client = hyper::Client::new();

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
            service = self.service_type(),
            action = action,
            payload = payload
        );

        let mut req = hyper::Request::new(hyper::Body::from(body));
        *req.method_mut() = hyper::Method::POST;
        *req.uri_mut() = self.control_url(ip);
        req.headers_mut().insert(
            hyper::header::CONTENT_TYPE,
            hyper::header::HeaderValue::from_static("xml"),
        );
        req.headers_mut().insert(
            "SOAPAction",
            header_value(&format!("\"{}#{}\"", self.service_type(), action))?,
        );

        let response = client.request(req).await?.into_body().try_concat().await?;
        let document = Document::parse(std::str::from_utf8(&response)?)?;

        let body = document
            .root()
            .first_children()
            .find(|x| x.has_tag_name("Body"))
            .ok_or(Error::ParseError)?;

        match body.first_element_child().ok_or(Error::ParseError)? {
            fault if fault.tag_name().name() == "Fault" => Err(UPnPError::from_fault_node(fault)),
            res if res.tag_name().name().starts_with(action) => res
                .children()
                .map(|node| -> Result<(String, String), Error> {
                    if let Some(text) = node.text() {
                        Ok((node.tag_name().name().to_string(), text.to_string()))
                    } else {
                        Err(Error::ParseError)
                    }
                })
                .collect(),
            _ => Err(Error::ParseError),
        }
    }

    pub async fn subscribe(&self, ip: hyper::Uri, callback: &str) -> Result<(), Error> {
        let client = hyper::client::Client::new();

        let mut req = hyper::Request::new(Default::default());
        *req.uri_mut() = self.event_sub_url(ip);
        *req.method_mut() = hyper::Method::from_bytes(b"SUBSCRIBE").expect("can not fail");
        req.headers_mut()
            .insert("CALLBACK", header_value(&format!("<{}>", callback))?);
        req.headers_mut()
            .insert("NT", HeaderValue::from_static("upnp:event"));
        req.headers_mut()
            .insert("TIMEOUT", HeaderValue::from_static("Second-300"));

        let _ = client.request(req).await?;

        Ok(())
    }
}

fn header_value(s: &str) -> Result<hyper::http::header::HeaderValue, Error> {
    s.parse::<hyper::header::HeaderValue>()
        .map_err(|e| Error::InvalidArguments(Box::new(e)))
}

fn assemble_url(ip: hyper::Uri, rest: &str) -> hyper::Uri {
    let mut parts = ip.into_parts();
    parts.path_and_query = Some(
        hyper::http::uri::PathAndQuery::from_shared(rest.into())
            .expect("url part assemble logic does not work"),
    );
    hyper::Uri::from_parts(parts).expect("url part assemble logic does not work")
}
