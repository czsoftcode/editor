//! IPC (Inter-Process Communication) for managing multiple editor instances.
//!
//! The first launched instance becomes the primary server — it listens on a Unix socket,
//! maintains a list of open projects and recent projects.
//! Every subsequent instance is a client and communicates with the server.
//!
//! Protocol (text commands, one per line):
//!   PING                    → PONG
//!   QUERY /abs/path         → OPEN | FOCUSED
//!   REGISTER pid /abs/path  → OK
//!   UNREGISTER pid          → OK
//!   ADD_RECENT /abs/path    → OK
//!   RECENT                  → /path1 \n /path2 \n ... \n END

use std::collections::HashMap;
use std::io::{BufRead, BufReader, Write};
use std::os::unix::net::{UnixListener, UnixStream};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};

use serde_json;

use crate::config;

const IPC_MAX_WORKERS: usize = 16;
const CONFIG_DIR_NAME: &str = "polycredo-editor";

// ---------------------------------------------------------------------------
// File paths
// ---------------------------------------------------------------------------

fn config_root_dir() -> PathBuf {
    dirs::config_dir().unwrap_or_else(std::env::temp_dir)
}

fn config_dir() -> PathBuf {
    config_root_dir().join(CONFIG_DIR_NAME)
}

fn socket_path() -> PathBuf {
    config_dir().join("polycredo-editor.sock")
}

fn recent_path() -> PathBuf {
    config_dir().join("recent.json")
}

fn session_path() -> PathBuf {
    config_dir().join("session.json")
}

fn normalize_project_path(path: &Path) -> Option<PathBuf> {
    if !path.is_absolute() {
        return None;
    }
    let canonical = path.canonicalize().ok()?;
    if canonical.is_dir() {
        Some(canonical)
    } else {
        None
    }
}

fn normalize_project_path_str(path: &str) -> Option<PathBuf> {
    normalize_project_path(Path::new(path))
}

// ---------------------------------------------------------------------------
// Recent projects storage
// ---------------------------------------------------------------------------

fn save_paths(file_path: &std::path::Path, paths: &[PathBuf]) {
    if let Some(parent) = file_path.parent() {
        if let Err(e) = std::fs::create_dir_all(parent) {
            eprintln!(
                "ipc: cannot create config directory {}: {e}",
                parent.display()
            );
            return;
        }
    }
    let strings: Vec<&str> = paths.iter().filter_map(|p| p.to_str()).collect();
    let Ok(content) = serde_json::to_string(&strings) else {
        eprintln!("ipc: path serialization failed");
        return;
    };
    let tmp = file_path.with_extension("tmp");
    if let Err(e) = std::fs::write(&tmp, content) {
        eprintln!("ipc: cannot write tmp file {}: {e}", tmp.display());
        return;
    }
    if let Err(e) = std::fs::rename(&tmp, file_path) {
        eprintln!(
            "ipc: cannot atomically rename {} -> {}: {e}",
            tmp.display(),
            file_path.display()
        );
        let _ = std::fs::remove_file(&tmp);
    }
}

fn parse_paths(content: &str) -> Vec<PathBuf> {
    let Ok(strings): Result<Vec<String>, _> = serde_json::from_str(content) else {
        return vec![];
    };
    strings
        .into_iter()
        .filter_map(|s| normalize_project_path_str(&s))
        .collect()
}

fn load_paths(file_path: &std::path::Path) -> Vec<PathBuf> {
    let Ok(content) = std::fs::read_to_string(file_path) else {
        return vec![];
    };
    parse_paths(&content)
}

fn save_recent(recent: &[PathBuf]) {
    save_paths(&recent_path(), recent);
}

fn load_recent() -> Vec<PathBuf> {
    load_paths(&recent_path())
}

// ---------------------------------------------------------------------------
// Per-process socket — window focus without external tools
// ---------------------------------------------------------------------------

/// Path to the socket of a specific process (for the FOCUS command).
pub fn process_socket_path(pid: u32) -> PathBuf {
    std::env::temp_dir().join(format!("polycredo-editor_{}.sock", pid))
}

