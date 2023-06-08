use super::*;
use crate::api::UpvertedFile;
use std::cmp::Ordering;
use std::collections::HashSet;

#[derive(Eq, Hash, Debug, Ord, PartialOrd, Clone)]
pub struct FileRow {
    pub path: Vec<String>,
    pub dir: bool,
    // Later I will show the total size of a directory.
    pub size: Option<i64>,
    pub so: Option<usize>,
}

impl FileRow {
    pub fn leaf(&self) -> Option<&String> {
        self.path.last()
    }

    pub fn iter_path(&self) -> impl Iterator<Item = &str> {
        self.path.iter().map(|x| x.as_str())
    }

    fn compare_with_collator(&self, other: &Self, collator: &Collator) -> Ordering {
        self.iter_path().cmp_by(other.iter_path(), |left, right| {
            collator.compare(left, right)
        })
    }
}

impl PartialEq for FileRow {
    fn eq(&self, other: &FileRow) -> bool {
        self.path == other.path
    }
}

fn file_rows(files: &[UpvertedFile]) -> Vec<FileRow> {
    files
        .iter()
        .enumerate()
        .map(|(so, file)| FileRow {
            path: file.path.clone(),
            dir: false,
            size: Some(file.length),
            so: Some(so),
        })
        .collect()
}

fn file_dir_file_rows(file: &UpvertedFile) -> impl IntoIterator<Item = FileRow> + '_ {
    let parts = &file.path;
    (1..parts.len()).map(|leaf| FileRow {
        path: parts[0..leaf].to_vec(),
        dir: true,
        size: None,
        so: None,
    })
}

fn dir_file_rows(files: &[UpvertedFile]) -> Vec<FileRow> {
    files
        .iter()
        .flat_map(file_dir_file_rows)
        .collect::<HashSet<_>>()
        .into_iter()
        .collect()
}

pub fn info_files_to_file_rows(upverted: &[UpvertedFile]) -> Vec<FileRow> {
    let mut rows = dir_file_rows(upverted);
    rows.extend(file_rows(upverted));
    let collator = new_collator();
    rows.sort_by(|left, right| left.compare_with_collator(right, &collator));
    rows
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::api::*;
    use crate::leptos::FileView;
    use pretty_assertions::assert_eq;
    use std::iter::once;

    fn dir_file_row<'a>(leaf: &'a str, path: &'a [&'a str]) -> FileRow {
        FileRow {
            path: path
                .iter()
                .cloned()
                .chain(once(leaf))
                .map(|s| s.to_string())
                .collect(),
            dir: true,
            size: None,
            so: None,
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
    fn test_simple_file_rows_and_views() {
        let upverted = InfoFiles {
            info: Info {
                name: "a".to_owned().into(),
                ..Default::default()
            },
            files: vec![
                File {
                    path: Some(
                        vec!["a", "b", "c", "10"]
                            .into_iter()
                            .map(ToOwned::to_owned)
                            .collect(),
                    ),
                    length: 1,
                },
                File {
                    path: Some(
                        vec!["a", "b", "c", "10"]
                            .into_iter()
                            .map(ToOwned::to_owned)
                            .collect(),
                    ),
                    length: 2,
                },
                File {
                    path: Some(
                        vec!["a", "b", "c", "2"]
                            .into_iter()
                            .map(ToOwned::to_owned)
                            .collect(),
                    ),
                    length: 3,
                },
                File {
                    path: Some(vec!["a", "b"].into_iter().map(ToOwned::to_owned).collect()),
                    length: 4,
                },
            ],
        }
        .upverted_files();
        let file_rows = info_files_to_file_rows(&upverted);
        same_contents(
            file_rows.clone(),
            vec![
                dir_file_row(&"a", &[]),
                dir_file_row(&"b", &["a"]),
                dir_file_row(&"c", &["a", "b"]),
                FileRow {
                    path: ["a", "b", "c", "10"]
                        .into_iter()
                        .map(ToOwned::to_owned)
                        .collect(),
                    dir: false,
                    size: Some(1),
                    so: Some(0),
                },
                FileRow {
                    path: ["a", "b", "c", "10"]
                        .into_iter()
                        .map(ToOwned::to_owned)
                        .collect(),
                    dir: false,
                    size: Some(2),
                    so: Some(1),
                },
                FileRow {
                    path: ["a", "b", "c", "2"]
                        .into_iter()
                        .map(ToOwned::to_owned)
                        .collect(),
                    dir: false,
                    size: Some(3),
                    so: Some(2),
                },
                FileRow {
                    path: ["a", "b"].into_iter().map(ToOwned::to_owned).collect(),
                    dir: false,
                    size: Some(4),
                    so: Some(3),
                },
            ],
        );
        let file_view = FileView::from_file_rows(&file_rows);
        assert_eq!(
            file_view,
            Some(FileView {
                name: "".to_string(),
                size: 10,
                so: None,
                children: vec![FileView {
                    name: "a".to_string(),
                    size: 10,
                    so: None,
                    children: vec![
                        FileView {
                            name: "b".to_string(),
                            size: 4,
                            so: Some(3),
                            children: vec![],
                        },
                        FileView {
                            name: "b".to_string(),
                            size: 6,
                            so: None,
                            children: vec![FileView {
                                name: "c".to_string(),
                                size: 6,
                                so: None,
                                children: vec![
                                    FileView {
                                        name: "2".to_string(),
                                        size: 3,
                                        so: Some(2),
                                        children: vec![],
                                    },
                                    FileView {
                                        name: "10".to_string(),
                                        size: 2,
                                        so: Some(1),
                                        children: vec![],
                                    },
                                    FileView {
                                        name: "10".to_string(),
                                        size: 1,
                                        so: Some(0),
                                        children: vec![],
                                    }
                                ],
                            }],
                        }
                    ],
                },]
            })
        )
    }

    #[test]
    fn test_single_file_torrent_file_rows() {
        assert_eq!(
            info_files_to_file_rows(
                &InfoFiles {
                    info: Info {
                        name: "a".to_owned().into(),
                        ..Default::default()
                    },
                    files: vec![File {
                        path: None,
                        ..Default::default()
                    }]
                }
                .upverted_files()
            ),
            vec![FileRow {
                path: vec!["a".to_string()],
                dir: false,
                size: Some(0),
                so: Some(0),
            }]
        )
    }
}
