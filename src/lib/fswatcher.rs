use futures::stream::Stream;
use notify::{
    event::{CreateKind, EventKind, ModifyKind},
    RecommendedWatcher, Watcher,
};
use std::task::{Context, Poll};
use tokio::sync;

#[derive(Debug)]
pub enum Error {
    CreateWatcherError(notify::Error),
    StartWatch(notify::Error),
}

pub type WatchMode = notify::RecursiveMode;

pub struct FSWatcher {
    watcher: RecommendedWatcher,
    pub rx: sync::mpsc::Receiver<notify::Event>,
}

pub enum Event {
    FileModified(std::path::PathBuf),
    FileCreated(std::path::PathBuf),
    FolderCreated(std::path::PathBuf),
    FolderMoved(std::path::PathBuf),
}

impl Stream for FSWatcher {
    type Item = Event;

    fn poll_next(
        mut self: std::pin::Pin<&mut Self>,
        cx: &mut Context<'_>,
    ) -> Poll<Option<Self::Item>> {
        loop {
            let event = match self.rx.poll_recv(cx) {
                Poll::Ready(Some(event)) => event,
                Poll::Ready(None) => return Poll::Ready(None),
                Poll::Pending => return Poll::Pending,
            };
            let notify::Event {
                attrs: _,
                paths,
                kind,
            } = event;

            match kind {
                EventKind::Modify(ModifyKind::Data(_)) => {
                    return Poll::Ready(Some(Event::FileModified(
                        paths.into_iter().nth(0).unwrap(),
                    )))
                }
                EventKind::Create(CreateKind::File) => {
                    return Poll::Ready(Some(Event::FileModified(
                        paths.into_iter().nth(0).unwrap(),
                    )))
                }
                EventKind::Any
                | EventKind::Modify(ModifyKind::Any)
                | EventKind::Modify(ModifyKind::Metadata(_))
                | EventKind::Modify(ModifyKind::Name(_))
                | EventKind::Modify(ModifyKind::Other)
                | EventKind::Create(CreateKind::Any)
                | EventKind::Create(CreateKind::Other)
                | EventKind::Create(CreateKind::Folder)
                | EventKind::Access(_)
                | EventKind::Remove(_)
                | EventKind::Other => continue,
            };
        }
    }
}

impl FSWatcher {
    pub fn new() -> Result<FSWatcher, Error> {
        let (tx, rx) = sync::mpsc::channel::<notify::Event>(5);
        let tx_rc = std::sync::Arc::new(sync::Mutex::new(tx));

        let watcher: RecommendedWatcher =
            notify::Watcher::new_immediate(move |res: Result<notify::Event, notify::Error>| {
                match res {
                    Ok(event) => {
                        let mut runtime = tokio::runtime::Builder::new()
                            .basic_scheduler()
                            .build()
                            .unwrap();
                        runtime.block_on(async {
                            let tx_rc = std::sync::Arc::clone(&tx_rc);
                            tx_rc.lock().await.send(event).await.unwrap();
                        });
                    }
                    Err(e) => println!("watch error: {:?}", e),
                }
            })
            .map_err(Error::CreateWatcherError)?;

        return Ok(FSWatcher { watcher, rx });
    }

    pub fn watch(&mut self, path: &std::path::Path, watch_mode: WatchMode) -> Result<(), Error> {
        self.watcher
            .watch(path, watch_mode)
            .map_err(Error::StartWatch)?;
        return Ok(());
    }
}
