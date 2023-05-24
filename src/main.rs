use gloo_net::http::Request;
use gloo_net::http::RequestMode::NoCors;
use serde::Deserialize;
use yew::prelude::*;

#[function_component(App)]
fn app() -> Html {
    let torrents = use_state(|| {
        Some(InfosSearch {
            items: vec![InfoItem {
                name: "Hello".to_string(),
                no_swarm_info: true,
                ..Default::default()
            }],
            ..Default::default()
        })
    });

    {
        let torrents = torrents.clone();
        use_effect_with_deps(
            move |_| {
                let videos = torrents.clone();
                wasm_bindgen_futures::spawn_local(async move {
                    let fetched_videos: InfosSearch = Request::get("/dhtindex/searchInfos?s=ubuntu%20linux")
                        .mode(NoCors)
                        .send()
                        .await
                        .unwrap()
                        .json()
                        .await
                        .unwrap();
                    torrents.set(Some(fetched_videos));
                });
                || ()
            },
            (),
        );
    }

    html! {
        <>
            <h1>{ "DHT search" }</h1>
            <div>
                <h3>{"Torrents"}</h3>
                <TorrentsList torrents={torrents.as_ref().unwrap_or(&Default::default()).items.clone()} />
            </div>
        </>
    }
}

#[derive(Clone, PartialEq, Deserialize, Default)]
#[serde(rename_all="PascalCase")]
struct InfosSearch {
    total: usize,
    err: Option<String>,
    items: Vec<InfoItem>,
}

#[derive(Clone, PartialEq, Deserialize, Default)]
#[serde(rename_all="PascalCase")]
struct InfoItem {
    info_hash: String,
    name: String,
    swarm_info: SwarmInfo,
    size: u64,
    age: String,
    no_swarm_info: bool,
}


#[derive(Clone, PartialEq, Deserialize, Default)]
#[serde(rename_all="PascalCase")]
struct SwarmInfo {
    seeders: u32,
    completed: u32,
    leechers: u32,
}

#[derive(Properties, PartialEq)]
struct TorrentListProps {
    torrents: Vec<InfoItem>,
}
#[function_component(TorrentsList)]
fn torrents_list(TorrentListProps { torrents }: &TorrentListProps) -> Html {
    torrents
        .iter()
        .map(|torrent| {
            html! {
                <p key={torrent.info_hash.clone()}>
                    { torrent.name.clone() }
                </p>
            }
        })
        .collect()
}

fn main() {
    yew::Renderer::<App>::new().render();
}
