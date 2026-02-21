//! Module for handling the Language Server Protocol (LSP) client using async-lsp.
use async_lsp::lsp_types::notification::{Initialized, PublishDiagnostics};
use async_lsp::lsp_types::request::Initialize;
use async_lsp::lsp_types::{
    ClientCapabilities, ClientInfo, Diagnostic, InitializeParams, InitializedParams,
    ServerCapabilities, TraceValue, Url,
};
use async_lsp::router::Router;
use async_lsp::{MainLoop, ServerSocket};
use eframe::egui;
use std::collections::HashMap;
use std::ops::ControlFlow;
use std::process::Stdio;
use std::sync::{Arc, Mutex};
use tokio::process::Command;
use tokio::runtime::Runtime;
use tokio_util::compat::{TokioAsyncReadCompatExt, TokioAsyncWriteCompatExt};
use tower::ServiceBuilder;

/// Shared state between the UI thread and the LSP thread, containing diagnostics from the server.
pub type DiagnosticsMap = Arc<Mutex<HashMap<Url, Vec<Diagnostic>>>>;

/// Represents the LSP client, handling communication with the language server.
pub struct LspClient {
    /// The Tokio runtime for our async tasks.
    pub _runtime: Runtime,
    /// Shared storage for diagnostics received from the server.
    diagnostics: DiagnosticsMap,
    /// Capabilities of the language server.
    _server_capabilities: Arc<Mutex<Option<ServerCapabilities>>>,
    /// Client socket to send requests to the server.
    _client_socket: ServerSocket,
}

impl LspClient {
    /// Creates a new LspClient and starts the language server process and initialization.
    pub fn new(egui_ctx: egui::Context, root_uri: Url) -> Option<Self> {
        let runtime = tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .build()
            .ok()?;

        let rust_analyzer_path =
            std::env::var("RUST_ANALYZER_PATH").unwrap_or_else(|_| "rust-analyzer".to_string());

        let mut process = Command::new(&rust_analyzer_path)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::inherit())
            .spawn()
            .ok()?;

        let stdin = process.stdin.take()?;
        let stdout = process.stdout.take()?;

        let diagnostics = DiagnosticsMap::default();
        let diagnostics_clone = diagnostics.clone();
        let egui_ctx_clone = egui_ctx.clone();

        let (main_loop, client_socket) = MainLoop::new_client(|_socket| {
            let mut router = Router::new(());
            let diagnostics_handler = diagnostics_clone.clone();
            let egui_ctx_handler = egui_ctx_clone.clone();

            router.notification::<PublishDiagnostics>(move |_, params| {
                let mut diag_map = diagnostics_handler.lock().unwrap();
                diag_map.insert(params.uri, params.diagnostics);
                egui_ctx_handler.request_repaint();
                ControlFlow::Continue(())
            });

            ServiceBuilder::new().service(router)
        });

        let server_capabilities = Arc::new(Mutex::new(None));
        let server_capabilities_clone = server_capabilities.clone();
        let client_socket_init = client_socket.clone();

        runtime.spawn(async move {
            // Use compat() to convert Tokio types to futures-io types
            if let Err(e) = main_loop
                .run_buffered(stdout.compat(), stdin.compat_write())
                .await
            {
                eprintln!("LSP main loop failed: {:?}", e);
            }
        });

        // Start initialization in the background
        runtime.spawn(async move {
            #[allow(deprecated)]
            let initialize_params = InitializeParams {
                process_id: Some(std::process::id()),
                root_uri: Some(root_uri),
                root_path: None,
                client_info: Some(ClientInfo {
                    name: "PolyCredo Editor".to_string(),
                    version: Some(env!("CARGO_PKG_VERSION").to_string()),
                }),
                capabilities: ClientCapabilities::default(),
                trace: Some(TraceValue::Off),
                workspace_folders: None,
                initialization_options: None,
                locale: None,
                work_done_progress_params: Default::default(),
            };

            match client_socket_init
                .request::<Initialize>(initialize_params)
                .await
            {
                Ok(result) => {
                    let mut caps = server_capabilities_clone.lock().unwrap();
                    *caps = Some(result.capabilities);
                    let _ = client_socket_init.notify::<Initialized>(InitializedParams {});
                }
                Err(e) => {
                    eprintln!("LSP initialization request failed: {:?}", e);
                }
            }
        });

        Some(Self {
            _runtime: runtime,
            diagnostics,
            _server_capabilities: server_capabilities,
            _client_socket: client_socket,
        })
    }

    /// Returns a thread-safe handle to the diagnostics map.
    pub fn diagnostics(&self) -> &DiagnosticsMap {
        &self.diagnostics
    }

    /// Checks if rust-analyzer is available in the system (synchronously).
    pub fn is_installed() -> bool {
        let rust_analyzer_path =
            std::env::var("RUST_ANALYZER_PATH").unwrap_or_else(|_| "rust-analyzer".to_string());

        // Use std::process::Command for a synchronous check
        std::process::Command::new("sh")
            .arg("-c")
            .arg(format!("command -v {}", rust_analyzer_path))
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status()
            .map(|s| s.success())
            .unwrap_or(false)
    }

    /// Attempts to install rust-analyzer using rustup.
    pub async fn install_rust_analyzer() -> Result<(), String> {
        let status = Command::new("rustup")
            .args(["component", "add", "rust-analyzer"])
            .status()
            .await
            .map_err(|e| e.to_string())?;

        if status.success() {
            Ok(())
        } else {
            Err(format!("rustup exited with status: {}", status))
        }
    }
}
