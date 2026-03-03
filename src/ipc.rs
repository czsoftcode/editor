use std::collections::HashMap;
use std::io::{BufRead, BufReader, Write};
use std::net::{TcpListener, TcpStream};
#[cfg(windows)]
use std::net::SocketAddr;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};

#[cfg(unix)]
use std::os::unix::net::{UnixListener, UnixStream};

use crate::config;

const IPC_MAX_WORKERS: usize = 16;
const CONFIG_DIR_NAME: &str = "polycredo-editor";

// ---------------------------------------------------------------------------
// Platform Abstraction
// ---------------------------------------------------------------------------

enum IpcListener {
    #[cfg(unix)]
    Unix(UnixListener),
    Tcp(TcpListener),
}

enum IpcStream {
    #[cfg(unix)]
    Unix(UnixStream),
    Tcp(TcpStream),
}

impl IpcStream {
    fn set_read_timeout(&self, timeout: Option<std::time::Duration>) -> std::io::Result<()> {
        match self {
            #[cfg(unix)]
            IpcStream::Unix(s) => s.set_read_timeout(timeout),
            IpcStream::Tcp(s) => s.set_read_timeout(timeout),
        }
    }

    fn set_write_timeout(&self, timeout: Option<std::time::Duration>) -> std::io::Result<()> {
        match self {
            #[cfg(unix)]
            IpcStream::Unix(s) => s.set_write_timeout(timeout),
            IpcStream::Tcp(s) => s.set_write_timeout(timeout),
        }
    }
}

impl Write for IpcStream {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        match self {
            #[cfg(unix)]
            IpcStream::Unix(s) => s.write(buf),
            IpcStream::Tcp(s) => s.write(buf),
        }
    }

    fn flush(&mut self) -> std::io::Result<()> {
        match self {
            #[cfg(unix)]
            IpcStream::Unix(s) => s.flush(),
            IpcStream::Tcp(s) => s.flush(),
        }
    }
}

impl std::io::Read for IpcStream {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        match self {
            #[cfg(unix)]
            IpcStream::Unix(s) => s.read(buf),
            IpcStream::Tcp(s) => s.read(buf),
        }
    }
}

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
    if cfg!(unix) {
        config_dir().join("polycredo-editor.sock")
    } else {
        config_dir().join("ipc.port")
    }
}

fn recent_path() -> PathBuf {
    config_dir().join("recent.json")
}

fn session_path() -> PathBuf {
    config_dir().join("session.json")
}

