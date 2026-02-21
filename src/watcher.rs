use notify::{Event, EventKind, RecommendedWatcher, RecursiveMode, Watcher};
use std::path::{Path, PathBuf};
use std::sync::mpsc;

pub enum FileEvent {
    Changed(PathBuf),
    Removed(PathBuf),
}

pub struct FileWatcher {
    _watcher: Option<RecommendedWatcher>,
    receiver: mpsc::Receiver<FileEvent>,
    sender: mpsc::Sender<FileEvent>,
    watched_path: Option<PathBuf>,
}

impl FileWatcher {
    pub fn new() -> Self {
        let (sender, receiver) = mpsc::channel();
        Self {
            _watcher: None,
            receiver,
            sender,
            watched_path: None,
        }
    }

    pub fn watch(&mut self, path: &Path) {
        let canonical = match path.canonicalize() {
            Ok(p) => p,
            Err(_) => return,
        };

        if self.watched_path.as_ref() == Some(&canonical) {
            return;
        }

        let tx = self.sender.clone();
        let watcher = notify::recommended_watcher(move |res: Result<Event, notify::Error>| {
            if let Ok(event) = res {
                for p in event.paths {
                    let ev = if event.kind.is_remove() {
                        Some(FileEvent::Removed(p))
                    } else if event.kind.is_modify() || event.kind.is_create() {
                        Some(FileEvent::Changed(p))
                    } else {
                        None
                    };
                    if let Some(ev) = ev {
                        let _ = tx.send(ev);
                    }
                }
            }
        });

        match watcher {
            Ok(mut w) => {
                let _ = w.watch(&canonical, RecursiveMode::NonRecursive);
                self._watcher = Some(w);
                self.watched_path = Some(canonical);
            }
            Err(_) => {
                self._watcher = None;
                self.watched_path = None;
            }
        }
    }

    pub fn try_recv(&self) -> Vec<FileEvent> {
        let mut events = Vec::new();
        while let Ok(ev) = self.receiver.try_recv() {
            events.push(ev);
        }
        events
    }
}

#[derive(Clone)]
pub enum FsChange {
    Created(PathBuf),
    Removed(PathBuf),
    Modified(PathBuf),
}

pub struct ProjectWatcher {
    _watcher: Option<RecommendedWatcher>,
    receiver: mpsc::Receiver<FsChange>,
}

impl ProjectWatcher {
    pub fn new(root: &Path) -> Self {
        let (tx, receiver) = mpsc::channel();

        let watcher = notify::recommended_watcher(move |res: Result<Event, notify::Error>| {
            if let Ok(event) = res {
                for p in &event.paths {
                    // Ignore changes in .git, target, history, etc.
                    let skip = p.components().any(|c| {
                        let s = c.as_os_str().to_string_lossy();
                        matches!(s.as_ref(), ".git" | "target" | "node_modules")
                    }) || p.to_string_lossy().contains(".polycredo/history");

                    if skip {
                        continue;
                    }

                    let change = match &event.kind {
                        EventKind::Create(_) => Some(FsChange::Created(p.clone())),
                        EventKind::Remove(_) => Some(FsChange::Removed(p.clone())),
                        EventKind::Modify(_) => Some(FsChange::Modified(p.clone())),
                        _ => None,
                    };
                    if let Some(c) = change {
                        let _ = tx.send(c);
                    }
                }
            }
        });

        let watcher = match watcher {
            Ok(mut w) => {
                let _ = w.watch(root, RecursiveMode::Recursive);
                Some(w)
            }
            Err(_) => None,
        };

        Self {
            _watcher: watcher,
            receiver,
        }
    }

    pub fn poll(&self) -> Vec<FsChange> {
        let mut changes = Vec::new();
        while let Ok(change) = self.receiver.try_recv() {
            changes.push(change);
        }
        changes
    }

    pub fn add_path(&mut self, path: &Path) {
        if let Some(ref mut w) = self._watcher {
            let _ = w.watch(path, RecursiveMode::Recursive);
        }
    }
}
