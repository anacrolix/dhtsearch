use crate::api::*;
use leptos::*;

#[component]
fn App(cx: Scope) -> impl IntoView {
    let (query, set_query) = create_signal(cx, None);
    let torrents = create_local_resource(
        cx,
        query,
        |query| async move { search(query.unwrap()).await },
    );
    view! { cx,
        <h1>{ "DHT search" }</h1>
        <input type="text" on:input=move |ev| {
            set_query(Some(event_target_value(&ev)));
        }/>
        <div>
            <h3>{"Torrents"}</h3>
            <TorrentsListLeptos search={Signal::derive(cx, move || torrents.read(cx))}/>
        </div>
    }
}

#[component]
fn TorrentsListLeptos(cx: Scope, search: Signal<Option<InfosSearch>>) -> impl IntoView {
    let rows = move || {
        search()
            .map(|search| {
                search
                    .items
                    .into_iter()
                    .map(|torrent| view! { cx, <tr>{torrent.name}</tr>}).collect_view(cx)
            })
            .unwrap_or_default()
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
