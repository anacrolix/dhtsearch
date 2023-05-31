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

#[derive(Clone, PartialEq, Deserialize, Default, Debug)]
#[serde(rename_all = "PascalCase")]
pub struct SwarmInfo {
    pub seeders: u32,
    pub completed: u32,
    pub leechers: u32,
}

#[derive(Clone, PartialEq, Deserialize, Default, Debug)]
#[serde(rename_all = "PascalCase")]
pub struct InfoFiles {
    pub info: Info,
    pub files: Vec<File>,
}

type InfoFilesPayload = Vec<InfoFiles>;

#[derive(Clone, PartialEq, Deserialize, Default, Debug)]
#[serde(rename_all = "PascalCase")]
pub struct Info {
    pub info_id: i64,
    // This is sent as base64 from Go, might need a custom module.
    pub name: String,
    pub info_hash: String,
    pub age: String,
    // The variant here might need optional fields.
    pub scrape_data: SwarmInfo,
    // This looks optional in Go.
    pub scrape_time: String,
}

#[derive(Clone, PartialEq, Deserialize, Default, Debug)]
#[serde(rename_all = "PascalCase")]
pub struct File {
    // This might be optional (for null), and might be sent as byte array that needs manual
    // splitting.
    pub path: Option<Vec<String>>,
    pub length: i64,
}

const DHT_INDEXER_URL: &str = if false {
    "/dhtindex/"
} else {
    "https://dht-indexer-v2.fly.dev/"
};

pub type Result<T> = std::result::Result<T, gloo_net::Error>;

pub async fn search(query: String) -> Result<InfosSearch> {
    // return Err(GlooError("shit".to_string()));
    let mut url = DHT_INDEXER_URL.to_string();
    url.push_str("searchInfos?");
    let url = url::form_urlencoded::Serializer::new(url)
        .extend_pairs(&[("s", query)])
        .finish();
    info!("searching {:?}", url);
    Request::get(url.as_ref()).send().await?.json().await
}

pub async fn get_info_files(info_hashes: Vec<String>) -> Result<InfoFilesPayload> {
    // return Err(GlooError("shit".to_string()));
    let mut url = DHT_INDEXER_URL.to_string();
    url.push_str("infoFiles?");
    let url = url::form_urlencoded::Serializer::new(url)
        .extend_pairs(info_hashes.iter().map(|ih| ("ih", ih)))
        .finish();
    Request::get(url.as_ref())
        .header("Accept", "application/json")
        .send()
        .await?
        .json()
        .await
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_deserialize_info_files() -> serde_json::Result<()> {
        let data = r#"
        [
          {
            "Info": {
              "InfoId": 9223354404497263000,
              "Name": "VGhlLkludGVybmV0cy5Pd24uQm95LlRoZS5TdG9yeS5vZi5BYXJvbi5Td2FydHouNzIwcC5IRFJpcC54MjY0LkFBQy5NVkdyb3VwLm9yZy5tcDQ=",
              "InfoHash": "40f3761b9080949ca6ffed3522ad872bc0bef41b",
              "Age": "2022-11-11T01:47:42Z",
              "ScrapeData": {
                "Completed": 44,
                "Seeders": 16,
                "Leechers": 1
              },
              "ScrapeTime": "2023-05-08T15:05:43Z"
            },
            "Files": [
              {
                "Path": null,
                "Length": 1701564468
              }
            ]
          }
        ]
        "#;
        let v: InfoFilesPayload = serde_json::from_str(data)?;
        assert_eq!(v.len(), 1);
        assert_eq!(v[0].files.len(), 1);
        Ok(())
    }
}