/// Starts a listener on the per-process socket. Returns a channel on which the main thread
/// receives a signal to try `ViewportCommand::Focus`.
pub fn start_process_listener() -> std::sync::mpsc::Receiver<()> {
    let (tx, rx) = std::sync::mpsc::channel();
    let sock = process_socket_path(std::process::id());
    let _ = std::fs::remove_file(&sock);
    if let Ok(listener) = UnixListener::bind(&sock) {
        std::thread::spawn(move || {
            for stream in listener.incoming().flatten() {
                let reader = BufReader::new(&stream);
                if let Some(Ok(line)) = reader.lines().next() {
                    if line.trim() == "FOCUS" {
                        let _ = tx.send(());
                    }
                }
            }
        });
    }
    rx
}

/// Sends the FOCUS command to the target process via its per-process socket.
/// Returns true if the connection was successful (process is alive).
fn focus_process_window(pid: u32) -> bool {
    let sock = process_socket_path(pid);
    let Ok(stream) = UnixStream::connect(&sock) else {
        return false;
    };
    stream
        .set_write_timeout(Some(std::time::Duration::from_secs(1)))
        .ok();
    let mut writer = &stream;
    writeln!(writer, "FOCUS").is_ok()
}

// ---------------------------------------------------------------------------
// Server
// ---------------------------------------------------------------------------

struct ServerState {
    /// Mapping path → PID of the open project
    registered: HashMap<PathBuf, u32>,
    /// Recent projects (max 10)
    recent: Vec<PathBuf>,
    /// Channel for passing project opening requests to the primary instance.
    open_tx: std::sync::mpsc::Sender<PathBuf>,
}

fn process_command(line: &str, state: &Arc<Mutex<ServerState>>) -> Vec<String> {
    if line == "PING" {
        return vec!["PONG".into()];
    }

    if line == "RECENT" {
        let st = state.lock().unwrap();
        let mut lines: Vec<String> = st
            .recent
            .iter()
            .map(|p| p.to_string_lossy().into_owned())
            .collect();
        lines.push("END".into());
        return lines;
    }

    if let Some(path_str) = line.strip_prefix("QUERY ") {
        let Some(path) = normalize_project_path_str(path_str) else {
            return vec!["OPEN".into()];
        };
        // Read PID without holding the lock during network call
        let pid_opt = state.lock().unwrap().registered.get(&path).copied();
        if let Some(pid) = pid_opt {
            if focus_process_window(pid) {
                // Process responded — requested focus for its window
                return vec!["FOCUSED".into()];
            } else {
                // Process is dead — clear registration
                state.lock().unwrap().registered.remove(&path);
            }
        }
        return vec!["OPEN".into()];
    }

    if let Some(rest) = line.strip_prefix("REGISTER ") {
        if let Some((pid_str, path_str)) = rest.split_once(' ') {
            if let Ok(pid) = pid_str.parse::<u32>() {
                let Some(path) = normalize_project_path_str(path_str) else {
                    return vec!["ERR bad REGISTER".into()];
                };
                let mut st = state.lock().unwrap();
                // Add registration (do not clear other entries of this PID — one PID can
                // own multiple projects in a multi-viewport architecture)
                st.registered.insert(path, pid);
                return vec!["OK".into()];
            }
        }
        return vec!["ERR bad REGISTER".into()];
    }

    if let Some(pid_str) = line.strip_prefix("UNREGISTER ") {
        if let Ok(pid) = pid_str.parse::<u32>() {
            state.lock().unwrap().registered.retain(|_, v| *v != pid);
        }
        return vec!["OK".into()];
    }

    if let Some(path_str) = line.strip_prefix("ADD_RECENT ") {
        if let Some(path) = normalize_project_path_str(path_str) {
            let mut st = state.lock().unwrap();
            st.recent.retain(|p| p != &path);
            st.recent.insert(0, path);
            st.recent.truncate(config::MAX_RECENT_PROJECTS);
            save_recent(&st.recent);
        }
        return vec!["OK".into()];
    }

    // SAVE_SESSION <json-array> — serialized writing of session.json via the server.
    // The server holds the state mutex during writing, so concurrent calls from multiple processes
    // cannot write to session.tmp at once.
    if let Some(json) = line.strip_prefix("SAVE_SESSION ") {
        let paths: Vec<PathBuf> = serde_json::from_str::<Vec<String>>(json)
            .unwrap_or_default()
            .into_iter()
            .map(PathBuf::from)
            .collect();
        let _guard = state.lock().unwrap();
        save_paths(&session_path(), &paths);
        return vec!["OK".into()];
    }

    // OPEN_IN_NEW_WINDOW /abs/path — secondary instance requests the primary to open a project.
    if let Some(path_str) = line.strip_prefix("OPEN_IN_NEW_WINDOW ") {
        if let Some(path) = normalize_project_path_str(path_str) {
            let st = state.lock().unwrap();
            let _ = st.open_tx.send(path);
        }
        return vec!["OK".into()];
    }

    // FOCUS_MAIN — secondary instance without arguments requests the primary to be brought to the foreground.
    if line == "FOCUS_MAIN" {
        let pid_opt = state.lock().unwrap().registered.values().next().copied();
        if let Some(pid) = pid_opt {
            if focus_process_window(pid) {
                return vec!["OK".into()];
            }
        }
        return vec!["ERR no registered window".into()];
    }

    vec!["ERR unknown command".into()]
}

