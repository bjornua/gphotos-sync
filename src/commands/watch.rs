use crate::config;
// use crate::gauth::Credentials;
// use crate::hash::Hashes;
// use crate::upload;
// use crate::utils::path_matches_ext;
use clap::{App, Arg, ArgMatches, SubCommand};
use notify::{self, Watcher};

use core::future::Future;
use futures::future;
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
}

pub async fn command(matches: &ArgMatches<'_>) {
    let path = std::path::Path::new(matches.value_of_os("DIRECTORY").unwrap());

    if let Err(e) = main_loop(path).await {
        println!("Error: {:?}", e);
    };
}

async fn main_loop(path: &std::path::Path) -> Result<(), MainError> {
    loop {
        let mut cfg_path = path.to_owned();
        cfg_path.push("gphotos-sync.cbor");
        let cfg = config::load(cfg_path).map_err(MainError::LoadConfig)?;
        let mut root_moved = watch_path_moved(path);
        let mut file_changed_watcher = watch_file_changes(path).unwrap();
        let mut chunked_file_changed_watcher = file_changed_watcher.rx.ready_chunks(5);

        let mut file_changed = chunked_file_changed_watcher.next();
        loop {
            match future::select(root_moved, file_changed).await {
                future::Either::Left(((), _)) => {
                    println!("Directory moved restarting");
                    break;
                }
                future::Either::Right((None, _)) => {
                    println!("File watcher ended, restarting");
                    break;
                }

                future::Either::Right((Some(changed_files), root_moved_back)) => {
                    println!("File moved");
                    root_moved = root_moved_back;
                    sync_files(&cfg, changed_files).await;
                    file_changed = chunked_file_changed_watcher.next();
                }
            };
        }
    }
}

#[derive(Debug)]
enum WatchFileChangesError {
    CreateWatcherError(notify::Error),
    WatchDir(notify::Error),
}

struct FileWatcher {
    watcher: notify::RecommendedWatcher,
    rx: tokio::sync::mpsc::Receiver<std::path::PathBuf>,
}

fn watch_file_changes(path: &std::path::Path) -> Result<FileWatcher, WatchFileChangesError> {
    let (tx, rx) = tokio::sync::mpsc::channel::<std::path::PathBuf>(5);
    let tx_rc = std::sync::Arc::new(tokio::sync::Mutex::new(tx));

    let mut watcher: notify::RecommendedWatcher = notify::Watcher::new_immediate(
        move |res: Result<notify::Event, notify::Error>| match res {
            Ok(event) => {
                let tx_rc = std::sync::Arc::clone(&tx_rc);

                // println!("event: {:?}", event);
                let path = event.paths.first().unwrap().clone();
                tokio::spawn(async move { tx_rc.lock().await.send(path).await });
            }
            Err(e) => println!("watch error: {:?}", e),
        },
    )
    .map_err(WatchFileChangesError::CreateWatcherError)
    .unwrap();
    watcher
        .watch(path, notify::RecursiveMode::Recursive)
        .map_err(WatchFileChangesError::WatchDir)
        .unwrap();

    // futures::stream::once(Box::pin(async { std::path::PathBuf::new() }))
    // return futures::stream::repeat(path.to_path_buf());

    return Ok(FileWatcher {
        watcher: watcher,
        rx: rx,
    });
}

// If any part of the watched path is moved, we should reset. Otherwise, the program keep
// For instance for the path
// watching the old (moved) file descriptors.
fn watch_path_moved(path: &std::path::Path) -> impl Future<Output = ()> {
    // notify::RecommendedWatcher::new(tx: Sender<Result<Event>>, delay: Duration)
    futures::future::pending()
}

fn sync_files(cfg: &config::Config, path: Vec<std::path::PathBuf>) -> impl Future<Output = ()> {
    println!("Syncing files path {:?}", path);
    async { () }
}

// New plan:
// Use crossbeam
//
// watch (maindir)
// -> queue: changes in main dir
// -> queue: changes in parentdirs
// -> race:
//     -> queue.recv
//     -> watch_path_moved
//       -> race:
//         -> watch ../
//         -> watch ../../
//         -> watch ../../../
//   -> if change in parentdir, restart
//   -> if changes in main dir, check and upload

// async fn main_inner(matches: &ArgMatches<'_>) -> Result<(), MainError> {
//     let directory = std::path::P()
// }

// async fn watch_parentdir(path: &std::path::Path, tx: crossbeam_channel::Sender<()>) {}

// enum SyncDirError {
//     CreateWatch(notify::Error),
//     StartWatch(notify::Error),
//     UploadError(upload::UploadError),
//     SaveConfig(config::SaveError),
// }

// async fn sync_dir(path: &std::path::Path, cfg: &mut config::Config) -> Result<(), SyncDirError> {
//     let (tx, rx) = channel();
//     let mut watcher = notify::RecommendedWatcher::new(tx, std::time::Duration::from_secs(2))
//         .map_err(SyncDirError::CreateWatch)?;

//     watcher
//         .watch(path, notify::RecursiveMode::Recursive)
//         .map_err(SyncDirError::StartWatch)?;

//     loop {
//         match rx.recv() {
//             Ok(Ok(event)) => {
//                 handle_event(&mut cfg.credentials, &mut cfg.uploaded_files, event)
//                     .await
//                     .map_err(SyncDirError::UploadError)?;
//                 config::save("./gphotos-sync.cbor", &cfg).map_err(SyncDirError::SaveConfig)?;
//             }
//             Ok(Err(err)) => {
//                 println!("Watch error: {:?}", err);
//             }
//             Err(err) => {
//                 println!("Channel error: {:?}", err);
//             }
//         }
//     }
// }

// async fn handle_event(
//     credentials: &mut Credentials,
//     uploaded_files: &mut Hashes,
//     event: notify::event::Event,
// ) -> Result<(), upload::UploadError> {
//     use notify::event::{CreateKind, EventKind, ModifyKind};
//     match event.kind {
//         EventKind::Create(CreateKind::File)
//         | EventKind::Create(CreateKind::Any)
//         | EventKind::Modify(ModifyKind::Data(_))
//         | EventKind::Modify(ModifyKind::Any) => {
//             println!("Processing {:?}", event.paths);
//             let paths = event.paths.into_iter().filter(path_matches_ext);
//             upload::upload_many(credentials, uploaded_files, paths).await
//         }
//         _ => Ok(()),
//     }
// }
