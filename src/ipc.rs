//! IPC (Inter-Process Communication) pro správu více instancí editoru.
//!
//! První spuštěná instance se stane primárním serverem — naslouchá na Unix socketu,
//! drží seznam otevřených projektů a nedávných projektů.
//! Každá další instance je klientem a komunikuje se serverem.
//!
//! Protokol (textové příkazy, každý na jednom řádku):
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
use std::sync::{Arc, Mutex};

use serde_json;

use crate::config;

// ---------------------------------------------------------------------------
// Cesty k souborům
// ---------------------------------------------------------------------------

fn config_dir() -> PathBuf {
    dirs::config_dir()
        .unwrap_or_else(std::env::temp_dir)
        .join("rust_editor")
}

fn socket_path() -> PathBuf {
    config_dir().join("rust_editor.sock")
}

fn recent_path() -> PathBuf {
    config_dir().join("recent.json")
}

fn session_path() -> PathBuf {
    config_dir().join("session.json")
}

// ---------------------------------------------------------------------------
// Ukládání nedávných projektů
// ---------------------------------------------------------------------------

fn save_paths(file_path: &std::path::Path, paths: &[PathBuf]) {
    if let Some(parent) = file_path.parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    let strings: Vec<&str> = paths.iter().filter_map(|p| p.to_str()).collect();
    let Ok(content) = serde_json::to_string(&strings) else { return };
    let tmp = file_path.with_extension("tmp");
    if std::fs::write(&tmp, content).is_ok() {
        let _ = std::fs::rename(&tmp, file_path);
    }
}

fn load_paths(file_path: &std::path::Path) -> Vec<PathBuf> {
    let Ok(content) = std::fs::read_to_string(file_path) else { return vec![] };
    let Ok(strings): Result<Vec<String>, _> = serde_json::from_str(&content) else { return vec![] };
    strings.into_iter().map(PathBuf::from).filter(|p| p.is_dir()).collect()
}

fn save_recent(recent: &[PathBuf]) {
    save_paths(&recent_path(), recent);
}

fn load_recent() -> Vec<PathBuf> {
    load_paths(&recent_path())
}

// ---------------------------------------------------------------------------
// Per-process socket — fokus okna bez externích nástrojů
// ---------------------------------------------------------------------------

/// Cesta k socketu konkrétního procesu (pro příkaz FOCUS).
pub fn process_socket_path(pid: u32) -> PathBuf {
    std::env::temp_dir().join(format!("rust_editor_{}.sock", pid))
}

/// Spustí listener na per-process socketu. Vrátí kanál, na kterém main vlákno
/// dostává signál ke zkusení `ViewportCommand::Focus`.
pub fn start_process_listener() -> std::sync::mpsc::Receiver<()> {
    let (tx, rx) = std::sync::mpsc::channel();
    let sock = process_socket_path(std::process::id());
    let _ = std::fs::remove_file(&sock);
    if let Ok(listener) = UnixListener::bind(&sock) {
        std::thread::spawn(move || {
            for stream in listener.incoming().flatten() {
                let tx = tx.clone();
                std::thread::spawn(move || {
                    let reader = BufReader::new(&stream);
                    if let Some(Ok(line)) = reader.lines().next() {
                        if line.trim() == "FOCUS" {
                            let _ = tx.send(());
                        }
                    }
                });
            }
        });
    }
    rx
}

/// Pošle příkaz FOCUS cílovému procesu přes jeho per-process socket.
/// Vrací true pokud se spojení podařilo (proces je naživu).
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

