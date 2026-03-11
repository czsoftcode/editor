use crate::app::project_config::trash_dir_path;
use std::fmt;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Clone, Copy, Debug, serde::Serialize, serde::Deserialize, PartialEq, Eq)]
pub enum TrashEntryKind {
    File,
    Directory,
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, PartialEq, Eq)]
pub struct TrashEntryMeta {
    pub trash_name: String,
    pub original_relative_path: PathBuf,
    pub deleted_at: u128,
    pub entry_kind: TrashEntryKind,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TrashMoveOutcome {
    pub moved_from: PathBuf,
    pub moved_to: PathBuf,
    pub meta: TrashEntryMeta,
}

#[derive(Debug)]
pub struct TrashError {
    message: String,
}

impl TrashError {
    fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }
}

impl fmt::Display for TrashError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl std::error::Error for TrashError {}

fn now_unix_millis() -> Result<u128, TrashError> {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis())
        .map_err(|_| TrashError::new("nelze urcit aktualni cas pro trash metadata"))
}

fn entry_kind(path: &Path) -> Result<TrashEntryKind, TrashError> {
    let meta = std::fs::symlink_metadata(path)
        .map_err(|e| TrashError::new(format!("nelze nacist metadata mazane polozky: {e}")))?;
    if meta.is_dir() {
        Ok(TrashEntryKind::Directory)
    } else {
        Ok(TrashEntryKind::File)
    }
}

pub fn ensure_trash_dir(project_root: &Path) -> Result<PathBuf, TrashError> {
    let trash_dir = trash_dir_path(project_root);
    if trash_dir.exists() && !trash_dir.is_dir() {
        return Err(TrashError::new(
            "konflikt cesty: `.polycredo/trash` existuje jako soubor; prejmenujte nebo odstranite tento soubor a zkuste akci znovu",
        ));
    }
    std::fs::create_dir_all(&trash_dir).map_err(|e| {
        TrashError::new(format!(
            "nelze vytvorit adresar `.polycredo/trash` ({e}); zkontrolujte prava zapisu k projektu"
        ))
    })?;
    Ok(trash_dir)
}

fn build_unique_destination(
    trash_root: &Path,
    original_relative_path: &Path,
    deleted_at: u128,
) -> Result<(String, PathBuf), TrashError> {
    let file_name = original_relative_path
        .file_name()
        .and_then(|n| n.to_str())
        .ok_or_else(|| TrashError::new("nelze odvodit nazev mazane polozky"))?;
    let rel_parent = original_relative_path
        .parent()
        .map(Path::to_path_buf)
        .unwrap_or_default();

    for attempt in 0..1000_u32 {
        let trash_name = if attempt == 0 {
            format!("{file_name}.trash-{deleted_at}")
        } else {
            format!("{file_name}.trash-{deleted_at}-{attempt}")
        };
        let rel_target = rel_parent.join(&trash_name);
        let abs_target = trash_root.join(rel_target);
        if !abs_target.exists() {
            return Ok((trash_name, abs_target));
        }
    }
    Err(TrashError::new(
        "nelze vytvorit unikatni trash nazev po 1000 pokusech",
    ))
}

pub fn move_path_to_trash(
    project_root: &Path,
    source_path: &Path,
) -> Result<TrashMoveOutcome, TrashError> {
    let source_abs = source_path
        .canonicalize()
        .map_err(|e| TrashError::new(format!("nelze kanonizovat mazanou cestu: {e}")))?;
    let project_abs = project_root
        .canonicalize()
        .map_err(|e| TrashError::new(format!("nelze kanonizovat root projektu: {e}")))?;
    if !source_abs.starts_with(&project_abs) {
        return Err(TrashError::new(
            "mazana cesta je mimo root projektu; operace byla zastavena",
        ));
    }
    let relative_path = source_abs
        .strip_prefix(&project_abs)
        .map_err(|_| TrashError::new("nelze odvodit relativni cestu mazane polozky"))?
        .to_path_buf();
    let kind = entry_kind(&source_abs)?;
    let deleted_at = now_unix_millis()?;

    let trash_root = ensure_trash_dir(&project_abs)?;
    let (trash_name, destination_abs) =
        build_unique_destination(&trash_root, &relative_path, deleted_at)?;
    if let Some(parent) = destination_abs.parent() {
        std::fs::create_dir_all(parent)
            .map_err(|e| TrashError::new(format!("nelze pripravit cilovy trash adresar: {e}")))?;
    }

    std::fs::rename(&source_abs, &destination_abs).map_err(|e| {
        let reason = if e.raw_os_error() == Some(18) {
            "presun mezi filesystémy (EXDEV) neni v phase 35 podporen bezpecnym fallbackem"
        } else {
            "presun do trash selhal"
        };
        TrashError::new(format!("{reason}: {e}; puvodni polozka zustava beze zmeny"))
    })?;

    let meta = TrashEntryMeta {
        trash_name,
        original_relative_path: relative_path,
        deleted_at,
        entry_kind: kind,
    };

    Ok(TrashMoveOutcome {
        moved_from: source_abs,
        moved_to: destination_abs,
        meta,
    })
}
