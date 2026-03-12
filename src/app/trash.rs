use crate::app::project_config::{project_trash_dir, trash_meta_path};
use std::fmt;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

// phase36-delete-scope-guard-enabled: this module stays delete-flow only.
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

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TrashMetadataStatus {
    Valid,
    Missing,
    Invalid,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TrashListEntry {
    pub trash_path: PathBuf,
    pub name: String,
    pub entry_kind: TrashEntryKind,
    pub deleted_at: Option<u128>,
    pub original_relative_path: Option<PathBuf>,
    pub metadata_status: TrashMetadataStatus,
    pub metadata_error: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TrashRestoreOutcome {
    pub restored_from: PathBuf,
    pub restored_to: PathBuf,
    pub original_target: PathBuf,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RestoreConflictPolicy {
    Cancel,
    RestoreAsCopy,
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

fn validate_metadata_contract(meta: &TrashEntryMeta, entry_path: &Path) -> Result<(), TrashError> {
    if meta.original_relative_path.is_absolute() {
        return Err(TrashError::new(
            "metadata original_relative_path nesmi byt absolutni cesta",
        ));
    }
    if meta
        .original_relative_path
        .components()
        .any(|c| matches!(c, std::path::Component::ParentDir))
    {
        return Err(TrashError::new(
            "metadata original_relative_path nesmi obsahovat `..`",
        ));
    }

    let expected_name = entry_path
        .file_name()
        .and_then(|n| n.to_str())
        .ok_or_else(|| TrashError::new("trash polozka nema validni nazev"))?;
    if meta.trash_name != expected_name {
        return Err(TrashError::new(
            "metadata trash_name neodpovida nazvu trash polozky",
        ));
    }

    let actual_kind = entry_kind(entry_path)?;
    if meta.entry_kind != actual_kind {
        return Err(TrashError::new(
            "metadata entry_kind neodpovida realnemu typu trash polozky",
        ));
    }

    Ok(())
}

fn write_metadata_sidecar(entry_path: &Path, meta: &TrashEntryMeta) -> Result<(), TrashError> {
    let meta_path = trash_meta_path(entry_path);
    let raw = serde_json::to_string_pretty(meta)
        .map_err(|e| TrashError::new(format!("nelze serializovat trash metadata: {e}")))?;
    fs::write(meta_path, raw)
        .map_err(|e| TrashError::new(format!("nelze zapsat trash metadata sidecar: {e}")))
}

fn read_metadata_sidecar(entry_path: &Path) -> Result<TrashEntryMeta, TrashError> {
    let meta_path = trash_meta_path(entry_path);
    let raw = fs::read_to_string(&meta_path)
        .map_err(|e| TrashError::new(format!("nelze nacist trash metadata sidecar: {e}")))?;
    let meta: TrashEntryMeta = serde_json::from_str(&raw)
        .map_err(|e| TrashError::new(format!("metadata nejsou validni JSON: {e}")))?;
    validate_metadata_contract(&meta, entry_path)?;
    Ok(meta)
}

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
    let trash_dir = project_trash_dir(project_root);
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

fn resolve_trash_destination(
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

fn is_inside_trash_dir(project_root: &Path, candidate_abs: &Path) -> bool {
    let trash_abs = project_trash_dir(project_root);
    candidate_abs == trash_abs || candidate_abs.starts_with(&trash_abs)
}

fn format_fail_closed_move_error(reason: &str, err: &std::io::Error) -> TrashError {
    TrashError::new(format!(
        "{reason}: {err}; puvodni polozka zustava beze zmeny, zkontrolujte prava a zkuste akci znovu"
    ))
}

pub fn list_trash_entries(project_root: &Path) -> Result<Vec<TrashListEntry>, TrashError> {
    let project_abs = project_root
        .canonicalize()
        .map_err(|e| TrashError::new(format!("nelze kanonizovat root projektu: {e}")))?;
    let trash_root = project_trash_dir(&project_abs);
    if !trash_root.exists() {
        return Ok(Vec::new());
    }
    if !trash_root.is_dir() {
        return Err(TrashError::new(
            "konflikt cesty: `.polycredo/trash` existuje jako soubor",
        ));
    }

    let mut entries = Vec::new();
    for walk_entry in walkdir::WalkDir::new(&trash_root).min_depth(1) {
        let walk_entry = walk_entry
            .map_err(|e| TrashError::new(format!("nelze cist obsah trash adresare: {e}")))?;
        let path = walk_entry.path();
        if path.is_file() && path.extension().and_then(|x| x.to_str()) == Some("json") {
            let file_name = path
                .file_name()
                .and_then(|x| x.to_str())
                .unwrap_or_default();
            if file_name.ends_with(".meta.json") {
                continue;
            }
        }
        if !path.is_file() && !path.is_dir() {
            continue;
        }

        let name = path
            .file_name()
            .and_then(|n| n.to_str())
            .ok_or_else(|| TrashError::new("trash polozka nema validni nazev"))?
            .to_string();
        let kind = entry_kind(path)?;

        let (deleted_at, original_relative_path, metadata_status, metadata_error) =
            match read_metadata_sidecar(path) {
                Ok(meta) => (
                    Some(meta.deleted_at),
                    Some(meta.original_relative_path),
                    TrashMetadataStatus::Valid,
                    None,
                ),
                Err(err) => {
                    let meta_path = trash_meta_path(path);
                    let status = if meta_path.exists() {
                        TrashMetadataStatus::Invalid
                    } else {
                        TrashMetadataStatus::Missing
                    };
                    (None, None, status, Some(err.to_string()))
                }
            };

        entries.push(TrashListEntry {
            trash_path: path.to_path_buf(),
            name,
            entry_kind: kind,
            deleted_at,
            original_relative_path,
            metadata_status,
            metadata_error,
        });
    }

    entries.sort_by(|a, b| {
        b.deleted_at
            .cmp(&a.deleted_at)
            .then_with(|| a.name.cmp(&b.name))
    });
    Ok(entries)
}

pub fn restore_from_trash(
    project_root: &Path,
    trash_entry_path: &Path,
) -> Result<TrashRestoreOutcome, TrashError> {
    restore_from_trash_with_policy(
        project_root,
        trash_entry_path,
        RestoreConflictPolicy::Cancel,
    )
}

fn resolve_restore_copy_destination(original_target: &Path) -> Result<PathBuf, TrashError> {
    if !original_target.exists() {
        return Ok(original_target.to_path_buf());
    }
    let parent = original_target
        .parent()
        .ok_or_else(|| TrashError::new("cilova cesta nema parent adresar"))?;
    let stem = original_target
        .file_stem()
        .and_then(|s| s.to_str())
        .ok_or_else(|| TrashError::new("cilovy nazev nema validni stem"))?;
    let ext = original_target.extension().and_then(|s| s.to_str());
    for attempt in 1..=1000_u32 {
        let candidate_name = if let Some(ext) = ext {
            format!("{stem}-restored-copy-{attempt}.{ext}")
        } else {
            format!("{stem}-restored-copy-{attempt}")
        };
        let candidate = parent.join(candidate_name);
        if !candidate.exists() {
            return Ok(candidate);
        }
    }
    Err(TrashError::new(
        "nelze najit volny nazev pro obnovu jako kopii po 1000 pokusech",
    ))
}

pub fn restore_from_trash_with_policy(
    project_root: &Path,
    trash_entry_path: &Path,
    conflict_policy: RestoreConflictPolicy,
) -> Result<TrashRestoreOutcome, TrashError> {
    let restore_error = |detail: String| TrashError::new(format!("restore selhal: {detail}"));

    let project_abs = project_root
        .canonicalize()
        .map_err(|e| restore_error(format!("nelze kanonizovat root projektu: {e}")))?;
    let trash_root = ensure_trash_dir(&project_abs)?;

    let source_candidate = if trash_entry_path.is_absolute() {
        trash_entry_path.to_path_buf()
    } else {
        trash_root.join(trash_entry_path)
    };
    if !source_candidate.exists() {
        return Err(restore_error(
            "trash source neexistuje; polozka mohla byt uz obnovena nebo odstranena".to_string(),
        ));
    }
    let source_abs = source_candidate
        .canonicalize()
        .map_err(|e| restore_error(format!("nelze kanonizovat trash source: {e}")))?;
    if !source_abs.starts_with(&trash_root) {
        return Err(restore_error(
            "source neni uvnitr `.polycredo/trash`; operace byla zastavena".to_string(),
        ));
    }
    if source_abs.is_dir() {
        return Err(restore_error(
            "obnova adresare zatim neni v MVP podporena".to_string(),
        ));
    }
    let meta_path = trash_meta_path(&source_abs);
    if !meta_path.exists() {
        return Err(restore_error(
            "chybi metadata sidecar, polozku nelze bezpecne mapovat na puvodni cestu".to_string(),
        ));
    }

    let meta = read_metadata_sidecar(&source_abs).map_err(|e| restore_error(e.to_string()))?;
    let original_target = project_abs.join(&meta.original_relative_path);
    if !original_target.starts_with(&project_abs) {
        return Err(restore_error(
            "metadata ukazuji mimo root projektu; operace byla zastavena".to_string(),
        ));
    }
    let restore_target = if original_target.exists() {
        match conflict_policy {
            RestoreConflictPolicy::Cancel => {
                return Err(restore_error(
                    "konflikt: cilova cesta uz existuje; pouzijte restore jako kopii".to_string(),
                ));
            }
            RestoreConflictPolicy::RestoreAsCopy => {
                resolve_restore_copy_destination(&original_target)
                    .map_err(|e| restore_error(e.to_string()))?
            }
        }
    } else {
        original_target.clone()
    };

    if let Some(parent) = restore_target.parent() {
        fs::create_dir_all(parent)
            .map_err(|e| restore_error(format!("nelze vytvorit parent adresare: {e}")))?;
    }

    fs::rename(&source_abs, &restore_target).map_err(|e| {
        restore_error(format!(
            "{e}; trash polozka zustava beze zmeny, zkontrolujte prava a zkuste znovu"
        ))
    })?;

    if let Err(cleanup_err) = fs::remove_file(&meta_path) {
        return match fs::rename(&restore_target, &source_abs) {
            Ok(_) => Err(restore_error(format!(
                "operace byla vracena zpet: metadata cleanup selhal ({cleanup_err}); trash polozka zustava beze zmeny"
            ))),
            Err(rollback_err) => Err(restore_error(format!(
                "presunul data, ale cleanup metadata selhal ({cleanup_err}); rollback selhal ({rollback_err})"
            ))),
        };
    }

    Ok(TrashRestoreOutcome {
        restored_from: source_abs,
        restored_to: restore_target,
        original_target,
    })
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
    if is_inside_trash_dir(&project_abs, &source_abs) {
        return Err(TrashError::new(
            "nelze smazat interni `.polycredo/trash`; zkuste smazat polozku mimo trash",
        ));
    }
    let kind = entry_kind(&source_abs)?;
    let deleted_at = now_unix_millis()?;

    let trash_root = ensure_trash_dir(&project_abs)?;
    let (trash_name, destination_abs) =
        resolve_trash_destination(&trash_root, &relative_path, deleted_at)?;
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
        format_fail_closed_move_error(reason, &e)
    })?;

    let meta = TrashEntryMeta {
        trash_name,
        original_relative_path: relative_path,
        deleted_at,
        entry_kind: kind,
    };

    if let Err(meta_err) = write_metadata_sidecar(&destination_abs, &meta) {
        return match std::fs::rename(&destination_abs, &source_abs) {
            Ok(_) => Err(TrashError::new(format!(
                "presun do trash byl vracen zpet: {meta_err}; puvodni polozka zustava beze zmeny"
            ))),
            Err(rollback_err) => Err(TrashError::new(format!(
                "presun do trash dokoncen, ale zapis metadata selhal ({meta_err}); navrat selhal ({rollback_err}), polozka zustala v trash bez metadata"
            ))),
        };
    }

    Ok(TrashMoveOutcome {
        moved_from: source_abs,
        moved_to: destination_abs,
        meta,
    })
}
