use crate::api::*;
use leptos::*;

#[component]
fn App(cx: Scope) -> impl IntoView {
    let (_query, set_query) = create_signal(cx, None);
    let (torrents, _set_torrents) = create_signal(cx, vec![]);
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

pub fn mount_to_body() {
    leptos::mount_to_body(|cx| view! { cx,  <App/> })
}
