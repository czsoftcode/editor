use crate::app::ui::widgets::modal::StandardModal;
use crate::i18n::I18n;
use crate::tr;
use eframe::egui;
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};

/// Wraps an apt-get command with pkexec/sudo-n fallback for privilege elevation.
fn _apt_install_cmd(cmd: &str) -> String {
    format!(
        "if command -v pkexec >/dev/null 2>&1; then \
           pkexec bash -c '{cmd}'; \
         elif sudo -n bash -c '{cmd}' 2>&1; then \
           true; \
         else \
           echo 'ERROR: Root access required. Run manually:'; \
           echo '  sudo {cmd}'; \
           exit 1; \
         fi"
    )
}

#[derive(Clone, PartialEq, Default)]
pub enum InstallStatus {
    #[default]
    Pending,
    Downloading(f32),
    RunningCommand,
    Success,
    Error(String),
}

#[derive(Clone)]
pub enum InstallMethod {
    Download { url: String, target: PathBuf },
    SystemCommand { cmd: String, args: Vec<String> },
}

#[derive(Clone)]
pub struct Dependency {
    pub id: String,
    pub name: String,
    pub description_key: String,
    pub method: InstallMethod,
}

pub struct DependencyWizard {
    pub show: bool,
    pub active_dependency: Option<Dependency>,
    pub status: Arc<Mutex<InstallStatus>>,
    pub is_busy: Arc<AtomicBool>,
    pub output_lines: Arc<Mutex<Vec<String>>>,
}

impl DependencyWizard {
    pub fn new() -> Self {
        Self {
            show: false,
            active_dependency: None,
            status: Arc::new(Mutex::new(InstallStatus::Pending)),
            is_busy: Arc::new(AtomicBool::new(false)),
            output_lines: Arc::new(Mutex::new(Vec::new())),
        }
    }

    pub fn open_for_appimagetool(&mut self) {
        let home = dirs::home_dir().unwrap_or_else(|| PathBuf::from("/"));
        let target = home.join(".local/bin/appimagetool");

        self.active_dependency = Some(Dependency {
            id: "appimagetool".into(),
            name: "appimagetool".into(),
            description_key: "dep-wizard-appimagetool-desc".into(),
            method: InstallMethod::Download {
                url: "https://github.com/AppImage/appimagetool/releases/download/continuous/appimagetool-x86_64.AppImage".into(),
                target
            },
        });
        self.reset_status();
        self.show = true;
    }

    fn reset_status(&mut self) {
        let mut s = self.status.lock().unwrap();
        *s = InstallStatus::Pending;
        self.is_busy.store(false, Ordering::SeqCst);
        self.output_lines.lock().unwrap().clear();
    }

