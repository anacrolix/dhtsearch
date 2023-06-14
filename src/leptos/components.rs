use super::*;
use crate::filerow::info_files_to_file_rows;
use ::leptos::html::Input;
use humansize::{format_size, DECIMAL};
use web_sys::SubmitEvent;

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

fn with_cached_info_files<T>(
    cache: ReadSignal<InfoFilesCache>,
    info_hash: ReadSignal<Option<String>>,
    with: impl Fn(&InfoFiles) -> T,
) -> Option<T> {
    cache.with(|cache| {
        info_hash.with(|info_hash| {
            info_hash
                .as_ref()
                .map(|info_hash| match cache.get(info_hash) {
                    Some(Some(Ok(info_files))) => Some(with(info_files)),
                    _ => None,
                })
                .flatten()
        })
    })
}

#[component]
fn InsideRouter(cx: Scope) -> impl IntoView {
    // let search_query = move || use_query_map(cx)().get("s").cloned().unwrap_or_default();
    let (search_query, set_search_query) = create_signal(cx, "".to_owned());
    let torrent_ih = create_rw_signal(cx, None);
    provide_context(cx, torrent_ih.write_only());
    let search_resource: SearchResultResource =
        create_local_resource(cx, search_query, |query| async move {
            if query.is_empty() {
                return Ok(None);
            }
            Ok(Some(search(query).await?))
        });
    let info_files_cache = create_rw_signal(cx, InfoFilesCache::new());
    create_effect(cx, move |attempted: Option<HashSet<_>>| {
        let mut attempted = attempted.unwrap_or_default();
        info!("missing info files effect running");
        let needed = get_needed_info_hashes(cx, torrent_ih(), search_resource);
        let spawn_fetch = move |info_hashes: Vec<_>| {
            if info_hashes.is_empty() {
                return;
            }
            spawn_local(async move {
                fetch_info_files_into_cache(info_files_cache, info_hashes)
                    .await
                    .expect("fetch info files into cache failed")
            })
        };
        info_files_cache.with(|cache| {
            let mut missing = get_missing_info_hashes(cache, needed);
            missing.retain(|info_hash| !attempted.contains(info_hash));
            const FETCH_INDIVIDUALLY: bool = false;
            if FETCH_INDIVIDUALLY {
                for info_hash in missing {
                    assert!(attempted.insert(info_hash.clone()));
                    spawn_fetch(vec![info_hash]);
                }
            } else {
                attempted.extend(missing.clone());
                spawn_fetch(missing);
            }
        });
        attempted
    });
    let file_rows: Signal<Option<Vec<FileRow>>> = create_memo(cx, move |_last| {
        with_cached_info_files(
            info_files_cache.read_only(),
            torrent_ih.read_only(),
            |info_files| {
                debug!("running file rows memo for {}", &info_files.info.info_hash);
                info_files_to_file_rows(&info_files.upverted_files())
            },
        )
    })
    .into();
    let search_view = move || {
        {
            view! { cx,
                <Suspense fallback=move || {
                    view! { cx, <p>{format!("Searching for {:?}...", search_query())}</p> }
                }>
                    <SearchResult
                        herp=search_resource
                        info_files_cache=info_files_cache.read_only()
                        set_torrent_ih=torrent_ih.write_only()
                        search_query=search_query.into()
                    />
                </Suspense>
            }
        }
        .into_view(cx)
    };
    let with_current_info = move || {
        with_cached_info_files(
            info_files_cache.read_only(),
            torrent_ih.read_only(),
            |info_files| info_files.info.clone(),
        )
    };
    let contents_view = move || match torrent_ih() {
        Some(info_hash) => {
            let info = with_current_info.derive_signal(cx);
            view! { cx, <TorrentInfo file_rows info info_hash/> }
        }
        .into_view(cx),
        None => search_view.into_view(cx),
    };
    let set_search_query = move |query| {
        cx.batch(|| {
            torrent_ih.set(None);
            set_search_query(query);
        })
    };
    view! { cx,
        <h1>{"DHT search"}</h1>
        <div class="search-form">
            <SearchForm search_query set_search_query/>
        </div>
        <ErrorBoundary fallback=|cx, errors| {
            view! { cx, <ul>{list_errors(cx, errors)}</ul> }
        }>{contents_view}</ErrorBoundary>
    }
}

#[component]
fn SearchForm<F>(cx: Scope, search_query: ReadSignal<String>, set_search_query: F) -> impl IntoView
where
    F: Fn(String) + 'static,
{
    let input_element: NodeRef<Input> = create_node_ref(cx);
    let on_submit = move |ev: SubmitEvent| {
        ev.prevent_default();
        let value = input_element().expect("<input> to exist").value();
        debug!("on submit search ran with {:?}", &value);
        set_search_query(value);
    };
    view! { cx,
        <form method="GET" action="" class="search-form" on:submit=on_submit>
            <button>
                <i class="fa fa-search"></i>
            </button>
            <input
                style="width: 100%"
                type="search"
                name="s"
                prop:value=search_query
                node_ref=input_element
            />
        </form>
    }
}

