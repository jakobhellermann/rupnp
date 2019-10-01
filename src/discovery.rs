use crate::{Device, Error};
use futures::prelude::*;
use ssdp_client::search::SearchTarget;
use std::time::Duration;

pub async fn discover(
    search_target: SearchTarget,
    timeout: Duration,
) -> Result<impl Stream<Item = Result<Device, Error>>, Error> {
    Ok(ssdp_client::search(search_target, timeout, 3)
        .await?
        .map(|search_response| Ok(search_response?.location().to_string().parse()?))
        .and_then(Device::from_url))
}
