use super::*;
use crate::api::UpvertedFile;
use std::cmp::Ordering;
use std::collections::HashSet;

#[derive(Eq, Hash, Debug, Ord, PartialOrd, Clone)]
pub struct FileRow {
    pub leaf: String,
    pub path: Vec<String>,
    pub dir: bool,
    // Later I will show the total size of a directory.
    pub size: Option<i64>,
}

impl FileRow {
    fn iter_path(&self) -> impl Iterator<Item = &str> {
        self.path
            .iter()
            .map(|x| x.as_str())
            .chain(std::iter::once(self.leaf.as_str()))
    }

    fn compare_with_collator(&self, other: &Self, collator: &Collator) -> Ordering {
        self.iter_path().cmp_by(other.iter_path(), |left, right| {
            collator.compare(left, right)
        })
    }
}

impl PartialEq for FileRow {
    fn eq(&self, other: &FileRow) -> bool {
        self.leaf == other.leaf && self.path == other.path
    }
}

fn file_rows(files: &[UpvertedFile]) -> Vec<FileRow> {
    files
        .iter()
        .map(|file| {
            let (leaf, path) = file.path.split_last().unwrap();
            let leaf = leaf.clone();
            let path = path.to_vec();
            FileRow {
                leaf,
                path,
                dir: false,
                size: Some(file.length),
            }
        })
        .collect()
}

fn file_dir_file_rows(file: &UpvertedFile) -> impl IntoIterator<Item = FileRow> + '_ {
    let parts = &file.path;
    (0..parts.len() - 1).map(|leaf| FileRow {
        leaf: parts[leaf].clone(),
        path: parts[0..leaf].to_vec(),
        dir: true,
        size: None,
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
        let upverted = InfoFiles {
            info: Info {
                name: "a".to_owned().into(),
                ..Default::default()
            },
            files: vec![File {
                path: Some(
                    vec!["a", "b", "c", "d"]
                        .into_iter()
                        .map(ToOwned::to_owned)
                        .collect(),
                ),
                length: 42,
            }],
        }
        .upverted_files();
        same_contents(
            dir_file_rows(&upverted),
            vec![
                dir_file_row(&"a", &[]),
                dir_file_row(&"b", &["a"]),
                dir_file_row(&"c", &["a", "b"]),
            ],
        );
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
            vec![FileRow::<String> {
                leaf: &"a",
                path: &[],
                dir: false,
                size: Some(0),
            }]
        )
    }
}
