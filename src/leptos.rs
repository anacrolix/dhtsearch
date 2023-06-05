use super::*;
use crate::api::*;
use ::leptos::*;
use anyhow::anyhow;
use humansize::{format_size, DECIMAL};
use icu_collator::CollatorOptions;
use icu_collator::Numeric::On;
use leptos_router::*;
use log::info;
use std::cmp::Ordering;
use std::collections::{HashMap, HashSet};
use std::ffi::OsStr;
use std::path::Path;

type SearchResultResource = Resource<String, Result<Option<InfosSearch>>>;
type InfoFilesCache = HashMap<String, Option<Result<InfoFiles>>>;

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
        .and_then(Result::ok)
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
    let result = get_info_files(&info_hashes).await;
    cache_signal.update(|cache| match result {
        Ok(payload) => {
            for info_hash in info_hashes {
                cache.insert(
                    info_hash,
                    Some(Err(anyhow!("not included in response").into())),
                );
            }
            for info_files in payload {
                cache.insert(info_files.info.info_hash.clone(), Some(Ok(info_files)));
            }
        }
        Err(err) => {
            for info_hash in info_hashes {
                cache.insert(info_hash, Some(Err(err.clone())));
            }
        }
    });
    Ok(())
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
        <Form method="GET" action="">
            <input type="search" name="s" prop:value=search_query/>
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
                    Some(Some(Ok(info_files))) => Ok(view! { cx,
                        <a href=make_magnet_link(&info_files.info.info_hash)>"magnet link"</a>
                        <pre>{format!("{:#?}", info_files.info)}</pre>
                        <TorrentFiles info_files/>
                    }
                    .into_view(cx)),
                    Some(Some(Err(err))) => Err(err.clone()),
                })
        })
    }
}

#[derive(Eq, Hash, Debug)]
struct FileRow<'a, P>
where
    P: AsRef<str> + Eq,
{
    leaf: &'a str,
    path: &'a [P],
    dir: bool,
    // Later I will show the total size of a directory.
    size: Option<i64>,
}

impl<'b, P, Q> PartialEq<FileRow<'b, Q>> for FileRow<'b, P>
where
    P: Eq + AsRef<str> + PartialEq<Q>,
    Q: Eq + AsRef<str>,
{
    fn eq(&self, other: &FileRow<'b, Q>) -> bool {
        self.leaf == other.leaf && self.path == other.path
    }
}

impl<'a, P> PartialOrd<Self> for FileRow<'a, P>
where
    P: Eq + AsRef<str>,
{
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl<'a, P> Ord for FileRow<'a, P>
where
    P: AsRef<str> + PartialEq + Eq,
{
    fn cmp(&self, other: &Self) -> Ordering {
        let mut options = CollatorOptions::new();
        options.numeric = Some(On);
        let collator = icu_collator::Collator::try_new_unstable(
            &icu_testdata::unstable(),
            &Default::default(),
            options,
        )
        .unwrap();
        let left = &self.path;
        let right = &other.path;
        let l = std::cmp::min(left.len(), right.len());

        // Slice to the loop iteration range to enable bound check
        // elimination in the compiler
        let lhs = &left[..l];
        let rhs = &right[..l];

        for i in 0..l {
            match collator.compare(lhs[i].as_ref(), rhs[i].as_ref()) {
                Ordering::Equal => (),
                non_eq => return non_eq,
            }
        }

        match self.leaf.cmp(other.leaf) {
            Ordering::Equal => (),
            non_eq => return non_eq,
        };

        match left.len().cmp(&right.len()) {
            Ordering::Equal => (),
            non_eq => return non_eq,
        };

        Ordering::Equal
    }
}

fn file_rows(files: &[File]) -> Vec<FileRow<String>> {
    files
        .iter()
        .map(|file| FileRow::<String> {
            leaf: file.path.as_ref().unwrap().last().unwrap(),
            path: file.path.as_ref().unwrap(),
            dir: false,
            size: Some(file.length),
        })
        .collect()
}

fn file_dir_file_rows(file: &File) -> impl IntoIterator<Item = FileRow<String>> {
    file.path
        .iter()
        .filter(|path| path.len() >= 2)
        .flat_map(|parts| {
            (0..parts.len() - 1).map(|leaf| FileRow {
                leaf: &parts[leaf],
                path: &parts[0..leaf],
                dir: true,
                size: None,
            })
        })
}

fn dir_file_rows(files: &[File]) -> Vec<FileRow<String>> {
    let mut ret: Vec<_> = files
        .iter()
        .flat_map(file_dir_file_rows)
        .collect::<HashSet<_>>()
        .into_iter()
        .collect();
    ret.sort();
    ret
}

fn info_files_to_file_rows(info_files: &InfoFiles) -> Vec<FileRow<String>> {
    let mut rows = dir_file_rows(&info_files.files);
    rows.extend(file_rows(&info_files.files));
    rows.sort();
    rows
}

#[component]
fn TorrentFiles<'a>(cx: Scope, info_files: &'a InfoFiles) -> impl IntoView {
    info_files_to_file_rows(info_files).into_iter()
        .map(|row| {
            let leaf = row.leaf.to_owned();
            view! { cx,
                <tr>
                    <td style:padding-left=format!("{}em", row.path.len())>{leaf}</td>
                    <td>{row.size.map(|size| format_size(size as u64, DECIMAL))}</td>
                </tr>
            }
        })
        .collect_view(cx)
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

fn base_file_type<S>(base: &S) -> Option<String>
where
    S: AsRef<OsStr> + ?Sized,
{
    Path::new(base)
        .extension()
        .and_then(OsStr::to_str)
        .map(str::to_lowercase)
}

fn file_path_base(file: &File) -> Option<&str> {
    file.path
        .as_ref()
        .and_then(|parts| parts.last())
        .map(|last| last.as_str())
}

fn file_types(info_files: &InfoFiles) -> Vec<String> {
    if let [File { path: None, .. }] = info_files.files[..] {
        return base_file_type(info_files.info.name.as_str())
            .into_iter()
            .collect();
    }
    let mut files = info_files
        .files
        .iter()
        .filter_map(|file| {
            file_path_base(file)
                .and_then(base_file_type)
                .map(|ext| (file.length, ext))
        })
        .collect::<Vec<_>>();
    files.sort();
    files.reverse();
    let mut seen = HashSet::new();
    files.retain(|(_length, ext)| seen.insert(ext.clone()));
    files.truncate(7);
    files.into_iter().map(|elem| elem.1).collect()
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

#[cfg(test)]
mod tests {
    use super::*;

    fn dir_file_row<'a>(leaf: &'a str, path: &'a [&'a str]) -> FileRow<'a, &'a str> {
        FileRow {
            leaf,
            path,
            dir: true,
            size: None,
        }
    }

    fn same_contents<T: Debug + PartialEq<U> + Ord, U: Debug + Ord>(
        mut got: Vec<T>,
        mut expected: Vec<U>,
    ) {
        got.sort();
        expected.sort();
        assert_eq!(got, expected);
    }

    #[test]
    fn test_dir_file_rows() {
        same_contents(
            dir_file_rows(&vec![File {
                path: Some(
                    vec!["a", "b", "c", "d"]
                        .into_iter()
                        .map(ToOwned::to_owned)
                        .collect(),
                ),
                length: 42,
            }]),
            vec![
                dir_file_row(&"a", &[]),
                dir_file_row(&"b", &["a"]),
                dir_file_row(&"c", &["a", "b"]),
            ],
        );
    }

    #[test]
    fn test_single_file_torrent_file_rows() {

    }
}
