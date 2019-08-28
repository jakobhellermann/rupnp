use crate::{Device, Error};
use futures::prelude::*;
use ssdp_client::search::SearchTarget;
use std::time::Duration;

pub async fn discover(
    search_target: SearchTarget<'_>,
    timeout: Duration,
) -> Result<impl Stream<Item = Result<Device, Error>>, Error> {
    Ok(ssdp_client::search(search_target, timeout, 3)
        .await?
        .map(|search_response| -> Result<hyper::Uri, Error> {
            search_response?
                .location()
                .parse()
                .map_err(|e| Error::InvalidResponse(Box::new(e)))
        })
        .and_then(Device::from_url))
}
