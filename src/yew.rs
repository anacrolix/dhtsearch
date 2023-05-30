use crate::api::*;
use crate::make_magnet_link;
use log::info;
use std::ops::Deref;
use wasm_bindgen::JsCast;
use web_sys::{EventTarget, HtmlInputElement};
use yew::prelude::*;

#[derive(PartialEq, Clone, Properties)]
struct AppState {
    search_result: Option<InfosSearch>,
    query: Option<String>,
}

#[function_component(App)]
fn app() -> Html {
    let state = use_state_eq(|| AppState {
        search_result: Some(InfosSearch {
            items: vec![InfoItem {
                name: "Hello".to_string(),
                no_swarm_info: true,
                ..Default::default()
            }],
            ..Default::default()
        }),
        query: None,
    });

    let on_search = {
        let state = state.clone();
        Callback::from(move |query: String| {
            info!("query changed: {}", query);
            state.set(AppState {
                query: Some(query.clone()),
                ..state.deref().clone()
            });
            let state = state.clone();
            wasm_bindgen_futures::spawn_local(async move {
                let fetched_videos = search(query).await;
                let mut app_state = state.deref().clone();
                app_state.search_result = Some(fetched_videos);
                state.set(app_state);
            });
        })
    };

    html! {
        <>
            <h1>{ "DHT search" }</h1>
            <SearchForm on_search={on_search}/>
            <div>
                <h3>{"Torrents"}</h3>
                <TorrentsList torrents={
                    state.search_result.as_ref().unwrap_or(&Default::default()).items.clone()
                }/>
            </div>
        </>
    }
}

#[derive(Properties, PartialEq)]
struct TorrentListProps {
    torrents: Vec<InfoItem>,
}
#[function_component(TorrentsList)]
fn torrents_list(TorrentListProps { torrents }: &TorrentListProps) -> Html {
    let rows: Vec<Html> = torrents
        .iter()
        .map(|torrent| {
            let magnet_link = make_magnet_link(&torrent.info_hash);
            html! {
                <tr key={torrent.info_hash.clone()}>
                    <td><a href={magnet_link}>{ torrent.name.clone() }</a></td>
                    <td>{ torrent.swarm_info.seeders }</td>
                </tr>
            }
        })
        .collect();
    html! {
        <table>{ rows }</table>
    }
}

#[derive(PartialEq, Properties)]
pub struct SearchFormProps {
    on_search: Callback<String>,
}

#[function_component]
pub fn SearchForm(SearchFormProps { on_search }: &SearchFormProps) -> Html {
    let on_search = on_search.clone();
    let on_change = Callback::from(move |e: Event| {
        // When events are created the target is undefined, it's only
        // when dispatched does the target get added.
        let target: Option<EventTarget> = e.target();
        // Events can bubble so this listener might catch events from child
        // elements which are not of type HtmlInputElement
        let input = target.and_then(|t| t.dyn_into::<HtmlInputElement>().ok());
        if let Some(input) = input {
            on_search.emit(input.value());
        }
    });
    html! {
        <input onchange={on_change}/>
    }
}

pub fn mount_to_body() {
    yew::Renderer::<App>::new().render();
}
