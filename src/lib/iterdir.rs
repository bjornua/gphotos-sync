use crate::lib::utils::path_matches_ext;
use std::ffi::OsString;
use std::fs::{read_dir, DirEntry, ReadDir};

#[derive(Debug)]
pub enum IterDirError {
    ReadDirError {
        directory_path: OsString,
        cause: std::io::Error,
    },
    ReadDirIterateError {
        directory_path: OsString,
        cause: std::io::Error,
    },
    GetFileTypeError {
        dir_entry: DirEntry,
        cause: std::io::Error,
    },
    NotRegularFile {
        dir_entry: DirEntry,
    },
}

pub struct RecursiveIterDir {
    directories: Vec<OsString>,
    cur: Option<(OsString, ReadDir)>,
}
impl RecursiveIterDir {
    pub fn new(root_directory: OsString) -> Self {
        Self {
            directories: vec![root_directory],
            cur: None,
        }
    }
}
impl Iterator for RecursiveIterDir {
    type Item = Result<DirEntry, IterDirError>;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            let (dir_path, dir_iter) = match self.cur {
                Some(ref mut x) => x,
                None => {
                    let dir = self.directories.pop()?;
                    match read_dir(&dir) {
                        Ok(dir_iter) => {
                            self.cur.replace((dir, dir_iter));
                            self.cur.as_mut().unwrap()
                        }
                        Err(e) => {
                            return Some(Err(IterDirError::ReadDirError {
                                cause: e,
                                directory_path: dir,
                            }));
                        }
                    }
                }
            };
            let dir_entry = match dir_iter.next() {
                Some(Ok(dir_entry)) => dir_entry,
                Some(Err(e)) => {
                    return Some(Err(IterDirError::ReadDirIterateError {
                        cause: e,
                        directory_path: dir_path.to_owned(),
                    }));
                }
                None => {
                    // Current directory finished
                    self.cur = None;
                    continue;
                }
            };

            // let path = dir_entry.path().into_os_string();
            match dir_entry.file_type() {
                Ok(file_type) if file_type.is_dir() => {
                    self.directories.push(dir_entry.path().into_os_string());
                    continue;
                }
                Ok(file_type) if file_type.is_file() => return Some(Ok(dir_entry)),
                Ok(_) => return Some(Err(IterDirError::NotRegularFile { dir_entry })),
                Err(e) => {
                    return Some(Err(IterDirError::GetFileTypeError {
                        dir_entry,
                        cause: e,
                    }));
                }
            }
        }
    }
}

#[derive(Debug)]
pub enum FindFilesError {
    WrongExtension { path: std::path::PathBuf },
    IterDirError(IterDirError),
}

pub fn findfiles_with_ext(
    directory: OsString,
) -> impl Iterator<Item = Result<std::path::PathBuf, FindFilesError>> {
    let files = RecursiveIterDir::new(directory);

    let files = files.map(move |file| {
        let dir_entry = match file {
            Ok(direntry) => direntry,
            Err(e) => return Err(FindFilesError::IterDirError(e)),
        };
        let path = dir_entry.path();
        if !path_matches_ext(&path) {
            return Err(FindFilesError::WrongExtension { path });
        };

        return Ok(path);
    });
    files
}
