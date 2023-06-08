use super::*;
use crate::filerow::info_files_to_file_rows;
use humansize::{format_size, DECIMAL};

fn list_errors(cx: Scope, errors: RwSignal<Errors>) -> impl IntoView {
    errors
        .get()
        .into_iter()
        .map(|(_, e)| view! { cx, <li>{e.to_string()}</li> })
        .collect_view(cx)
}

#[component]
pub fn App(cx: Scope) -> impl IntoView {
    view! { cx,
        <div class="body">
            <div class="content">
                <Router>
                    <InsideRouter/>
                </Router>
            </div>
        </div>
    }
}

#[component]
fn InsideRouter(cx: Scope) -> impl IntoView {
    let search_query = move || use_query_map(cx)().get("s").cloned().unwrap_or_default();
    let torrent_ih = create_rw_signal(cx, None);
    let search_resource: SearchResultResource =
        create_local_resource(cx, search_query, |query| async move {
            if query.is_empty() {
                return Ok(None);
            }
            Ok(Some(search(query).await?))
        });
    let info_files_cache = create_rw_signal(cx, InfoFilesCache::new());
    create_effect(cx, move |_| {
        info!("missing info files effect running");
        let needed = get_needed_info_hashes(cx, torrent_ih(), search_resource);
        info_files_cache.with(|cache| {
            let missing = get_missing_info_hashes(cache, needed);
            if missing.is_empty() {
                return;
            }
            spawn_local(async move {
                fetch_info_files_into_cache(info_files_cache, missing)
                    .await
                    .expect("fetch info files into cache failed")
            });
        });
    });
    view! { cx,
        <h1>{"DHT search"}</h1>
        <div class="search-form">
            <SearchForm/>
        </div>
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
                                <SearchResult herp=search_resource info_files_cache=info_files_cache.read_only()/>
                            </Suspense>
                        }
                    }
                />
                <Route
                    path="/:ih"
                    view=move |cx| {
                        torrent_ih.set(use_params_map(cx).get().get("ih").cloned());
                        view! { cx,
                            <TorrentInfo
                                info_files_cache=info_files_cache.read_only()
                                info_hash=torrent_ih.derive_signal(cx)
                            />
                        }
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
        <Form method="GET" action="" class="search-form">
            <input style="width: 100%" type="search" name="s" prop:value=search_query/>
            <button>
                <i class="fa fa-search"></i>
            </button>
        </Form>
    }
}

#[component]
fn TorrentInfo(
    cx: Scope,
    info_files_cache: ReadSignal<InfoFilesCache>,
    info_hash: Signal<Option<String>>,
) -> impl IntoView {
    move || {
        info_hash.with(|info_hash| {
            info!("torrent info with {:?}", info_hash);
            info_hash
                .as_ref()
                .map(|info_hash| match info_files_cache().get(info_hash) {
                    None => Ok(view! { cx, <p>"Loading..."</p> }.into_view(cx)),
                    Some(None) => Err(anyhow!("missing ih param").into()),
                    Some(Some(Ok(info_files))) => {
                        let magnet_link = make_magnet_link(&info_files.info.info_hash);
                        Ok(view! { cx,
                            <a href=&magnet_link>
                                <i class="fa fa-magnet"></i>
                                {magnet_link}
                            </a>
                            <pre>{format!("{:#?}", info_files.info)}</pre>
                            <TorrentFiles info_files/>
                        }
                        .into_view(cx))
                    }
                    Some(Some(Err(err))) => Err(err.clone()),
                })
        })
    }
}

#[component]
fn TorrentFiles<'a>(cx: Scope, info_files: &'a InfoFiles) -> impl IntoView {
    let rows = info_files_to_file_rows(&info_files.upverted_files())
        .into_iter()
        .map(|row| {
            let leaf = row.leaf.to_owned();
            view! { cx,
                <tr>
                    <td style:padding-left=format!("{}em", row.path.len())>
                        <input type="checkbox" disabled/>
                        <i
                            style:width="1em"
                            style:padding-right="0.5em"
                            class="fa-regular"
                            class:fa-file=move || !row.dir
                            class:fa-folder=move || row.dir
                        ></i>
                        {leaf}
                    </td>
                    <td>{row.size.map(|size| format_size(size as u64, DECIMAL))}</td>
                </tr>
            }
        })
        .collect_view(cx);
    view! { cx,
        <table>
            <caption>{info_files.files.len()} " files"</caption>
            {rows}
        </table>
    }
}

#[component]
fn SearchResult(
    cx: Scope,
    herp: SearchResultResource,
    info_files_cache: ReadSignal<InfoFilesCache>,
) -> impl IntoView {
    herp.read(cx).map(|ready| match ready {
        Ok(None) => None,
        otherwise => Some(otherwise.map(|ok| {
            ok.map(|some| {
                view! { cx, <TorrentsList search_value=some info_files_cache/> }
            })
        })),
    })
}

fn view_file_types(cx: Scope, file_types: impl IntoIterator<Item = String>) -> impl IntoView {
    file_types
        .into_iter()
        .map(|file_type| view! { cx, <span class="file-type">{file_type}</span> })
        .collect_view(cx)
}

#[component]
fn TorrentsList(
    cx: Scope,
    search_value: InfosSearch,
    info_files_cache: ReadSignal<InfoFilesCache>,
) -> impl IntoView {
    let rows = move || {
        let cache = info_files_cache.get();
        search_value
            .clone()
            .items
            .into_iter()
            .map(|torrent| {
                let info_files = cache
                    .get(&torrent.info_hash)
                    .cloned()
                    .flatten()
                    .and_then(|result| result.ok());
                let loading = move ||
                    view! { cx, <i class="fa fa-spinner fa-spin-pulse"></i> }.into_view(cx);
                let num_files = info_files
                    .as_ref()
                    .map(|info_files| info_files.files.len().into_view(cx))
                    .unwrap_or_else(loading);
                let file_types = info_files
                    .as_ref()
                    .map(|info_files| view_file_types(cx, file_types(info_files)).into_view(cx))
                    .unwrap_or_else(loading);
                view! { cx,
                    <tr>
                        <td class="name">
                            <a href=torrent.info_hash>{torrent.name}</a>
                        </td>
                        <td>{torrent.swarm_info.seeders}</td>
                        <td>{format_size(torrent.size, DECIMAL)}</td>
                        <td>{torrent.age}</td>
                        <td>{num_files}</td>
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