#[component]
fn TorrentInfoMetadataItem<K, O, V>(cx: Scope, key: K, value: V) -> impl IntoView
where
    V: Display,
    K: ToOwned<Owned = O>,
    O: IntoView,
{
    view! { cx,
        <tr>
            <td>{key.to_owned()}</td>
            <td>{value.to_string()}</td>
        </tr>
    }
}

#[component]
fn TorrentInfo(
    cx: Scope,
    info: Signal<Option<Info>>,
    file_rows: Signal<Option<Vec<FileRow>>>,
    info_hash: String,
) -> impl IntoView {
    move || {
        let mut magnet_link_view = None;
        let mut metadata_items = vec![];
        info.with(|info| info.as_ref().map(|info|{
            let magnet_link = make_magnet_link(&info.info_hash);
            magnet_link_view = Some(view! { cx,
                    <p>
                        <a href=&magnet_link>
                            <i class="fa fa-magnet"></i>
                            {magnet_link}
                        </a>
                    </p>
                });
            metadata_items
                .push(view! { cx, <TorrentInfoMetadataItem key="Swarm" value=&info.scrape_data/> });
            metadata_items.push(
                view! { cx, <TorrentInfoMetadataItem key="Infohash" value=&info.info_hash/> },
            );
            metadata_items
                .push(view! { cx, <TorrentInfoMetadataItem key="Age" value=&info.age/> });
            metadata_items.push(view! { cx, <TorrentInfoMetadataItem key="Scrape Time" value=&info.scrape_time/> });
        }));
        let files_view = file_rows.with(|file_rows| {
            file_rows
                .as_ref()
                .map(|file_rows| {
                    metadata_items.push(view! { cx, <TorrentInfoMetadataItem key="Num Files" value=file_rows.len()/> });
                    info.with(|info| {
                        info.as_ref().map(|info| {
                            view! { cx, <TorrentFilesNested file_rows=&file_rows info_name=info.name.as_ref()/> }
                            .into_view(cx)
                        })
                    })
                })
                .flatten()
                .unwrap_or_else(|| view! { cx, <p>Loading...</p> }.into_view(cx))
        });
        let metadata_items_view = if metadata_items.is_empty() {
            None
        } else {
            Some(view! { cx, <table>{metadata_items.collect_view(cx)}</table> })
        };
        view! { cx,
            <section class="torrent-info">
            <h3>Torrent Info for {info_hash.clone()}</h3>
            {magnet_link_view}
            {metadata_items_view}
            {files_view}
            </section>
        }
    }
}

#[component]
fn TorrentFilesNested<'a>(
    cx: Scope,
    file_rows: &'a Vec<FileRow>,
    info_name: &'a str,
) -> impl IntoView {
    let mut root_file_view = FileView::from_file_rows(file_rows);
    if let Some(ref mut root) = root_file_view {
        root.name = info_name.to_owned();
        root.expanded = true;
    }
    view! { cx,
        <table class="torrent-files">
            <caption>"Files"</caption>
            {root_file_view}
        </table>
    }
}

#[component]
fn SearchResult(
    cx: Scope,
    herp: SearchResultResource,
    info_files_cache: ReadSignal<InfoFilesCache>,
    set_torrent_ih: WriteSignal<Option<String>>,
    search_query: Signal<String>,
) -> impl IntoView {
    herp.read(cx).map(|ready| match ready {
        Ok(None) => None,
        otherwise => Some(otherwise.map(|ok| {
            ok.map(|some| {
                view! { cx,
                    <h3>{format!("Search results for {:?}", search_query())}</h3>
                    <TorrentsList search_value=some info_files_cache set_torrent_ih/> }
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
    set_torrent_ih: WriteSignal<Option<String>>,
) -> impl IntoView {
    let rows = move || {
        let cache = info_files_cache.get();
        search_value
            .clone()
            .items
            .into_iter()
            .map(|torrent| {
                let info_files: Option<&InfoFiles> = cache
                    .get(&torrent.info_hash)
                    .and_then(|value| value.as_ref().map(|result| result.as_ref().ok()))
                    .flatten();
                let loading =
                    move || view! { cx, <i class="fa fa-spinner fa-spin-pulse"></i> }.into_view(cx);
                let num_files = info_files
                    .as_ref()
                    .map(|info_files| info_files.files.len().into_view(cx))
                    .unwrap_or_else(loading);
                let file_types = info_files
                    .as_ref()
                    .map(|info_files| view_file_types(cx, file_types(info_files)).into_view(cx))
                    .unwrap_or_else(loading);
                let on_click = move |_| {
                    info!("clicked {}", &torrent.info_hash);
                    set_torrent_ih(Some(torrent.info_hash.clone()));
                };
                view! { cx,
                    <tr>
                        <td class="name">
                            <a href="#" on:click=on_click>
                                {torrent.name}
                            </a>
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