#[derive(Default)]
struct ServerState {
    /// Mapování cesta → PID otevřeného projektu
    registered: HashMap<PathBuf, u32>,
    /// Nedávné projekty (max 10)
    recent: Vec<PathBuf>,
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
        let path = PathBuf::from(path_str);
        // Přečíst PID bez držení zámku při síťovém volání
        let pid_opt = state.lock().unwrap().registered.get(&path).copied();
        if let Some(pid) = pid_opt {
            if focus_process_window(pid) {
                // Proces odpověděl — požádal své okno o fokus
                return vec!["FOCUSED".into()];
            } else {
                // Proces je mrtvý — vyčistit registraci
                state.lock().unwrap().registered.remove(&path);
            }
        }
        return vec!["OPEN".into()];
    }

    if let Some(rest) = line.strip_prefix("REGISTER ") {
        if let Some((pid_str, path_str)) = rest.split_once(' ') {
            if let Ok(pid) = pid_str.parse::<u32>() {
                let path = PathBuf::from(path_str);
                let mut st = state.lock().unwrap();
                // Přidat registraci (nečistit ostatní záznamy tohoto PID — jeden PID může
                // vlastnit více projektů v multi-viewport architektuře)
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
        let path = PathBuf::from(path_str);
        if path.is_dir() {
            let mut st = state.lock().unwrap();
            st.recent.retain(|p| p != &path);
            st.recent.insert(0, path);
            st.recent.truncate(config::MAX_RECENT_PROJECTS);
            save_recent(&st.recent);
        }
        return vec!["OK".into()];
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
    for stream in listener.incoming().flatten() {
        let state = state.clone();
        std::thread::spawn(move || handle_connection(stream, state));
    }
}

/// Handle ke správě primární instance (server). Při drop odstraní socket.
pub struct IpcServer {
    sock_path: PathBuf,
}

impl IpcServer {
    /// Pokusí se stát primární instancí.
    /// Vrací `Some(IpcServer)` pokud se to podaří, `None` pokud primár již existuje.
    pub fn start() -> Option<Self> {
        let sock = socket_path();
        if let Some(parent) = sock.parent() {
            let _ = std::fs::create_dir_all(parent);
        }

        // Ověřit, zda primár už běží
        if Ipc::ping() {
            return None; // Jiná primární instance existuje
        }

        // Odstranit případný starý (nefunkční) socket
        let _ = std::fs::remove_file(&sock);

        let listener = UnixListener::bind(&sock).ok()?;

        let state = Arc::new(Mutex::new(ServerState {
            registered: HashMap::new(),
            recent: load_recent(),
        }));

        std::thread::spawn(move || server_loop(listener, state));

        Some(IpcServer { sock_path: sock })
    }
}

impl Drop for IpcServer {
    fn drop(&mut self) {
        let _ = std::fs::remove_file(&self.sock_path);
    }
}

// ---------------------------------------------------------------------------
// Klient
// ---------------------------------------------------------------------------

/// Bezstavový IPC klient — každé volání otevře nové spojení.
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

    /// Vrací true pokud primární instance odpovídá.
    pub fn ping() -> bool {
        Self::send_one("PING").map(|r| r == "PONG").unwrap_or(false)
    }

    /// Vrací true pokud projekt byl nalezen v jiném okně a aktivován (FOCUSED).
    /// Vrací false pokud projekt není nikde otevřen (OPEN).
    #[allow(dead_code)]
    pub fn query(path: &Path) -> bool {
        let cmd = format!("QUERY {}", path.display());
        Self::send_one(&cmd).map(|r| r == "FOCUSED").unwrap_or(false)
    }

    /// Zaregistruje aktuální proces jako vlastníka projektu.
    pub fn register(path: &Path) {
        let cmd = format!("REGISTER {} {}", std::process::id(), path.display());
        let _ = Self::send_one(&cmd);
    }

    /// Odregistruje aktuální proces (všechny jeho projekty).
    pub fn unregister() {
        let cmd = format!("UNREGISTER {}", std::process::id());
        let _ = Self::send_one(&cmd);
    }

    /// Přidá projekt do sdíleného seznamu nedávných projektů.
    pub fn add_recent(path: &Path) {
        let cmd = format!("ADD_RECENT {}", path.display());
        let _ = Self::send_one(&cmd);
    }

    /// Vrátí seznam nedávných projektů ze serveru.
    pub fn recent() -> Vec<PathBuf> {
        Self::send_multi("RECENT")
            .into_iter()
            .map(PathBuf::from)
            .filter(|p| p.is_dir())
            .collect()
    }
}

// ---------------------------------------------------------------------------
// Session — seznam naposledy otevřených oken (pro restore při spuštění)
// ---------------------------------------------------------------------------

/// Uloží seznam aktuálně otevřených projektů do session souboru.
pub fn save_session(paths: &[PathBuf]) {
    save_paths(&session_path(), paths);
}

/// Načte seznam projektů uložených v session souboru.
pub fn load_session() -> Vec<PathBuf> {
    load_paths(&session_path())
}
