use crate::{Device, Error};
use futures_util::{
    stream::{Stream, StreamExt},
    try_stream::TryStreamExt,
};
use ssdp_client::search::SearchTarget;
use std::time::Duration;

pub async fn discover(
    search_target: &SearchTarget,
    timeout: Duration,
) -> Result<impl Stream<Item = Result<Device, Error>>, Error> {
    Ok(ssdp_client::search(search_target, timeout, 3)
        .await?
        .map(|res| match res {
            Ok(search_response) => Ok(search_response.location().to_string().parse()?),
            Err(e) => Err(Error::SSDPError(e)),
        })
        .and_then(|x| Device::from_url(x)))
}