    pub fn render(&mut self, ctx: &egui::Context, i18n: &I18n) {
        if !self.show {
            return;
        }

        let dep = match self.active_dependency.clone() {
            Some(d) => d,
            None => return,
        };

        let in_progress = !matches!(*self.status.lock().unwrap(), InstallStatus::Pending);
        let (modal_w, modal_h) = if in_progress {
            (660.0, 520.0)
        } else {
            (500.0, 320.0)
        };
        let modal = StandardModal::new(tr!(i18n, "dep-wizard-title"), "dependency_wizard")
            .with_size(modal_w, modal_h);

        let mut local_show = self.show;
        modal.show(ctx, &mut local_show, |ui| {
            modal.ui_body(ui, |ui| {
                let status = self.status.lock().unwrap().clone();

                ui.vertical_centered(|ui| {
                    ui.add_space(10.0);
                    ui.heading(&dep.name);
                    ui.add_space(10.0);

                    match &status {
                        InstallStatus::Pending => {
                            ui.label(tr!(i18n, &dep.description_key, tool = &dep.name));
                            ui.add_space(10.0);
                            match &dep.method {
                                InstallMethod::Download { target, .. } => {
                                    ui.label(tr!(
                                        i18n,
                                        "dep-wizard-install-question",
                                        path = target.display().to_string()
                                    ));
                                }
                                InstallMethod::SystemCommand { .. } => {
                                    ui.label(tr!(i18n, "dep-wizard-install-cmd-question"));
                                }
                            }
                        }
                        InstallStatus::Downloading(p) => {
                            ui.label(tr!(i18n, "dep-wizard-status-downloading"));
                            ui.add(egui::ProgressBar::new(*p).show_percentage());
                        }
                        InstallStatus::RunningCommand => {
                            ui.horizontal(|ui| {
                                ui.add(egui::Spinner::new());
                                ui.label(tr!(i18n, "dep-wizard-status-running"));
                            });
                        }
                        InstallStatus::Success => {
                            ui.colored_label(
                                egui::Color32::GREEN,
                                tr!(i18n, "dep-wizard-status-success"),
                            );
                        }
                        InstallStatus::Error(e) => {
                            ui.colored_label(
                                egui::Color32::RED,
                                tr!(i18n, "dep-wizard-status-error", error = e.clone()),
                            );
                        }
                    }
                });

                // Terminal output
                let output = self.output_lines.lock().unwrap().clone();
                if !output.is_empty() {
                    ui.add_space(6.0);
                    ui.separator();
                    let auto_scroll = matches!(
                        status,
                        InstallStatus::RunningCommand | InstallStatus::Downloading(_)
                    );
                    let bg = ui.visuals().extreme_bg_color;
                    egui::ScrollArea::vertical()
                        .id_salt("dep_wizard_output")
                        .max_height(320.0)
                        .stick_to_bottom(auto_scroll)
                        .show(ui, |ui| {
                            let rect = ui.max_rect();
                            ui.painter().rect_filled(rect, 2.0, bg);
                            ui.set_width(rect.width());
                            for line in &output {
                                ui.label(
                                    egui::RichText::new(line)
                                        .monospace()
                                        .size(11.0)
                                        .color(egui::Color32::from_rgb(180, 220, 180)),
                                );
                            }
                        });
                }
            });

            modal.ui_footer_actions(ui, i18n, |f| {
                let status = self.status.lock().unwrap().clone();

                match status {
                    InstallStatus::Pending => {
                        if f.cancel() {
                            self.show = false;
                        }
                        let btn_text = match &dep.method {
                            InstallMethod::Download { .. } => tr!(i18n, "dep-wizard-btn-install"),
                            InstallMethod::SystemCommand { .. } => {
                                tr!(i18n, "dep-wizard-btn-run-cmd")
                            }
                        };
                        if f.ui.button(btn_text).clicked() {
                            self.start_install(ctx.clone());
                        }
                    }
                    InstallStatus::Downloading(_) | InstallStatus::RunningCommand => {
                        f.ui.add(egui::Spinner::new());
                    }
                    InstallStatus::Success | InstallStatus::Error(_) => {
                        if f.close() {
                            self.show = false;
                        }
                    }
                }
                None::<()>
            });
        });
        self.show = local_show;
    }

