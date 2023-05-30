use super::make_magnet_link;
use crate::api::*;
use humansize::{format_size, DECIMAL};
use leptos::html::Input;
use leptos::*;
use std::sync::Arc;
use web_sys::SubmitEvent;

type SearchErrWrapper<T> = Arc<T>;

#[component]
fn App(cx: Scope) -> impl IntoView {
    let (query, set_query) = create_signal(cx, "".to_string());
    let query_input: NodeRef<Input> = create_node_ref(cx);
    let torrents = create_local_resource(cx, query, |query| async move {
        search(query).await.map_err(SearchErrWrapper::new)
    });
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
            <Suspense fallback=move || view! { cx, <p>"Searching..."</p> }>
                <div>
                    <h3>{"Torrents"}</h3>
                    <TorrentsListLeptos search=torrents/>
                </div>
            </Suspense>
        </ErrorBoundary>
    }
}

#[component]
fn TorrentsListLeptos(
    cx: Scope,
    search: Resource<String, Result<InfosSearch, SearchErrWrapper<gloo_net::Error>>>,
) -> impl IntoView {
    let rows = move || {
        search.read(cx).map(|search| {
            search.map(move |ok| {
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
        <table>
            {rows}
        </table>
    }
}

pub fn mount_to_body() {
    leptos::mount_to_body(|cx| view! { cx,  <App/> })
}
