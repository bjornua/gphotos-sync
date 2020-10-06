use futures::stream::{Stream, StreamExt};
use notify::{
    event::{CreateKind, EventKind, ModifyKind, RenameMode},
    RecommendedWatcher, Watcher,
};
use std::path::PathBuf;
use std::task::{Context, Poll};
use tokio::sync::{
    self,
    mpsc::{self, Receiver},
};

#[derive(Debug)]
pub enum Error {
    CreateWatcherError(notify::Error),
    StartWatch(notify::Error),
}

pub type WatchMode = notify::RecursiveMode;

pub struct FSWatcher {
    watcher: RecommendedWatcher,
    rx_folder: Receiver<Event>,
    rx_file: Receiver<Event>,
}
#[derive(Debug)]
pub enum Event {
    FileModified(PathBuf),
    PathMoved(PathBuf),
}

fn simplify_event(event: notify::Event) -> impl Iterator<Item = Event> {
    let notify::Event {
        attrs: _,
        paths,
        kind,
    } = event;

    return paths.into_iter().filter_map(move |p| {
        return match kind {
            EventKind::Modify(ModifyKind::Data(_)) | EventKind::Create(CreateKind::File) => {
                Some(Event::FileModified(p))
            }
            EventKind::Modify(ModifyKind::Name(RenameMode::From))
            | EventKind::Modify(ModifyKind::Name(RenameMode::To))
            | EventKind::Modify(ModifyKind::Name(RenameMode::Both)) => Some(Event::PathMoved(p)),
            _ => None,
        };
    });
}

impl Stream for FSWatcher {
    type Item = Event;

    fn poll_next(
        mut self: std::pin::Pin<&mut Self>,
        cx: &mut Context<'_>,
    ) -> Poll<Option<Self::Item>> {
        if let folder_ready @ Poll::Ready(_) = self.rx_folder.poll_next_unpin(cx) {
            return folder_ready;
        }
        return self.rx_file.poll_next_unpin(cx);
    }
}

impl FSWatcher {
    pub fn new() -> Result<FSWatcher, Error> {
        let (tx_file, rx_file) = mpsc::channel::<Event>(5);
        let (tx_folder, rx_folder) = mpsc::channel::<Event>(5);

        let tx_file = std::sync::Arc::new(sync::Mutex::new(tx_file));
        let tx_folder = std::sync::Arc::new(sync::Mutex::new(tx_folder));

        let watcher: RecommendedWatcher =
            notify::Watcher::new_immediate(move |res: Result<notify::Event, notify::Error>| {
                match res {
                    Ok(event) => {
                        let mut runtime = tokio::runtime::Builder::new()
                            .basic_scheduler()
                            .build()
                            .unwrap();
                        runtime.block_on(async {
                            for event in simplify_event(event) {
                                match event {
                                    e @ Event::FileModified(_) => {
                                        let tx_file = std::sync::Arc::clone(&tx_file);
                                        tx_file.lock().await.send(e).await.unwrap();
                                    }
                                    e @ Event::PathMoved(_) => {
                                        let tx_folder = std::sync::Arc::clone(&tx_folder);
                                        tx_folder.lock().await.send(e).await.unwrap();
                                    }
                                }
                            }
                        });
                    }
                    Err(e) => println!("watch error: {:?}", e),
                }
            })
            .map_err(Error::CreateWatcherError)?;

        return Ok(FSWatcher {
            watcher,
            rx_file,
            rx_folder,
        });
    }

    pub fn watch(&mut self, path: &std::path::Path, watch_mode: WatchMode) -> Result<(), Error> {
        self.watcher
            .watch(path, watch_mode)
            .map_err(Error::StartWatch)?;
        return Ok(());
    }
}
