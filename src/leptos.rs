use super::make_magnet_link;
use super::*;
use crate::api::*;
use ::leptos::*;
use anyhow::anyhow;
use humansize::{format_size, DECIMAL};
use leptos_router::*;
use log::info;
use std::collections::HashMap;
use std::ffi::OsStr;
use std::iter;
use std::path::Path;
use std::result::Result as StdResult;
use std::sync::Arc;

type SearchErrWrapper<T> = Arc<T>;
type CloneableApiError = Arc<Error>;
type SearchResultResource = Resource<String, Result<Option<InfosSearch>>>;
type InfoFilesCache = HashMap<String, InfoFiles>;
type InfoFilesResource = Resource<Option<InfosSearch>, Option<Result<InfoFilesCache>>>;

fn list_errors(cx: Scope, errors: RwSignal<Errors>) -> impl IntoView {
    errors
        .get()
        .into_iter()
        .map(|(_, e)| view! { cx, <li>{e.to_string()}</li> })
        .collect_view(cx)
}

#[component]
fn App(cx: Scope) -> impl IntoView {
    view! { cx,
        <Router>
            <InsideRouter/>
        </Router>
    }
}

fn get_needed_info_hashes(
    cx: Scope,
    torrent_ih: Option<String>,
    search_result: SearchResultResource,
) -> Vec<String> {
    search_result
        .read(cx)
        .map(|result| result.ok())
        .flatten()
        .flatten()
        .unwrap_or_default()
        .items
        .into_iter()
        .map(|item| item.info_hash)
        .chain(torrent_ih.into_iter())
        .collect()
}

fn get_missing_info_hashes(cache: &InfoFilesCache, mut needed: Vec<String>) -> Vec<String> {
    needed.retain(|ih| !cache.contains_key(ih));
    needed
}

async fn fetch_info_files_into_cache(
    cache_signal: RwSignal<InfoFilesCache>,
    info_hashes: Vec<String>,
) -> Result<()> {
    let payload = get_info_files(info_hashes).await?;
    cache_signal.update(|cache| {
        for info_files in payload {
            cache.insert(info_files.info.info_hash.clone(), info_files);
        }
    });
    Ok(())
}

#[component]
fn InsideRouter(cx: Scope) -> impl IntoView {
    let search_query = move || use_query_map(cx)().get("s").cloned().unwrap_or_default();
    let torrent_ih = move || use_params_map(cx).get().get("ih").cloned();
    let search_resource: SearchResultResource =
        create_local_resource(cx, search_query, |query| async move {
            if query.is_empty() {
                return Ok(None);
            }
            Ok(Some(search(query).await?))
        });
    let info_files_cache = create_rw_signal(cx, InfoFilesCache::new());
    create_action(cx, move |()| async move {});
    let info_files_resource: InfoFilesResource = create_local_resource(
        cx,
        move || search_resource.read(cx).and_then(Result::ok).flatten(),
        |search_resource_value: Option<InfosSearch>| async move {
            match search_resource_value {
                Some(infos_search) => Some({
                    let result = get_info_files(
                        infos_search
                            .items
                            .into_iter()
                            .map(|info_item| info_item.info_hash)
                            .collect(),
                    )
                    .await;
                    info!("{:?}", result);
                    result.map(|ok| {
                        ok.into_iter()
                            .map(|info_files| (info_files.info.info_hash.clone(), info_files))
                            .collect::<InfoFilesCache>()
                    })
                }),
                None => None,
            }
        },
    );
    view! { cx,
        <h1>{"DHT search"}</h1>
        <SearchForm/>
        <ErrorBoundary fallback=|cx, errors| {
            view! { cx, <ul>{list_errors(cx, errors)}</ul> }
        }>
            <Routes>
                <Route
                    path="/"
                    view=move |cx| {
                        view! { cx,
                            <Suspense fallback=move || {
                                view! { cx, <p>"Searching..."</p> }
                            }>
                                <SearchResult herp=search_resource info_files_resource/>
                            </Suspense>
                        }
                    }
                />
                <Route
                    path="/:ih"
                    view=move |cx| {
                        view! { cx, <TorrentInfo/> }
                    }
                />
            </Routes>
        </ErrorBoundary>
    }
}

