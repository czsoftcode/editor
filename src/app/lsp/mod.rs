//! Module for handling the Language Server Protocol (LSP) client using async-lsp.
use async_lsp::lsp_types::notification::{
    DidChangeTextDocument, DidCloseTextDocument, DidOpenTextDocument, Initialized,
    PublishDiagnostics,
};
use async_lsp::lsp_types::request::{
    Completion, GotoDefinition, HoverRequest, Initialize, References,
};
use async_lsp::lsp_types::{
    ClientCapabilities, ClientInfo, CompletionClientCapabilities, CompletionItemCapability,
    CompletionParams, CompletionResponse, Diagnostic, DidChangeTextDocumentParams,
    DidCloseTextDocumentParams, DidOpenTextDocumentParams, GotoCapability, GotoDefinitionParams,
    GotoDefinitionResponse, Hover, HoverClientCapabilities, HoverParams, InitializeParams,
    InitializedParams, Location, MarkupKind, ReferenceClientCapabilities, ReferenceContext,
    ReferenceParams, ServerCapabilities, TextDocumentClientCapabilities,
    TextDocumentContentChangeEvent, TextDocumentIdentifier, TextDocumentItem,
    TextDocumentPositionParams, TextDocumentSyncClientCapabilities, TraceValue, Url,
    VersionedTextDocumentIdentifier, WorkspaceFolder,
};
use async_lsp::router::Router;
use async_lsp::{MainLoop, ServerSocket};
use eframe::egui;
use std::collections::HashMap;
use std::ops::ControlFlow;
use std::process::Stdio;
use std::sync::atomic::{AtomicBool, Ordering};
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
    /// Set to true after the initialize/initialized handshake completes.
    initialized: Arc<AtomicBool>,
}

