use crate::error::{self, Error};
use futures01::{Future, Stream};
use getset::Getters;
use hyper::header::HeaderValue;
use serde::Deserialize;
use xmltree::Element;

use futures::compat::Future01CompatExt;

#[derive(Deserialize, Debug, Getters, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Service {
    service_type: String,
    #[get = "pub"]
    service_id: String,
    #[serde(rename = "SCPDURL")]
    #[get = "pub"]
    scpd_url: String,
    #[serde(rename = "controlURL")]
    #[get = "pub"]
    control_url: String,
    #[serde(rename = "eventSubURL")]
    #[get = "pub"]
    event_sub_url: String,
}

impl Service {
    pub fn service_type(&self) -> &str {
        self.service_type.trim_start_matches("urn:")
    }

    pub async fn action<'a>(
        &'a self,
        ip: &'a str,
        action: &'a str,
        payload: &'a str,
    ) -> Result<Element, Error> {
        let client = hyper::Client::new();

        let body = format!(
            r#"
            <s:Envelope xmlns:s="http://schemas.xmlsoap.org/soap/envelope/"
                s:encodingStyle="http://schemas.xmlsoap.org/soap/encoding/">
                <s:Body>
                    <u:{action} xmlns:u="urn:{service}">
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
        *req.uri_mut() = format!("{}{}", ip, self.control_url()).parse().unwrap(); // FIXME
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

        let response_str = format!("{}Response", action);

        let body = await!(client
            .request(req)
            .and_then(|res| res.into_body().concat2())
            .map_err(Error::NetworkError)
            .compat())?;

        let mut element = Element::parse(body.as_ref())?;
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

    pub async fn subscribe<'a>(
        &'a self,
        ip: &'a str,
        callback: &'a str,
    ) -> Result<(), hyper::Error> {
        let client = hyper::client::Client::new();

        let mut req = hyper::Request::new(Default::default());
        *req.uri_mut() = format!("{}{}", ip, self.event_sub_url()).parse().unwrap(); // FIXME
        *req.method_mut() = hyper::Method::from_bytes(b"SUBSCRIBE").unwrap();
        req.headers_mut().insert(
            "CALLBACK",
            HeaderValue::from_str(&format!("<{}>", callback)).unwrap(),
        );
        req.headers_mut()
            .insert("NT", HeaderValue::from_static("upnp:event"));
        req.headers_mut()
            .insert("TIMEOUT", HeaderValue::from_static("Second-300"));

        await!(client
            .request(req)
            .and_then(|res| res.into_body().concat2())
            .map(|_chunks| {})
            .compat())
    }
}

pub fn urn_to_name(urn: &str) -> String {
    let mut x = urn.rsplitn(3, ':');
    format!(
        "{name}{version}",
        version = x.next().unwrap(),
        name = x.next().unwrap()
    )
}