#[component]
fn SearchForm(cx: Scope) -> impl IntoView {
    let search_query = move || use_query_map(cx)().get("s").cloned().unwrap_or_default();
    view! { cx,
        <Form method="GET" action="">
            <input type="search" name="s" prop:value=search_query/>
        </Form>
    }
}

#[component]
fn TorrentInfo(cx: Scope) -> impl IntoView {
    let info_files = create_local_resource(
        cx,
        move || use_params_map(cx).get().get("ih").cloned(),
        |info_hash| async move {
            match info_hash {
                Some(info_hash) => Some(match get_info_files(vec![info_hash.clone()]).await {
                    Ok(payload) if !payload.is_empty() => Ok(payload.into_iter().next().unwrap()),
                    Ok(_) => Err(Arc::new(anyhow!("unknown infohash").into())),
                    Err(err) => Err(err),
                }),
                None => None,
            }
        },
    );
    move || match info_files.read(cx) {
        None => Ok(view! { cx, <p>"Loading..."</p> }.into_view(cx)),
        Some(None) => Err(Arc::new(Error::Anyhow(anyhow!("missing ih param")))),
        Some(Some(Ok(info_files))) => Ok(view! { cx,
            <a href=make_magnet_link(&info_files.info.info_hash)>"magnet link"</a>
            <pre>{format!("{:#?}", info_files)}</pre>
        }
        .into_view(cx)),
        Some(Some(Err(err))) => Err(err),
    }
}

#[component]
fn SearchResult(
    cx: Scope,
    herp: SearchResultResource,
    info_files_resource: InfoFilesResource,
) -> impl IntoView {
    herp.read(cx).map(|ready| match ready {
        Ok(None) => None,
        otherwise => Some(otherwise.map(|ok| {
            ok.map(|some| {
                view! { cx, <TorrentsList search_value=some info_files=info_files_resource/> }
            })
        })),
    })
}

fn base_file_type<S>(base: &S) -> Option<&str>
where
    S: AsRef<OsStr> + ?Sized,
{
    Path::new(base).extension().and_then(OsStr::to_str)
}

fn file_type(file: &File) -> Option<&str> {
    file.path
        .as_ref()
        .and_then(|parts| parts.last())
        .and_then(|base| base_file_type(base))
}

fn file_types(info_files: &InfoFiles) -> Vec<&str> {
    let mut all: Vec<_> = iter::once(base_file_type(info_files.info.name.as_str()))
        .chain(info_files.files.iter().map(file_type))
        .flatten()
        .take(7)
        .collect();
    all.sort();
    all.dedup();
    all
}

fn view_file_types(cx: Scope, file_types: Vec<&str>) -> impl IntoView {
    file_types
        .into_iter()
        .map(|file_type| view! { cx, <span class="badge">{file_type.to_owned()}</span> })
        .collect_view(cx)
}

#[component]
fn TorrentsList(
    cx: Scope,
    search_value: InfosSearch,
    info_files: Resource<Option<InfosSearch>, Option<StdResult<InfoFilesCache, CloneableApiError>>>,
) -> impl IntoView {
    let rows = {
        let cache: InfoFilesCache = info_files
            .read(cx)
            .flatten()
            .and_then(Result::ok)
            .unwrap_or_default();
        search_value
            .items
            .into_iter()
            .map(|torrent| {
                let info_files = cache.get(&torrent.info_hash);
                let file_types = info_files
                    .as_ref()
                    .map(|info_files| view_file_types(cx, file_types(info_files)));
                view! { cx,
                    <tr>
                        <td class="name">
                            <a href=torrent.info_hash>{torrent.name}</a>
                        </td>
                        <td>{torrent.swarm_info.seeders}</td>
                        <td>{format_size(torrent.size, DECIMAL)}</td>
                        <td>{torrent.age}</td>
                        <td>{info_files.as_ref().map(|info_files| info_files.files.len())}</td>
                        <td>{file_types}</td>
                    </tr>
                }
            })
            .collect_view(cx)
    };
    view! { cx,
        <table>
            <tr>
                <th>"Name"</th>
                <th>"Seeders"</th>
                <th>"Size"</th>
                <th>"Age"</th>
                <th>"Files"</th>
                <th>"File Types"</th>
            </tr>
            {rows}
        </table>
    }
}

pub fn mount_to_body() {
    ::leptos::mount_to_body(|cx| view! { cx, <App/> })
}
