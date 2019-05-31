use crate::error::{self, Error};
use failure::ResultExt;
use futures::compat::Future01CompatExt;
use futures01::{Future, Stream};
use getset::Getters;
use hyper::header::HeaderValue;
use serde::Deserialize;
use xmltree::Element;

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
        *req.uri_mut() = assemble_url(ip, self.control_url())?;
        req.headers_mut().insert(
            hyper::header::CONTENT_TYPE,
            hyper::header::HeaderValue::from_static("xml"),
        );
        req.headers_mut().insert(
            "SOAPAction",
            header_value(&format!("\"{}#{}\"", self.service_type(), action))?,
        );

        let response_str = format!("{}Response", action);

        let body = client
            .request(req)
            .and_then(|res| res.into_body().concat2())
            .map_err(Error::NetworkError)
            .compat()
            .await?;

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

    pub async fn subscribe<'a>(&'a self, ip: &'a str, callback: &'a str) -> Result<(), Error> {
        let client = hyper::client::Client::new();

        let mut req = hyper::Request::new(Default::default());
        *req.uri_mut() = assemble_url(ip, self.event_sub_url())?;
        *req.method_mut() = hyper::Method::from_bytes(b"SUBSCRIBE").expect("can not fail");
        req.headers_mut()
            .insert("CALLBACK", header_value(&format!("<{}>", callback))?);
        req.headers_mut()
            .insert("NT", HeaderValue::from_static("upnp:event"));
        req.headers_mut()
            .insert("TIMEOUT", HeaderValue::from_static("Second-300"));

        client
            .request(req)
            .and_then(|res| res.into_body().concat2())
            .map(|_chunks| {})
            .map_err(Error::NetworkError)
            .compat()
            .await
    }
}

fn header_value(s: &str) -> Result<hyper::http::header::HeaderValue, Error> {
    s.parse::<hyper::header::HeaderValue>()
        .with_context(|e| format!("invalid header: {}", e))
        .map_err(|e| Error::InvalidArguments(e.into()))
}

fn assemble_url(ip: &str, rest: &str) -> Result<hyper::Uri, Error> {
    format!("{}{}", ip, rest)
        .parse::<hyper::Uri>()
        .with_context(|e| format!("invalid url: {}", e))
        .map_err(|e| Error::InvalidArguments(e.into()))
}

pub fn urn_to_name(urn: &str) -> Option<String> {
    let mut x = urn.rsplitn(3, ':');
    Some(format!(
        "{name}{version}",
        version = x.next()?,
        name = x.next()?
    ))
}