fn handle_connection(stream: UnixStream, state: Arc<Mutex<ServerState>>) {
    stream
        .set_read_timeout(Some(std::time::Duration::from_secs(5)))
        .ok();
    let reader = BufReader::new(&stream);
    if let Some(Ok(line)) = reader.lines().next() {
        let responses = process_command(&line, &state);
        let mut writer = &stream;
        for resp in responses {
            let _ = writeln!(writer, "{}", resp);
        }
    }
}

fn server_loop(listener: UnixListener, state: Arc<Mutex<ServerState>>) {
    let active_workers = Arc::new(AtomicUsize::new(0));
    for stream in listener.incoming().flatten() {
        if active_workers.load(Ordering::Relaxed) >= IPC_MAX_WORKERS {
            continue;
        }
        let state = state.clone();
        let active_workers = active_workers.clone();
        active_workers.fetch_add(1, Ordering::SeqCst);
        std::thread::spawn(move || {
            handle_connection(stream, state);
            active_workers.fetch_sub(1, Ordering::SeqCst);
        });
    }
}

/// Handle for managing the primary instance (server). Removes the socket on drop.
pub struct IpcServer {
    sock_path: PathBuf,
}

impl IpcServer {
    /// Attempts to become the primary instance.
    /// Returns `Some((IpcServer, Receiver<PathBuf>))` if successful —
    /// the Receiver brings paths to open in a new window from secondary instances.
    /// Returns `None` if a primary already exists.
    pub fn start() -> Option<(Self, std::sync::mpsc::Receiver<PathBuf>)> {
        let sock = socket_path();
        if let Some(parent) = sock.parent() {
            if let Err(e) = std::fs::create_dir_all(parent) {
                eprintln!(
                    "ipc: cannot create socket directory {}: {e}",
                    parent.display()
                );
                return None;
            }
        }

        // Verify if primary is already running
        if Ipc::ping() {
            return None; // Another primary instance exists
        }

        // Remove potential old (non-functional) socket
        if let Err(e) = std::fs::remove_file(&sock) {
            if e.kind() != std::io::ErrorKind::NotFound {
                eprintln!("ipc: cannot remove old socket {}: {e}", sock.display());
            }
        }

        let listener = UnixListener::bind(&sock).ok()?;

        let (open_tx, open_rx) = std::sync::mpsc::channel();
        let state = Arc::new(Mutex::new(ServerState {
            registered: HashMap::new(),
            recent: load_recent(),
            open_tx,
        }));

        std::thread::spawn(move || server_loop(listener, state));

        Some((IpcServer { sock_path: sock }, open_rx))
    }
}

