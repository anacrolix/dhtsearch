use super::*;
use crate::api::*;
use ::leptos::*;
use anyhow::anyhow;
use filerow::FileRow;
use leptos_router::*;
use std::collections::{HashMap, HashSet};
use std::ffi::OsStr;
use std::path::Path;

mod components;

use components::App;

type SearchResultResource = Resource<String, Result<Option<InfosSearch>>>;
type InfoFilesCache = HashMap<String, Option<Result<InfoFiles>>>;

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

pub fn mount_to_body() {
    ::leptos::mount_to_body(|cx| view! { cx, <App/> })
}

#[derive(Debug, PartialEq)]
pub(crate) struct FileView {
    pub name: String,
    pub children: Vec<FileView>,
    pub size: u64,
    pub so: Option<usize>,
}

impl FileView {
    pub fn from_file_rows<'a>(
        file_rows: impl IntoIterator<Item = &'a FileRow> + Copy,
    ) -> Option<Self> {
        let this_file_row = &FileRow {
            path: vec![],
            dir: true,
            size: None,
            so: None,
        };
        Some(Self::from_file_rows_inner(this_file_row, file_rows))
    }

    fn from_file_rows_inner<'a>(
        target: &FileRow,
        file_rows: impl IntoIterator<Item = &'a FileRow> + Copy,
    ) -> Self {
        let children = if target.dir {
            let mut children: Vec<FileView> = file_rows
                .into_iter()
                .filter(|file_row: &&FileRow| {
                    let target_len = target.path.len();
                    file_row.path.len() == target_len + 1
                        && target.path.iter().eq(file_row.path.iter().take(target_len))
                })
                .map(|file_row| Self::from_file_rows_inner(file_row, file_rows))
                .collect();
            let collator = new_collator();
            children.sort_by(|left, right| {
                collator
                    .compare(&left.name, &right.name)
                    .then(
                        left.children
                            .is_empty()
                            .cmp(&right.children.is_empty())
                            .reverse(),
                    )
                    .then(left.size.cmp(&right.size).reverse())
            });
            children
        } else {
            vec![]
        };
        FileView {
            name: target.leaf().cloned().unwrap_or_default(),
            so: target.so,
            size: target.size.unwrap_or_default() as u64
                + children.iter().map(|file_view| file_view.size).sum::<u64>(),
            children,
        }
    }
}

impl IntoView for FileView {
    fn into_view(self, cx: Scope) -> View {
        let child_rows = self.children.collect_view(cx);
        view! { cx,
            <tr>
                <td>{self.name}</td>
                <td>{self.size}</td>
            </tr>
            {child_rows}
        }
        .into_view(cx)
    }
}
