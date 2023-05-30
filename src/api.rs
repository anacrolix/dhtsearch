use serde::Deserialize;

#[derive(Clone, PartialEq, Deserialize, Default)]
#[serde(rename_all = "PascalCase")]
pub(crate) struct InfosSearch {
    pub total: usize,
    pub err: Option<String>,
    pub items: Vec<InfoItem>,
}

#[derive(Clone, PartialEq, Deserialize, Default)]
#[serde(rename_all = "PascalCase")]
pub(crate) struct InfoItem {
    pub info_hash: String,
    pub name: String,
    pub swarm_info: SwarmInfo,
    pub size: u64,
    pub age: String,
    pub no_swarm_info: bool,
}

#[derive(Clone, PartialEq, Deserialize, Default)]
#[serde(rename_all = "PascalCase")]
pub(crate) struct SwarmInfo {
    pub seeders: u32,
    pub completed: u32,
    pub leechers: u32,
}
