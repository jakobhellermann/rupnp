use crate::error::{self, Error};
use futures::prelude::*;
use getset::Getters;
use hyper::header::HeaderValue;
use serde::Deserialize;
use ssdp_client::search::URN;
use xmltree::Element;

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

    pub async fn action<'a>(
        &'a self,
        ip: hyper::Uri,
        action: &'a str,
        payload: &'a str,
    ) -> Result<Element, Error> {
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
        *req.uri_mut() = self.control_url(ip);
        req.headers_mut().insert(
            hyper::header::CONTENT_TYPE,
            hyper::header::HeaderValue::from_static("xml"),
        );
        req.headers_mut().insert(
            "SOAPAction",
            header_value(&format!("\"{}#{}\"", self.service_type(), action))?,
        );

        let response_str = format!("{}Response", action);

        let res = client.request(req).await?;
        let body = res.into_body().try_concat().await?;

        let mut element = Element::parse(body.as_ref()).map_err(|_| Error::ParseError)?;
        let mut body = element
            .take_child("Body")
            .ok_or_else(|| Error::ParseError)?;

        if let Some(fault) = body.get_child("Fault") {
            return match error::parse(fault) {
                Ok(err) => Err(Error::UPnPError(err)),
                Err(err) => Err(err),
            };
        }

        if let Some(response) = body.take_child(response_str) {
            Ok(response)
        } else {
            Err(Error::ParseError)
        }
    }

    pub async fn subscribe<'a>(&'a self, ip: hyper::Uri, callback: &'a str) -> Result<(), Error> {
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
