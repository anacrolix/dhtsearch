use super::make_magnet_link;
use crate::api::*;
use humansize::{format_size, DECIMAL};
use leptos::html::Input;
use leptos::*;
use web_sys::SubmitEvent;

#[component]
fn App(cx: Scope) -> impl IntoView {
    let (query, set_query) = create_signal(cx, "".to_string());
    let query_input: NodeRef<Input> = create_node_ref(cx);
    let torrents = create_local_resource(cx, query, |query| async move { search(query).await });
    let on_query_submit = move |ev: SubmitEvent| {
        ev.prevent_default();
        set_query(query_input().unwrap().value());
    };
    view! { cx,
        <h1>{ "DHT search" }</h1>
        <form on:submit=on_query_submit>
            <input type="text" node_ref=query_input/>
        </form>
        <ErrorBoundary
            fallback=|cx, errors| view! { cx,
                <ul>
                    { move || errors.get().into_iter().map(|(_, e)| view! { cx, <li>{e.to_string()}</li>}).collect_view(cx)}
                </ul>
            }
        >
            <div>
                <h3>{"Torrents"}</h3>
                <TorrentsListLeptos search=torrents/>
            </div>
        </ErrorBoundary>
    }
}

#[component]
fn TorrentsListLeptos(
    cx: Scope,
    search: Resource<String, Result<InfosSearch, gloo_net::Error>>,
) -> impl IntoView {
    let rows = move || {
        search.with(cx, |search| {
            search.as_ref().ok().map(move |ok| {
                ok.items.clone()
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
        })
    };
    view! { cx,
        <Suspense
            fallback=move || view! { cx, <p>"Searching..."</p> }
        >
        {rows}
        </Suspense>
    }
}

pub fn mount_to_body() {
    leptos::mount_to_body(|cx| view! { cx,  <App/> })
}
