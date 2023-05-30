use crate::api::*;
use leptos::html::Input;
use leptos::*;
use web_sys::SubmitEvent;
use humansize::{format_size, DECIMAL};
use super::make_magnet_link;

#[component]
fn App(cx: Scope) -> impl IntoView {
    let (query, set_query) = create_signal(cx, None);
    let query_input: NodeRef<Input> = create_node_ref(cx);
    let torrents = create_local_resource(
        cx,
        query,
        |query| async move { search(query.unwrap()).await },
    );
    let on_query_submit = move |ev: SubmitEvent| {
        ev.prevent_default();
        set_query(query_input().map(|input| input.value()));
    };
    view! { cx,
        <h1>{ "DHT search" }</h1>
        <form on:submit=on_query_submit>
            <input type="text" node_ref=query_input/>
        </form>
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
                    .map(|torrent| view! { cx,
                        <tr>
                            <td><a href={make_magnet_link(&torrent.name)}>{torrent.name}</a></td>
                            <td>{torrent.swarm_info.seeders}</td>
                            <td>{format_size(torrent.size, DECIMAL)}</td>
                            <td>{torrent.age}</td>
                        </tr>
                    })
                    .collect_view(cx)
            })
            .unwrap_or_default()
    };
    view! { cx,
        <table>
            <tr>
                <th>"Name"</th>
                <th>"Seeders"</th>
                <th>"Size"</th>
                <th>"Age"</th>
            </tr>
            {rows}
        </table>
    }
}

pub fn mount_to_body() {
    leptos::mount_to_body(|cx| view! { cx,  <App/> })
}
