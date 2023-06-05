use super::*;
use gloo_net::http::Request;
use gloo_net::http::Response;
use log::info;
use serde::de::DeserializeOwned;
use serde::Deserialize;
use std::fmt::Debug;

mod info_name;

use info_name::InfoName;

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

pub struct UpvertedFile {
    pub path: Vec<String>,
    pub length: FileLength,
}

impl InfoFiles {
    pub fn upverted_files(&self) -> Vec<UpvertedFile> {
        self.files
            .iter()
            .map(|file| UpvertedFile {
                path: file
                    .path
                    .clone()
                    .unwrap_or_else(|| vec![self.info.name.to_string()]),
                length: file.length,
            })
            .collect()
    }
}

type InfoFilesPayload = Vec<InfoFiles>;

#[derive(Clone, PartialEq, Deserialize, Default, Debug)]
#[serde(rename_all = "PascalCase")]
pub struct Info {
    pub info_id: i64,
    // This is sent as base64 from Go.
    pub name: InfoName,
    pub info_hash: String,
    pub age: String,
    // The variant here might need optional fields.
    pub scrape_data: SwarmInfo,
    // This looks optional in Go.
    pub scrape_time: String,
}

pub type FileLength = i64;

#[derive(Clone, PartialEq, Deserialize, Default, Debug)]
#[serde(rename_all = "PascalCase")]
pub struct File {
    // This might be optional (for null), and might be sent as byte array that needs manual
    // splitting.
    pub path: Option<Vec<String>>,
    pub length: FileLength,
}

const DHT_INDEXER_URL: &str = if false {
    "/dhtindex/"
} else {
    "https://dht-indexer-v2.fly.dev/"
};

pub async fn search(query: String) -> Result<InfosSearch> {
    // return Err(GlooError("shit".to_string()));
    let mut url = DHT_INDEXER_URL.to_string();
    url.push_str("searchInfos?");
    let url = url::form_urlencoded::Serializer::new(url)
        .extend_pairs(&[("s", query)])
        .finish();
    info!("searching {:?}", url);
    Ok(Request::get(url.as_ref()).send().await?.json().await?)
}

async fn handle_go_json_response<T: DeserializeOwned>(
    resp: Response,
) -> std::result::Result<T, Error> {
    if !resp.ok() {
        return Err(Error::Anyhow(anyhow::anyhow!(std::str::from_utf8(
            &resp.binary().await?
        )
        .map_err(anyhow::Error::new)?
        .to_owned())));
    }
    resp.json().await.map_err(Into::into)
}

pub async fn get_info_files(info_hashes: &[String]) -> Result<InfoFilesPayload> {
    let mut url = DHT_INDEXER_URL.to_string();
    url.push_str("infoFiles?");
    let url = url::form_urlencoded::Serializer::new(url)
        .extend_pairs(info_hashes.iter().map(|ih| ("ih", ih)))
        .finish();
    let response = Request::get(url.as_ref())
        .header("Accept", "application/json")
        // I think this gets clobbered by the JS fetch API. I also doubt that lz4 is a valid
        // encoding for browsers by default.
        .header("Accept-Encoding", "lz4, br")
        .send()
        .await
        .map_err(Into::into)
        .map_err(Arc::new)?;
    handle_go_json_response(response).await.map_err(Into::into)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test_log::test]
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
        assert_eq!(
            &v[0].info.name,
            "The.Internets.Own.Boy.The.Story.of.Aaron.Swartz.720p.HDRip.x264.AAC.MVGroup.org.mp4"
        );
        assert_eq!(format!("{:?}", v[0].info.name), format!("{:?}","The.Internets.Own.Boy.The.Story.of.Aaron.Swartz.720p.HDRip.x264.AAC.MVGroup.org.mp4"));
        Ok(())
    }
}
