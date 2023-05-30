use super::make_magnet_link;
use crate::api::*;
use humansize::{format_size, DECIMAL};
use leptos::html::Input;
use leptos::*;
use std::sync::Arc;
use web_sys::SubmitEvent;

type SearchErrWrapper<T> = Arc<T>;

fn list_errors(cx: Scope, errors: RwSignal<Errors>) -> impl IntoView {
    errors
        .get()
        .into_iter()
        .map(|(_, e)| view! { cx, <li>{e.to_string()}</li>})
        .collect_view(cx)
}

#[component]
fn App(cx: Scope) -> impl IntoView {
    let (query, set_query) = create_signal(cx, "".to_string());
    let query_input: NodeRef<Input> = create_node_ref(cx);
    let search_resource = create_local_resource(cx, query, |query| async move {
        if query.is_empty() {
            return Ok(None);
        }
        Ok(Some(search(query).await.map_err(SearchErrWrapper::new)?))
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
                    { list_errors(cx, errors) }
                </ul>
            }
        >
            <Suspense fallback=move || view! { cx, <p>"Searching..."</p> }>
                {move || search_resource.read(cx).map(|ready: Result<_, SearchErrWrapper<gloo_net::Error>>| match ready {
                    Ok(None) => None,
                    otherwise => Some(otherwise.map(|ok|ok.map(|some| view! { cx, <TorrentsListLeptos search_value=some/> }))),
                })}
            </Suspense>
        </ErrorBoundary>
    }
}

#[component]
fn TorrentsListLeptos(cx: Scope, search_value: InfosSearch) -> impl IntoView {
    let rows =
        search_value.items
            .into_iter()
            .map(|torrent| view! { cx,
                        <tr>
                            <td class="name"><a href={make_magnet_link(&torrent.name)}>{torrent.name}</a></td>
                            <td>{torrent.swarm_info.seeders}</td>
                            <td>{format_size(torrent.size, DECIMAL)}</td>
                            <td>{torrent.age}</td>
                        </tr>
                    })
            .collect_view(cx);
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
