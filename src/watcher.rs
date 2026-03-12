use notify::{Event, EventKind, RecommendedWatcher, RecursiveMode, Watcher};
use std::collections::{HashMap, HashSet};
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
        let mut count = 0;
        while let Ok(ev) = self.receiver.try_recv() {
            events.push(ev);
            count += 1;
            if count >= 500 {
                break;
            }
        }
        events
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum FsChange {
    Created(PathBuf),
    Removed(PathBuf),
    Modified(PathBuf),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
enum FsChangeKind {
    Created,
    Removed,
    Modified,
}

impl FsChange {
    #[allow(dead_code)]
    pub fn path(&self) -> &PathBuf {
        match self {
            FsChange::Created(p) => p,
            FsChange::Removed(p) => p,
            FsChange::Modified(p) => p,
        }
    }

    fn kind(&self) -> FsChangeKind {
        match self {
            FsChange::Created(_) => FsChangeKind::Created,
            FsChange::Removed(_) => FsChangeKind::Removed,
            FsChange::Modified(_) => FsChangeKind::Modified,
        }
    }
}

fn merge_event(current: FsChangeKind, incoming: FsChangeKind) -> FsChangeKind {
    use FsChangeKind::{Created, Modified, Removed};
    match (current, incoming) {
        (Removed, _) | (_, Removed) => Removed,
        (Created, _) | (_, Created) => Created,
        _ => Modified,
    }
}

fn change_from_kind(path: PathBuf, kind: FsChangeKind) -> FsChange {
    match kind {
        FsChangeKind::Created => FsChange::Created(path),
        FsChangeKind::Removed => FsChange::Removed(path),
        FsChangeKind::Modified => FsChange::Modified(path),
    }
}

pub const PROJECT_WATCHER_BATCH_WINDOW_MS: u64 = 120;
const PROJECT_WATCHER_MAX_EVENTS: usize = 500;

#[derive(Debug, Default)]
pub struct ProjectWatcherBatch {
    pub changes: Vec<FsChange>,
    pub overflowed: bool,
}

fn build_project_watcher_batch(
    changes: Vec<FsChange>,
    max_events: usize,
) -> ProjectWatcherBatch {
    if changes.len() > max_events {
        return ProjectWatcherBatch {
            changes: Vec::new(),
            overflowed: true,
        };
    }

    let mut seen: HashSet<(PathBuf, FsChangeKind)> = HashSet::new();
    let mut merged_by_path: HashMap<PathBuf, FsChangeKind> = HashMap::new();

    for change in changes {
        let path = change.path().clone();
        let kind = change.kind();

        if !seen.insert((path.clone(), kind)) {
            continue;
        }

        merged_by_path
            .entry(path)
            .and_modify(|existing| *existing = merge_event(*existing, kind))
            .or_insert(kind);
    }

    let mut ordered: Vec<_> = merged_by_path.into_iter().collect();
    ordered.sort_by(|(a, _), (b, _)| a.cmp(b));

    let changes = ordered
        .into_iter()
        .map(|(path, kind)| change_from_kind(path, kind))
        .collect();

    ProjectWatcherBatch {
        changes,
        overflowed: false,
    }
}

pub(crate) fn build_project_watcher_batch_for_tests(
    changes: Vec<FsChange>,
    max_events: usize,
) -> ProjectWatcherBatch {
    build_project_watcher_batch(changes, max_events)
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
                    // Skip the entire .polycredo directory and common high-frequency ignore directories.
                    let is_in_polycredo = p.as_path().to_string_lossy().contains(".polycredo");

                    let skip = if is_in_polycredo {
                        true
                    } else {
                        p.components().any(|c| {
                            let s = c.as_os_str().to_string_lossy();
                            matches!(
                                s.as_ref(),
                                ".git" | "target" | "node_modules" | "history" | ".flatpak-builder"
                            )
                        })
                    };

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

    pub fn poll(&self) -> ProjectWatcherBatch {
        let mut raw_changes = Vec::new();
        while let Ok(change) = self.receiver.try_recv() {
            raw_changes.push(change);
            if raw_changes.len() > PROJECT_WATCHER_MAX_EVENTS {
                while self.receiver.try_recv().is_ok() {}
                return ProjectWatcherBatch {
                    changes: Vec::new(),
                    overflowed: true,
                };
            }
        }

        build_project_watcher_batch(raw_changes, PROJECT_WATCHER_MAX_EVENTS)
    }

    pub fn add_path(&mut self, path: &Path) {
        if let Some(ref mut w) = self._watcher {
            let _ = w.watch(path, RecursiveMode::Recursive);
        }
    }
}
