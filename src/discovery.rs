use crate::{Device, Error};
use futures::stream::{FuturesUnordered, Stream};
use ssdp_client::search::SearchTarget;
use std::time::Duration;

pub async fn discover(
    search_target: &SearchTarget,
    timeout: Duration,
) -> Result<impl Stream<Item = Result<Device, Error>>, Error> {
    Ok(ssdp_client::search(search_target, timeout, 3)
        .await?
        .iter()
        .map(|search_response| search_response.location().to_string().parse())
        .collect::<Result<Vec<_>, _>>()?
        .into_iter()
        .map(Device::from_url)
        .collect::<FuturesUnordered<_>>())
}
