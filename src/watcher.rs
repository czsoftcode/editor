use notify::{Event, EventKind, RecommendedWatcher, RecursiveMode, Watcher};
use std::path::{Path, PathBuf};
use std::sync::mpsc;

pub struct FileWatcher {
    _watcher: Option<RecommendedWatcher>,
    receiver: mpsc::Receiver<PathBuf>,
    sender: mpsc::Sender<PathBuf>,
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
                if event.kind.is_modify() || event.kind.is_create() {
                    for p in event.paths {
                        let _ = tx.send(p);
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

    pub fn try_recv(&self) -> Option<PathBuf> {
        let mut last = None;
        while let Ok(path) = self.receiver.try_recv() {
            last = Some(path);
        }
        last
    }
}

pub enum FsChange {
    Created(PathBuf),
    Removed(PathBuf),
    Modified,
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
                    // Ignorovat změny v .git, target atd.
                    let skip = p.components().any(|c| {
                        let s = c.as_os_str().to_string_lossy();
                        matches!(s.as_ref(), ".git" | "target" | "node_modules")
                    });
                    if skip {
                        continue;
                    }

                    let change = match &event.kind {
                        EventKind::Create(_) => Some(FsChange::Created(p.clone())),
                        EventKind::Remove(_) => Some(FsChange::Removed(p.clone())),
                        EventKind::Modify(_) => Some(FsChange::Modified),
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
}