pub fn plugins_dir() -> PathBuf {
    config_dir().join("plugins")
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
// Connection Helpers
// ---------------------------------------------------------------------------

fn get_ipc_address() -> Option<String> {
    #[cfg(unix)]
    {
        Some(socket_path().to_string_lossy().into_owned())
    }
    #[cfg(not(unix))]
    {
        std::fs::read_to_string(socket_path()).ok()
    }
}

fn connect_to_server() -> Option<IpcStream> {
    #[cfg(unix)]
    {
        UnixStream::connect(socket_path()).ok().map(IpcStream::Unix)
    }
    #[cfg(not(unix))]
    {
        let addr_str = get_ipc_address()?;
        let addr: SocketAddr = addr_str.parse().ok()?;
        TcpStream::connect_timeout(&addr, std::time::Duration::from_secs(1))
            .ok()
            .map(IpcStream::Tcp)
    }
}

// ---------------------------------------------------------------------------
// Recent projects storage
// ---------------------------------------------------------------------------

fn save_paths(file_path: &std::path::Path, paths: &[PathBuf]) {
    if let Some(parent) = file_path.parent()
        && let Err(e) = std::fs::create_dir_all(parent)
    {
        eprintln!(
            "ipc: cannot create config directory {}: {e}",
            parent.display()
        );
        return;
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
    if !file_path.exists() {
        let tmp = file_path.with_extension("tmp");
        if tmp.exists() {
            eprintln!(
                "ipc: recovering {} from {}",
                file_path.display(),
                tmp.display()
            );
            if let Err(e) = std::fs::rename(&tmp, file_path) {
                eprintln!("ipc: recovery failed: {e}");
            }
        }
    }

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
// Per-process socket — window focus
// ---------------------------------------------------------------------------

pub fn process_socket_path(pid: u32) -> PathBuf {
    if cfg!(unix) {
        std::env::temp_dir().join(format!("polycredo-editor_{}.sock", pid))
    } else {
        std::env::temp_dir().join(format!("polycredo-editor_{}.port", pid))
    }
}

pub fn start_process_listener() -> std::sync::mpsc::Receiver<()> {
    let (tx, rx) = std::sync::mpsc::channel();
    let pid = std::process::id();
    let path = process_socket_path(pid);

    #[cfg(unix)]
    {
        let _ = std::fs::remove_file(&path);
        if let Ok(listener) = UnixListener::bind(&path) {
            std::thread::spawn(move || {
                for stream in listener.incoming().flatten() {
                    handle_focus_connection(IpcStream::Unix(stream), tx.clone());
                }
            });
        }
    }
    #[cfg(not(unix))]
    {
        if let Ok(listener) = TcpListener::bind("127.0.0.1:0") {
            if let Ok(addr) = listener.local_addr() {
                let _ = std::fs::write(&path, addr.to_string());
                std::thread::spawn(move || {
                    for stream in listener.incoming().flatten() {
                        handle_focus_connection(IpcStream::Tcp(stream), tx.clone());
                    }
                });
            }
        }
    }
    rx
}

fn handle_focus_connection(stream: IpcStream, tx: std::sync::mpsc::Sender<()>) {
    let mut stream = stream;
    let mut reader = BufReader::new(&mut stream);
    let mut line = String::new();
    if reader.read_line(&mut line).is_ok() && line.trim() == "FOCUS" {
        let _ = tx.send(());
    }
}

#[cfg(unix)]
fn connect_to_process_socket(path: &Path) -> Option<IpcStream> {
    UnixStream::connect(path).ok().map(IpcStream::Unix)
}

#[cfg(not(unix))]
fn connect_to_process_socket(path: &Path) -> Option<IpcStream> {
    let addr_str = std::fs::read_to_string(path).ok()?;
    let addr: SocketAddr = addr_str.parse().ok()?;
    TcpStream::connect_timeout(&addr, std::time::Duration::from_secs(1))
        .ok()
        .map(IpcStream::Tcp)
}

fn focus_process_window(pid: u32) -> bool {
    let path = process_socket_path(pid);
    if let Some(mut stream) = connect_to_process_socket(&path) {
        stream
            .set_write_timeout(Some(std::time::Duration::from_secs(1)))
            .ok();
        writeln!(stream, "FOCUS").is_ok()
    } else {
        false
    }
}

// ---------------------------------------------------------------------------
// Server
// ---------------------------------------------------------------------------

struct ServerState {
    registered: HashMap<PathBuf, u32>,
    recent: Vec<PathBuf>,
    open_tx: std::sync::mpsc::Sender<PathBuf>,
}

fn process_command(line: &str, state: &Arc<Mutex<ServerState>>) -> Vec<String> {
    if line == "PING" {
        return vec!["PONG".into()];
    }

    if line == "RECENT" {
        let st = state.lock().expect("Failed to lock ServerState for RECENT");
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
        let pid_opt = state.lock().unwrap().registered.get(&path).copied();
        if let Some(pid) = pid_opt {
            if focus_process_window(pid) {
                return vec!["FOCUSED".into()];
            } else {
                state.lock().unwrap().registered.remove(&path);
            }
        }
        return vec!["OPEN".into()];
    }

    if let Some(rest) = line.strip_prefix("REGISTER ") {
        if let Some((pid_str, path_str)) = rest.split_once(' ')
            && let Ok(pid) = pid_str.parse::<u32>()
            && let Some(path) = normalize_project_path_str(path_str)
        {
            state.lock().unwrap().registered.insert(path, pid);
            return vec!["OK".into()];
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

    if let Some(path_str) = line.strip_prefix("OPEN_IN_NEW_WINDOW ") {
        if let Some(path) = normalize_project_path_str(path_str) {
            let st = state.lock().unwrap();
            let _ = st.open_tx.send(path);
        }
        return vec!["OK".into()];
    }

    if line == "FOCUS_MAIN" {
        let pid_opt = state.lock().unwrap().registered.values().next().copied();
        if let Some(pid) = pid_opt
            && focus_process_window(pid)
        {
            return vec!["OK".into()];
        }
        return vec!["ERR no registered window".into()];
    }

    vec!["ERR unknown command".into()]
}

fn handle_connection(mut stream: IpcStream, state: Arc<Mutex<ServerState>>) {
    stream
        .set_read_timeout(Some(std::time::Duration::from_secs(5)))
        .ok();
    let mut line = String::new();
    {
        let mut reader = BufReader::new(&mut stream);
        if reader.read_line(&mut line).is_err() {
            return;
        }
    }

    let responses = process_command(line.trim(), &state);
    for resp in responses {
        let _ = writeln!(stream, "{}", resp);
    }
}

fn server_loop(listener: IpcListener, state: Arc<Mutex<ServerState>>) {
    let active_workers = Arc::new(AtomicUsize::new(0));

    match listener {
        #[cfg(unix)]
        IpcListener::Unix(l) => {
            for stream in l.incoming().flatten() {
                spawn_worker(
                    IpcStream::Unix(stream),
                    state.clone(),
                    active_workers.clone(),
                );
            }
        }
        IpcListener::Tcp(l) => {
            for stream in l.incoming().flatten() {
                spawn_worker(
                    IpcStream::Tcp(stream),
                    state.clone(),
                    active_workers.clone(),
                );
            }
        }
    }
}

fn spawn_worker(
    stream: IpcStream,
    state: Arc<Mutex<ServerState>>,
    active_workers: Arc<AtomicUsize>,
) {
    if active_workers.load(Ordering::Relaxed) >= IPC_MAX_WORKERS {
        return;
    }
    active_workers.fetch_add(1, Ordering::SeqCst);
    std::thread::spawn(move || {
        handle_connection(stream, state);
        active_workers.fetch_sub(1, Ordering::SeqCst);
    });
}

pub struct IpcServer {
    sock_path: PathBuf,
}

impl IpcServer {
    #[cfg(unix)]
    fn bind_listener(path: &Path) -> Option<IpcListener> {
        UnixListener::bind(path).ok().map(IpcListener::Unix)
    }

    #[cfg(not(unix))]
    fn bind_listener(path: &Path) -> Option<IpcListener> {
        let l = TcpListener::bind("127.0.0.1:0").ok()?;
        if let Ok(addr) = l.local_addr() {
            let _ = std::fs::write(path, addr.to_string());
        }
        Some(IpcListener::Tcp(l))
    }

    pub fn start() -> Option<(Self, std::sync::mpsc::Receiver<PathBuf>)> {
        let sock = socket_path();
        if let Some(parent) = sock.parent() {
            let _ = std::fs::create_dir_all(parent);
        }

        if Ipc::ping() {
            return None;
        }

        let _ = std::fs::remove_file(&sock);
        let listener = Self::bind_listener(&sock)?;

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
        let _ = std::fs::remove_file(&self.sock_path);
    }
}

// ---------------------------------------------------------------------------
// Client
// ---------------------------------------------------------------------------

pub struct Ipc;

impl Ipc {
    fn send_one(command: &str) -> Option<String> {
        let mut stream = connect_to_server()?;
        writeln!(stream, "{}", command).ok()?;
        stream.flush().ok()?;
        let mut line = String::new();
        BufReader::new(&mut stream).read_line(&mut line).ok()?;
        Some(line.trim().to_string())
    }

    fn send_multi(command: &str) -> Vec<String> {
        let Some(mut stream) = connect_to_server() else {
            return vec![];
        };
        let _ = writeln!(stream, "{}", command);
        let _ = stream.flush();
        let mut lines = Vec::new();
        let mut reader = BufReader::new(&mut stream);
        let mut buffer = String::new();
        while reader.read_line(&mut buffer).is_ok() {
            let line = buffer.trim();
            if line == "END" {
                break;
            }
            lines.push(line.to_string());
            buffer.clear();
        }
        lines
    }

    pub fn ping() -> bool {
        Self::send_one("PING").map(|r| r == "PONG").unwrap_or(false)
    }

    pub fn query(path: &Path) -> bool {
        let Some(path) = normalize_project_path(path) else {
            return false;
        };
        let cmd = format!("QUERY {}", path.display());
        Self::send_one(&cmd)
            .map(|r| r == "FOCUSED")
            .unwrap_or(false)
    }

    pub fn register(path: &Path) {
        let Some(path) = normalize_project_path(path) else {
            return;
        };
        let cmd = format!("REGISTER {} {}", std::process::id(), path.display());
        let _ = Self::send_one(&cmd);
    }

    pub fn unregister() {
        let cmd = format!("UNREGISTER {}", std::process::id());
        let _ = Self::send_one(&cmd);
    }

    pub fn add_recent(path: &Path) {
        let Some(path) = normalize_project_path(path) else {
            return;
        };
        let cmd = format!("ADD_RECENT {}", path.display());
        let _ = Self::send_one(&cmd);
    }

    pub fn open_in_new_window(path: &Path) -> bool {
        let Some(path) = normalize_project_path(path) else {
            return false;
        };
        let cmd = format!("OPEN_IN_NEW_WINDOW {}", path.display());
        Self::send_one(&cmd).map(|r| r == "OK").unwrap_or(false)
    }

    pub fn focus_main() -> bool {
        Self::send_one("FOCUS_MAIN")
            .map(|r| r == "OK")
            .unwrap_or(false)
    }

    pub fn recent() -> Vec<PathBuf> {
        Self::send_multi("RECENT")
            .into_iter()
            .map(PathBuf::from)
            .filter(|p| p.is_dir())
            .collect()
    }

    fn save_session_via_ipc(paths: &[PathBuf]) -> bool {
        let strings: Vec<&str> = paths.iter().filter_map(|p| p.to_str()).collect();
        let Ok(json) = serde_json::to_string(&strings) else {
            return false;
        };
        let cmd = format!("SAVE_SESSION {json}");
        Self::send_one(&cmd).map(|r| r == "OK").unwrap_or(false)
    }
}

pub fn save_session(paths: &[PathBuf]) {
    if !Ipc::save_session_via_ipc(paths) {
        save_paths(&session_path(), paths);
    }
}

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
