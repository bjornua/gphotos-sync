use notify::{RecommendedWatcher, Watcher};
use tokio::sync;

#[derive(Debug)]
pub enum Error {
    CreateWatcherError(notify::Error),
    StartWatch(notify::Error),
}

type Event = notify::Event;

pub type WatchMode = notify::RecursiveMode;

pub struct FSWatcher {
    watcher: RecommendedWatcher,
    pub rx: sync::mpsc::Receiver<Event>,
}

impl FSWatcher {
    pub fn new() -> Result<FSWatcher, Error> {
        let (tx, rx) = sync::mpsc::channel::<Event>(5);
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

        return Ok(FSWatcher {
            watcher: watcher,
            rx: rx,
        });
    }

    pub fn watch(&mut self, path: &std::path::Path, watch_mode: WatchMode) -> Result<(), Error> {
        self.watcher
            .watch(path, watch_mode)
            .map_err(Error::StartWatch)?;
        return Ok(());
    }
}
