use super::*;
use crate::api::UpvertedFile;
use std::cmp::Ordering;
use std::collections::HashSet;

#[derive(Eq, Hash, Debug, Ord, PartialOrd)]
pub struct FileRow<'a, P>
where
    P: AsRef<str> + Eq,
{
    pub leaf: &'a str,
    pub path: &'a [P],
    pub dir: bool,
    // Later I will show the total size of a directory.
    pub size: Option<i64>,
}

impl<'a, P> FileRow<'a, P>
where
    P: AsRef<str> + Eq,
{
    fn iter_path(&self) -> impl Iterator<Item = &str> {
        self.path
            .iter()
            .map(|x| x.as_ref())
            .chain(std::iter::once(self.leaf))
    }

    fn compare_with_collator(&self, other: &Self, collator: &Collator) -> Ordering {
        self.iter_path().cmp_by(other.iter_path(), |left, right| {
            collator.compare(left, right)
        })
    }
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

fn file_rows(files: &[UpvertedFile]) -> Vec<FileRow<String>> {
    files
        .iter()
        .map(|file| {
            let (leaf, path) = file.path.split_last().unwrap();
            FileRow::<String> {
                leaf,
                path,
                dir: false,
                size: Some(file.length),
            }
        })
        .collect()
}

fn file_dir_file_rows(file: &UpvertedFile) -> impl IntoIterator<Item = FileRow<String>> {
    let parts = &file.path;
    (0..parts.len() - 1).map(|leaf| FileRow {
        leaf: &parts[leaf],
        path: &parts[0..leaf],
        dir: true,
        size: None,
    })
}

fn dir_file_rows(files: &[UpvertedFile]) -> Vec<FileRow<String>> {
    files
        .iter()
        .flat_map(file_dir_file_rows)
        .collect::<HashSet<_>>()
        .into_iter()
        .collect()
}

pub fn info_files_to_file_rows(upverted: &[UpvertedFile]) -> Vec<FileRow<String>> {
    let mut rows = dir_file_rows(upverted);
    rows.extend(file_rows(&upverted));
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