impl Drop for IpcServer {
    fn drop(&mut self) {
        if let Err(e) = std::fs::remove_file(&self.sock_path) {
            if e.kind() != std::io::ErrorKind::NotFound {
                eprintln!(
                    "ipc: cannot remove socket {}: {e}",
                    self.sock_path.display()
                );
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Client
// ---------------------------------------------------------------------------

/// Stateless IPC client — each call opens a new connection.
pub struct Ipc;

impl Ipc {
    fn connect() -> Option<UnixStream> {
        let stream = UnixStream::connect(socket_path()).ok()?;
        stream
            .set_read_timeout(Some(std::time::Duration::from_secs(2)))
            .ok();
        Some(stream)
    }

    fn send_one(command: &str) -> Option<String> {
        let stream = Self::connect()?;
        let mut writer = &stream;
        writeln!(writer, "{}", command).ok()?;
        writer.flush().ok()?;
        let mut line = String::new();
        BufReader::new(&stream).read_line(&mut line).ok()?;
        Some(line.trim().to_string())
    }

    fn send_multi(command: &str) -> Vec<String> {
        let Some(stream) = Self::connect() else {
            return vec![];
        };
        let mut writer = &stream;
        let _ = writeln!(writer, "{}", command);
        let _ = writer.flush();
        let mut lines = Vec::new();
        for line in BufReader::new(&stream).lines().flatten() {
            if line == "END" {
                break;
            }
            lines.push(line);
        }
        lines
    }

    /// Returns true if the primary instance responds.
    pub fn ping() -> bool {
        Self::send_one("PING").map(|r| r == "PONG").unwrap_or(false)
    }

    /// Returns true if the project was found in another window and activated (FOCUSED).
    /// Returns false if the project is not open anywhere (OPEN).
    #[allow(dead_code)]
    pub fn query(path: &Path) -> bool {
        let Some(path) = normalize_project_path(path) else {
            return false;
        };
        let cmd = format!("QUERY {}", path.display());
        Self::send_one(&cmd)
            .map(|r| r == "FOCUSED")
            .unwrap_or(false)
    }

    /// Registers the current process as the owner of the project.
    pub fn register(path: &Path) {
        let Some(path) = normalize_project_path(path) else {
            return;
        };
        let cmd = format!("REGISTER {} {}", std::process::id(), path.display());
        let _ = Self::send_one(&cmd);
    }

    /// Unregisters the current process (all its projects).
    pub fn unregister() {
        let cmd = format!("UNREGISTER {}", std::process::id());
        let _ = Self::send_one(&cmd);
    }

    /// Adds a project to the shared list of recent projects.
    pub fn add_recent(path: &Path) {
        let Some(path) = normalize_project_path(path) else {
            return;
        };
        let cmd = format!("ADD_RECENT {}", path.display());
        let _ = Self::send_one(&cmd);
    }

    /// Sends a request to the primary instance to open a project in a new window.
    pub fn open_in_new_window(path: &Path) -> bool {
        let Some(path) = normalize_project_path(path) else {
            return false;
        };
        let cmd = format!("OPEN_IN_NEW_WINDOW {}", path.display());
        Self::send_one(&cmd).map(|r| r == "OK").unwrap_or(false)
    }

    /// Requests the primary instance to be brought to the foreground.
    pub fn focus_main() -> bool {
        Self::send_one("FOCUS_MAIN")
            .map(|r| r == "OK")
            .unwrap_or(false)
    }

    /// Returns a list of recent projects from the server.
    pub fn recent() -> Vec<PathBuf> {
        Self::send_multi("RECENT")
            .into_iter()
            .map(PathBuf::from)
            .filter(|p| p.is_dir())
            .collect()
    }

    /// Saves the session via the IPC server (serialized write — no race condition).
    /// Returns true if the server accepted the command.
    fn save_session_via_ipc(paths: &[PathBuf]) -> bool {
        let strings: Vec<&str> = paths.iter().filter_map(|p| p.to_str()).collect();
        let Ok(json) = serde_json::to_string(&strings) else {
            return false;
        };
        let cmd = format!("SAVE_SESSION {json}");
        Self::send_one(&cmd).map(|r| r == "OK").unwrap_or(false)
    }
}

// ---------------------------------------------------------------------------
// Session — list of recently opened windows (for restoration at startup)
// ---------------------------------------------------------------------------

/// Saves the list of currently open projects to the session file.
/// If an IPC server is running, the write goes through it (serialized — no race condition).
/// Otherwise (first/only instance), it writes directly.
pub fn save_session(paths: &[PathBuf]) {
    if !Ipc::save_session_via_ipc(paths) {
        save_paths(&session_path(), paths);
    }
}

/// Loads the session and splits paths into existing directories and missing/deleted ones.
/// Returns `(existing, missing)`.
pub fn load_session_checked() -> (Vec<PathBuf>, Vec<PathBuf>) {
    let Ok(content) = std::fs::read_to_string(session_path()) else {
        return (vec![], vec![]);
    };
    let Ok(strings): Result<Vec<String>, _> = serde_json::from_str(&content) else {
        return (vec![], vec![]);
    };
    let mut found = Vec::new();
    let mut missing = Vec::new();
    for s in strings {
        if let Some(canonical) = normalize_project_path_str(&s) {
            found.push(canonical);
        } else {
            missing.push(PathBuf::from(s));
        }
    }
    (found, missing)
}