impl LspClient {
    /// Creates a new LspClient and starts the language server process and initialization.
    pub fn new(egui_ctx: egui::Context, root_uri: Url) -> Option<Self> {
        let runtime = tokio::runtime::Builder::new_multi_thread()
            .worker_threads(2)
            .enable_io()
            .build()
            .ok()?;

        let _guard = runtime.enter();

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
                // Throttle repaints — at most 10 per second to avoid CPU spikes
                // when rust-analyzer sends many rapid diagnostics during indexing.
                egui_ctx_handler.request_repaint_after(std::time::Duration::from_millis(100));
                ControlFlow::Continue(())
            });

            ServiceBuilder::new().service(router)
        });

        let server_capabilities = Arc::new(Mutex::new(None));
        let server_capabilities_clone = server_capabilities.clone();
        let client_socket_init = client_socket.clone();

        let initialized = Arc::new(AtomicBool::new(false));
        let initialized_clone = initialized.clone();

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
                root_uri: Some(root_uri.clone()),
                root_path: None,
                client_info: Some(ClientInfo {
                    name: "PolyCredo Editor".to_string(),
                    version: Some(env!("CARGO_PKG_VERSION").to_string()),
                }),
                capabilities: ClientCapabilities {
                    text_document: Some(TextDocumentClientCapabilities {
                        synchronization: Some(TextDocumentSyncClientCapabilities {
                            dynamic_registration: Some(false),
                            will_save: Some(false),
                            will_save_wait_until: Some(false),
                            did_save: Some(false),
                        }),
                        hover: Some(HoverClientCapabilities {
                            dynamic_registration: Some(false),
                            content_format: Some(vec![MarkupKind::Markdown, MarkupKind::PlainText]),
                        }),
                        definition: Some(GotoCapability {
                            dynamic_registration: Some(false),
                            link_support: Some(false),
                        }),
                        references: Some(ReferenceClientCapabilities {
                            dynamic_registration: Some(false),
                        }),
                        completion: Some(CompletionClientCapabilities {
                            dynamic_registration: Some(false),
                            completion_item: Some(CompletionItemCapability {
                                snippet_support: Some(false),
                                ..Default::default()
                            }),
                            context_support: Some(false),
                            ..Default::default()
                        }),
                        ..Default::default()
                    }),
                    ..Default::default()
                },
                trace: Some(TraceValue::Off),
                workspace_folders: Some(vec![WorkspaceFolder {
                    uri: root_uri,
                    name: "workspace".to_string(),
                }]),
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
                    initialized_clone.store(true, Ordering::Release);
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
            initialized,
        })
    }

    /// Returns true once the initialize/initialized handshake with the server is complete.
    pub fn is_initialized(&self) -> bool {
        self.initialized.load(Ordering::Acquire)
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

    /// Notifies the language server that a text document was opened.
    pub fn notify_did_open(&self, uri: Url, language_id: &str, version: i32, text: String) {
        let socket = self._client_socket.clone();
        let language_id = language_id.to_string();
        self._runtime.spawn(async move {
            let _ = socket.notify::<DidOpenTextDocument>(DidOpenTextDocumentParams {
                text_document: TextDocumentItem {
                    uri,
                    language_id,
                    version,
                    text,
                },
            });
        });
    }

    /// Notifies the language server about a text document change (full sync).
    pub fn notify_did_change(&self, uri: Url, version: i32, text: String) {
        let socket = self._client_socket.clone();
        self._runtime.spawn(async move {
            let _ = socket.notify::<DidChangeTextDocument>(DidChangeTextDocumentParams {
                text_document: VersionedTextDocumentIdentifier { uri, version },
                content_changes: vec![TextDocumentContentChangeEvent {
                    range: None,
                    range_length: None,
                    text,
                }],
            });
        });
    }

    /// Notifies the language server that a text document was closed.
    pub fn notify_did_close(&self, uri: Url) {
        let socket = self._client_socket.clone();
        self._runtime.spawn(async move {
            let _ = socket.notify::<DidCloseTextDocument>(DidCloseTextDocumentParams {
                text_document: TextDocumentIdentifier { uri },
            });
        });
    }

    /// Sends a hover request for the given document position.
    /// Returns a channel that will receive the hover result (or None if not available).
    pub fn request_hover(
        &self,
        uri: Url,
        position: async_lsp::lsp_types::Position,
    ) -> std::sync::mpsc::Receiver<Option<Hover>> {
        let (tx, rx) = std::sync::mpsc::channel();
        let socket = self._client_socket.clone();
        self._runtime.spawn(async move {
            let result = socket
                .request::<HoverRequest>(HoverParams {
                    text_document_position_params: TextDocumentPositionParams {
                        text_document: TextDocumentIdentifier { uri },
                        position,
                    },
                    work_done_progress_params: Default::default(),
                })
                .await
                .ok()
                .flatten();
            let _ = tx.send(result);
        });
        rx
    }

    /// Sends a go-to-definition request for the given document position.
    pub fn request_goto_definition(
        &self,
        uri: Url,
        position: async_lsp::lsp_types::Position,
    ) -> std::sync::mpsc::Receiver<Option<GotoDefinitionResponse>> {
        let (tx, rx) = std::sync::mpsc::channel();
        let socket = self._client_socket.clone();
        self._runtime.spawn(async move {
            let result = socket
                .request::<GotoDefinition>(GotoDefinitionParams {
                    text_document_position_params: TextDocumentPositionParams {
                        text_document: TextDocumentIdentifier { uri },
                        position,
                    },
                    work_done_progress_params: Default::default(),
                    partial_result_params: Default::default(),
                })
                .await
                .ok()
                .flatten();
            let _ = tx.send(result);
        });
        rx
    }

    /// Sends a find-references request for the given document position.
    pub fn request_references(
        &self,
        uri: Url,
        position: async_lsp::lsp_types::Position,
    ) -> std::sync::mpsc::Receiver<Option<Vec<Location>>> {
        let (tx, rx) = std::sync::mpsc::channel();
        let socket = self._client_socket.clone();
        self._runtime.spawn(async move {
            let result = socket
                .request::<References>(ReferenceParams {
                    text_document_position: TextDocumentPositionParams {
                        text_document: TextDocumentIdentifier { uri },
                        position,
                    },
                    work_done_progress_params: Default::default(),
                    partial_result_params: Default::default(),
                    context: ReferenceContext {
                        include_declaration: true,
                    },
                })
                .await
                .ok()
                .flatten();
            let _ = tx.send(result);
        });
        rx
    }

    /// Sends a completion request for the given document position.
    pub fn request_completion(
        &self,
        uri: Url,
        position: async_lsp::lsp_types::Position,
    ) -> std::sync::mpsc::Receiver<Option<CompletionResponse>> {
        let (tx, rx) = std::sync::mpsc::channel();
        let socket = self._client_socket.clone();
        self._runtime.spawn(async move {
            let result = socket
                .request::<Completion>(CompletionParams {
                    text_document_position: TextDocumentPositionParams {
                        text_document: TextDocumentIdentifier { uri },
                        position,
                    },
                    work_done_progress_params: Default::default(),
                    partial_result_params: Default::default(),
                    context: None,
                })
                .await
                .ok()
                .flatten();
            let _ = tx.send(result);
        });
        rx
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
