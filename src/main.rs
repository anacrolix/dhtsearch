use api::*;
use gloo_net::http::Request;
use leptos::*;
use log::info;
use serde::Deserialize;
use std::ops::Deref;
use url::Url;
use wasm_bindgen::JsCast;
use web_sys::{EventTarget, HtmlInputElement};

mod api;
mod yew;

#[component]
fn App(cx: Scope) -> impl IntoView {
    let (query, set_query) = create_signal(cx, None);
    let (torrents, set_torrents) = create_signal(cx, vec![]);
    view! { cx,
        <h1>{ "DHT search" }</h1>
        <input type="text" on:input=move |ev| {
            set_query(Some(event_target_value(&ev)));
        }/>
        <div>
            <h3>{"Torrents"}</h3>
            <TorrentsListLeptos torrents=torrents/>
        </div>
    }
}

#[component]
fn TorrentsListLeptos(cx: Scope, torrents: ReadSignal<Vec<InfoItem>>) -> impl IntoView {
    let rows = move || {
        torrents()
            .into_iter()
            .map(|torrent| view! { cx, <tr>{torrent.name}</tr>})
            .collect_view(cx)
    };
    view! { cx,
        <table>
            {rows}
        </table>
    }
}

fn main() {
    wasm_logger::init(wasm_logger::Config::default());
    mount_to_body(|cx| view! { cx,  <App/> })
}
