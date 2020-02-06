use crate::config;
// use crate::gauth::Credentials;
// use crate::hash::Hashes;
// use crate::upload;
// use crate::utils::path_matches_ext;
use clap::{App, Arg, ArgMatches, SubCommand};
use notify::Watcher;
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

pub async fn main(matches: &ArgMatches<'_>) {
    if let Err(e) = main_inner(matches).await {
        println!("Error: {:?}", e);
    };
}

async fn main_inner(matches: &ArgMatches<'_>) -> Result<(), MainError> {
    let path = std::path::Path::new(matches.value_of_os("DIRECTORY").unwrap());

    loop {
        // If any part of the watched path is moved, we should reset. Otherwise, the program keep
        // watching the old (moved) file descriptors.
        let watched_path_moved = watch_parent_moves(path);

        let file_changed = watch_file_changes(path);
        let mut cfg = config::load("./gphotos-sync.cbor").map_err(MainError::LoadConfig)?;
        loop {
            match futures::future::select(watched_path_moved, files_changed).await {
                () => {
                    break;
                }
                changed_files => {
                    // Handle files changes. Continue.
                    sync_files(changed_files)?
                }
            };
        }
    }
    return Ok(());
}

async fn watch_file_changes(
    path: &std::path::Path,
) -> impl futures::Stream<Item = std::path::PathBuf> {
    // notify::RecommendedWatcher::new(tx: Sender<Result<Event>>, delay: Duration)
}
async fn watch_parent_moves(path: &std::path::Path) -> impl futures::future::Future<()> {
    // notify::RecommendedWatcher::new(tx: Sender<Result<Event>>, delay: Duration)
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
//     let directory = std::path::Path::new(matches.value_of_os("DIRECTORY").unwrap());

//     let (tx, rx) = channel();
//     watch_parentdir(directory, tx);

//     let mut cfg = config::get("./gphotos-sync.cbor").map_err(MainError::ReadConfiguration)?;

//     return Ok(());
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
