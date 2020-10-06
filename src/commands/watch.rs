use crate::lib::config;
use crate::lib::fswatcher::{self, FSWatcher};
// use crate::gauth::Credentials;
// use crate::hash::Hashes;
// use crate::upload;
// use crate::utils::path_matches_ext;
use clap::{App, Arg, ArgMatches, SubCommand};

use core::future::Future;
use futures::StreamExt;

pub fn get_subcommand() -> App<'static, 'static> {
    SubCommand::with_name("watch")
        .about("Watches a folder for new images and uploads them to Google Photos")
        .arg(
            Arg::with_name("DIRECTORY")
                .help("The directory to watch for changes")
                .index(1)
                .required(true)
                .multiple(false),
        )
}

#[derive(Debug)]
enum MainError {
    LoadConfig(config::LoadError),
    WatchFilesError(WatchFilesError),
    CanonicalizeError(std::io::Error),
}

pub async fn command(matches: &ArgMatches<'_>) {
    let path = std::path::Path::new(matches.value_of_os("DIRECTORY").unwrap());

    if let Err(e) = main_loop(path).await {
        println!("Error: {:?}", e);
    };
}

async fn main_loop(path: &std::path::Path) -> Result<(), MainError> {
    let path = path.canonicalize().map_err(MainError::CanonicalizeError)?;

    loop {
        let mut cfg_path = path.to_owned();
        cfg_path.push("gphotos-sync.cbor");
        let cfg = config::load(cfg_path).map_err(MainError::LoadConfig)?;
        let mut watcher = create_watcher(&path).map_err(MainError::WatchFilesError)?;

        while let Some(event) = watcher.next().await {
            match event {
                fswatcher::Event::FileModified(file_path) => {
                    if file_path.starts_with(&path) {
                        sync_files(&cfg, file_path).await;
                    } else {
                        println!("Other file changed, {:?}", file_path);
                    }
                }
                fswatcher::Event::PathMoved(folder_path) => {
                    if path.starts_with(&folder_path) {
                        println!("Ancestor path ({:?}) moved, restarting", folder_path);
                        break;
                    } else {
                        println!("Path moved, {:?}", folder_path);
                    }
                }
            };
        }
    }
}

#[derive(Debug)]
enum WatchFilesError {
    CreateWatcherError(fswatcher::Error),
    StartWatchError(fswatcher::Error),
}

fn create_watcher(path: &std::path::Path) -> Result<FSWatcher, WatchFilesError> {
    let mut watcher = FSWatcher::new().map_err(WatchFilesError::CreateWatcherError)?;

    watcher
        .watch(path, notify::RecursiveMode::Recursive)
        .map_err(WatchFilesError::StartWatchError)?;

    for parent in path.ancestors() {
        watcher
            .watch(parent, notify::RecursiveMode::NonRecursive)
            .map_err(WatchFilesError::StartWatchError)?;
    }

    return Ok(watcher);
}

fn sync_files(_cfg: &config::Config, path: std::path::PathBuf) -> impl Future<Output = ()> {
    println!("Syncing files path {:?}", path);
    async { () }
}
