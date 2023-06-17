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

pub fn file_types(info_files: &InfoFiles) -> Vec<String> {
    if let [File { path: None, .. }] = info_files.files[..] {
        return Path::new(&info_files.info.name.as_str())
            .extension()
            .and_then(OsStr::to_str)
            .map(str::to_lowercase)
            .into_iter()
            .collect();
    }
    let mut ext_refs = HashMap::<&OsStr, FileLength>::with_capacity(info_files.files.len());
    for file in &info_files.files {
        let ext = match file
            .path
            .as_ref()
            .and_then(|x| x.last())
            .map(Path::new)
            .and_then(Path::extension)
        {
            Some(some) => some,
            None => {
                continue;
            }
        };
        if let Some(existing) = ext_refs.get(ext) {
            if *existing > file.length {
                continue;
            }
        }
        ext_refs.insert(ext, file.length);
    }
    const CARDINALITY: usize = 7;
    let mut owned_exts: HashMap<String, FileLength> = HashMap::with_capacity(ext_refs.len());
    for (ext, length) in ext_refs {
        let Some(ext) = ext.to_str() else { continue };
        let ext = ext.to_lowercase();
        if let Some(cur_length) = owned_exts.get(&ext) {
            if *cur_length >= length {
                continue;
            }
        }
        owned_exts.insert(ext, length);
    }
    let mut best: Vec<(String, FileLength)> = owned_exts.into_iter().collect();
    best.sort_by(|(_, left_len), (_, right_len)| left_len.cmp(right_len).reverse());
    best.into_iter()
        .take(CARDINALITY)
        .map(|(ext, _)| ext)
        .collect()
}

pub fn mount_to_body() {
    ::leptos::mount_to_body(|cx| view! { cx, <App/> })
}

#[derive(Debug, PartialEq, Clone)]
pub(crate) struct FileView {
    pub depth: usize,
    pub name: String,
    pub children: Vec<FileView>,
    pub size: u64,
    pub so: Option<usize>,
    pub expanded: bool,
}

impl FileView {
    pub fn from_file_rows<'a>(file_rows: impl IntoIterator<Item = &'a FileRow> + Copy) -> Self {
        let this_file_row = &FileRow {
            path: vec![],
            dir: true,
            size: None,
            so: None,
        };
        Self::from_file_rows_inner(this_file_row, file_rows)
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
            expanded: false,
            depth: target.path.len(),
        }
    }
}

impl IntoView for FileView {
    fn into_view(self, cx: Scope) -> View {
        let (expanded, set_expanded) = create_signal(cx, self.expanded);
        let name = self.name.clone();
        let name = if self.children.is_empty() {
            name.into_view(cx)
        } else {
            view! { cx,
                <a href="#" on:click=move |_cx| set_expanded(!expanded())>
                    {name}
                </a>
            }
            .into_view(cx)
        };
        let child_rows = {
            let children = self.children.clone();
            move || {
                if expanded() {
                    Some(children.clone().collect_view(cx))
                } else {
                    None
                }
            }
        };
        let row = self;
        let dir = !row.children.is_empty();
        view! { cx,
            <tr>
                <td style:padding-left=format!("{}em", row.depth)>
                    <input type="checkbox" disabled/>
                    <i
                        style:width="1em"
                        style:padding-right="0.5em"
                        class="fa-regular"
                        class:fa-file=move || !dir
                        class:fa-folder=move || dir
                    ></i>
                    {name}
                </td>
                <td>{format_size(row.size)}</td>
            </tr>
            {child_rows}
        }
        .into()
    }
}

fn format_size(size: u64) -> String {
    let mut options = humansize::DECIMAL;
    options.decimal_places = 1;
    humansize::format_size(size, options)
}
