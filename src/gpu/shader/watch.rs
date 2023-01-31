use core::time;
use std::{collections::HashSet, path, sync::mpsc, thread};

use notify::Watcher;

pub fn watch<F>(paths: &HashSet<path::PathBuf>, mut on_update: F) -> notify::RecommendedWatcher
where
    F: 'static + FnMut() + Send,
{
    let (tx, rx) = mpsc::channel();
    let mut watcher = notify::watcher(tx, time::Duration::from_millis(100))
        .expect("Failed to initialize watcher");
    for p in paths.into_iter() {
        watcher
            .watch(p, notify::RecursiveMode::NonRecursive)
            .expect("Failed to watch");
    }

    let ps = paths.clone();
    thread::spawn(move || loop {
        tracing::info!(paths = ?ps, "waiting for change");
        match rx.recv() {
            Err(e) => {
                tracing::error!(paths = ?ps, error = ?e, "recv");
            }
            Ok(notify::DebouncedEvent::Write(path)) => {
                tracing::info!(path = ?path, "updated");
                on_update();
            }
            _ => {}
        }
    });

    watcher
}
