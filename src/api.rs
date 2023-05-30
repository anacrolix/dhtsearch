use gloo_net::http::Request;
use log::info;
use serde::Deserialize;

#[derive(Clone, PartialEq, Deserialize, Default)]
#[serde(rename_all = "PascalCase")]
pub struct InfosSearch {
    pub total: usize,
    pub err: Option<String>,
    pub items: Vec<InfoItem>,
}

#[derive(Clone, PartialEq, Deserialize, Default)]
#[serde(rename_all = "PascalCase")]
pub struct InfoItem {
    pub info_hash: String,
    pub name: String,
    pub swarm_info: SwarmInfo,
    pub size: u64,
    pub age: String,
    pub no_swarm_info: bool,
}

#[derive(Clone, PartialEq, Deserialize, Default)]
#[serde(rename_all = "PascalCase")]
pub struct SwarmInfo {
    pub seeders: u32,
    pub completed: u32,
    pub leechers: u32,
}

pub async fn search(query: String) -> Result<InfosSearch, gloo_net::Error> {
    let url = if false {
        "/dhtindex/searchInfos?"
    } else {
        "https://dht-indexer-v2.fly.dev/searchInfos?"
    }
    .to_string();
    let url = url::form_urlencoded::Serializer::new(url)
        .extend_pairs(&[("s", query)])
        .finish();
    info!("searching {:?}", url);
    Request::get(url.as_ref()).send().await?.json().await
}
