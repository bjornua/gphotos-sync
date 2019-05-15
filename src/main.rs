/*
Goal:
    Empty pictures from sd-card and upload to google images without user interaction

Features:
    Generate list of file to process
    Detect sd-card is inserted
    Delete successfuly uploaded files
    Send notification (email, IM, whatever) when certain events happens
        * Transfer started (label, count, size, eta)
        * Transfer completed (suceeded, failed, size, elapsed, speed, thumbs, errors)
        * SD-Card Inserted
        * Failure
    * Multiple destinations
        * Google
        * FTP / SFTP / SCP

    Checksums?

    Duplicate detection (skip alredy uploaded files)
    Read configuration file from mounted file-system

*/

use std::ffi::OsString;
use std::fs;
use std::io::Write;

fn main() {
    let directory = if let Some(arg) = std::env::args_os().skip(1).next() {
        arg
    } else {
        return;
    };
    let (mut paths, errors) = walk_dir(OsString::from(directory));
    paths.sort();
    let stdout = std::io::stdout();
    let mut stdout_handle = stdout.lock();
    for path in paths {
        stdout_handle.write(path.to_str().unwrap().as_bytes());
        stdout_handle.write(b"\n");
    }
    // println!("{:#?}", paths);
    // println!("{:#?}", errors);
}

#[derive(Debug)]
enum WalkDirError {
    ReadDirError {
        path: OsString,
        cause: std::io::Error,
    },
    ReadDirIterateError {
        path: OsString,
        cause: std::io::Error,
    },
    GetFileTypeError {
        path: OsString,
        cause: std::io::Error,
    },
    NotRegularFile {
        path: OsString,
    },
}

fn walk_dir(root_directory: OsString) -> (Vec<OsString>, Vec<WalkDirError>) {
    let mut errors: Vec<WalkDirError> = Vec::with_capacity(0);
    let mut directories: Vec<OsString> = Vec::with_capacity(20);
    let mut files: Vec<OsString> = Vec::with_capacity(200);

    directories.push(root_directory);
    while let Some(directory) = directories.pop() {
        let paths = match fs::read_dir(&directory) {
            Ok(paths) => paths,
            Err(e) => {
                errors.push(WalkDirError::ReadDirError {
                    path: directory.to_owned(),
                    cause: e,
                });
                continue;
            }
        };
        for path_result in paths {
            let path = match path_result {
                Ok(path) => path,
                Err(e) => {
                    errors.push(WalkDirError::ReadDirIterateError {
                        path: directory.to_owned(),
                        cause: e,
                    });
                    continue;
                }
            };
            let filetype = match path.file_type() {
                Ok(filetype) => filetype,
                Err(e) => {
                    errors.push(WalkDirError::GetFileTypeError {
                        path: directory.to_owned(),
                        cause: e,
                    });
                    continue;
                }
            };
            if filetype.is_dir() {
                directories.push(path.path().into_os_string());
                continue;
            }
            if filetype.is_file() {
                files.push(path.path().into_os_string());
                continue;
            }
            errors.push(WalkDirError::NotRegularFile {
                path: path.path().into_os_string(),
            });
        }
    }

    return (files, errors);
}