    fn start_install(&self, ctx: egui::Context) {
        if self.is_busy.swap(true, Ordering::SeqCst) {
            return;
        }

        let dep = self.active_dependency.as_ref().unwrap().clone();
        let status_arc = Arc::clone(&self.status);
        let is_busy = Arc::clone(&self.is_busy);
        let output_lines = Arc::clone(&self.output_lines);

        std::thread::spawn(move || {
            let rt = tokio::runtime::Runtime::new().unwrap();
            rt.block_on(async {
                match dep.method {
                    InstallMethod::Download { url, target } => {
                        *status_arc.lock().unwrap() = InstallStatus::Downloading(0.1);
                        output_lines
                            .lock()
                            .unwrap()
                            .push(format!("Downloading: {}", url));
                        output_lines
                            .lock()
                            .unwrap()
                            .push(format!("Destination: {}", target.display()));
                        ctx.request_repaint();

                        if let Some(parent) = target.parent() {
                            let _ = std::fs::create_dir_all(parent);
                        }

                        let result = tokio::process::Command::new("curl")
                            .arg("-L")
                            .arg("-o")
                            .arg(&target)
                            .arg(&url)
                            .output()
                            .await;

                        match result {
                            Ok(out) if out.status.success() => {
                                #[cfg(unix)]
                                {
                                    use std::os::unix::fs::PermissionsExt;
                                    if let Ok(metadata) = std::fs::metadata(&target) {
                                        let mut perms = metadata.permissions();
                                        perms.set_mode(0o755);
                                        let _ = std::fs::set_permissions(&target, perms);
                                    }
                                }
                                #[cfg(windows)]
                                if target.extension().map_or(false, |ext| ext == "exe") {
                                    let _ = std::process::Command::new(&target).spawn();
                                }
                                output_lines
                                    .lock()
                                    .unwrap()
                                    .push("✓ Download complete.".into());
                                *status_arc.lock().unwrap() = InstallStatus::Success;
                            }
                            Ok(out) => {
                                let stderr = String::from_utf8_lossy(&out.stderr).to_string();
                                if !stderr.trim().is_empty() {
                                    output_lines.lock().unwrap().push(stderr);
                                }
                                output_lines
                                    .lock()
                                    .unwrap()
                                    .push("✗ Download failed.".into());
                                *status_arc.lock().unwrap() =
                                    InstallStatus::Error("Download failed".into());
                            }
                            Err(e) => {
                                output_lines.lock().unwrap().push(format!("✗ Error: {}", e));
                                *status_arc.lock().unwrap() = InstallStatus::Error(e.to_string());
                            }
                        }
                    }
                    InstallMethod::SystemCommand { cmd, args } => {
                        *status_arc.lock().unwrap() = InstallStatus::RunningCommand;
                        output_lines
                            .lock()
                            .unwrap()
                            .push(format!("$ {} {}", cmd, args.join(" ")));
                        ctx.request_repaint();

                        use tokio::io::{AsyncBufReadExt, BufReader};
                        let child = tokio::process::Command::new(&cmd)
                            .args(&args)
                            .stdout(std::process::Stdio::piped())
                            .stderr(std::process::Stdio::piped())
                            .spawn();

                        match child {
                            Ok(mut child) => {
                                let stdout = child.stdout.take().map(BufReader::new);
                                let stderr = child.stderr.take().map(BufReader::new);

                                let lines1 = Arc::clone(&output_lines);
                                let ctx1 = ctx.clone();
                                let stdout_task = tokio::spawn(async move {
                                    if let Some(reader) = stdout {
                                        let mut lines = reader.lines();
                                        while let Ok(Some(line)) = lines.next_line().await {
                                            lines1.lock().unwrap().push(line);
                                            ctx1.request_repaint();
                                        }
                                    }
                                });

                                let lines2 = Arc::clone(&output_lines);
                                let ctx2 = ctx.clone();
                                let stderr_task = tokio::spawn(async move {
                                    if let Some(reader) = stderr {
                                        let mut lines = reader.lines();
                                        while let Ok(Some(line)) = lines.next_line().await {
                                            lines2.lock().unwrap().push(line);
                                            ctx2.request_repaint();
                                        }
                                    }
                                });

                                let _ = tokio::join!(stdout_task, stderr_task);

                                match child.wait().await {
                                    Ok(exit) if exit.success() => {
                                        output_lines.lock().unwrap().push("✓ Done.".into());
                                        *status_arc.lock().unwrap() = InstallStatus::Success;
                                    }
                                    Ok(exit) => {
                                        output_lines
                                            .lock()
                                            .unwrap()
                                            .push(format!("✗ Exited with: {}", exit));
                                        *status_arc.lock().unwrap() =
                                            InstallStatus::Error(format!("Exit: {}", exit));
                                    }
                                    Err(e) => {
                                        output_lines
                                            .lock()
                                            .unwrap()
                                            .push(format!("✗ Error: {}", e));
                                        *status_arc.lock().unwrap() =
                                            InstallStatus::Error(e.to_string());
                                    }
                                }
                            }
                            Err(e) => {
                                output_lines
                                    .lock()
                                    .unwrap()
                                    .push(format!("✗ Failed to start: {}", e));
                                *status_arc.lock().unwrap() = InstallStatus::Error(e.to_string());
                            }
                        }
                    }
                }
                is_busy.store(false, Ordering::SeqCst);
                ctx.request_repaint();
            });
        });
    }
}
