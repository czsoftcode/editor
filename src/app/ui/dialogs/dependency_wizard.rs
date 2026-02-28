use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::sync::atomic::{AtomicBool, Ordering};
use eframe::egui;
use crate::app::ui::widgets::modal::StandardModal;
use crate::i18n::I18n;
use crate::tr;

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
}

impl DependencyWizard {
    pub fn new() -> Self {
        Self {
            show: false,
            active_dependency: None,
            status: Arc::new(Mutex::new(InstallStatus::Pending)),
            is_busy: Arc::new(AtomicBool::new(false)),
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

    pub fn open_for_nsis(&mut self) {
        #[cfg(windows)]
        let method = InstallMethod::Download {
            url: "https://downloads.sourceforge.net/project/nsis/NSIS%203/3.10/nsis-3.10-setup.exe".into(),
            target: dirs::cache_dir().unwrap_or_default().join("nsis-setup.exe"),
        };
        #[cfg(not(windows))]
        let method = InstallMethod::SystemCommand {
            cmd: "pkexec".into(),
            args: vec!["apt-get".into(), "install".into(), "-y".into(), "nsis".into()],
        };

        self.active_dependency = Some(Dependency {
            id: "nsis".into(),
            name: "NSIS".into(),
            description_key: "dep-wizard-nsis-desc".into(),
            method,
        });
        self.reset_status();
        self.show = true;
    }

    pub fn open_for_rpm(&mut self) {
        self.active_dependency = Some(Dependency {
            id: "rpm".into(),
            name: "rpm-build".into(),
            description_key: "dep-wizard-rpm-desc".into(),
            method: InstallMethod::SystemCommand {
                cmd: "pkexec".into(),
                args: vec!["dnf".into(), "install".into(), "-y".into(), "rpm-build".into(), "rpm".into()],
            },
        });
        self.reset_status();
        self.show = true;
    }

    pub fn open_for_generate_rpm(&mut self) {
        self.active_dependency = Some(Dependency {
            id: "generate-rpm".into(),
            name: "cargo-generate-rpm".into(),
            description_key: "dep-wizard-generate-rpm-desc".into(),
            method: InstallMethod::SystemCommand {
                cmd: "cargo".into(),
                args: vec!["install".into(), "cargo-generate-rpm".into()],
            },
        });
        self.reset_status();
        self.show = true;
    }

    pub fn open_for_appimage(&mut self) {
        self.active_dependency = Some(Dependency {
            id: "appimage".into(),
            name: "cargo-appimage".into(),
            description_key: "dep-wizard-appimage-desc".into(),
            method: InstallMethod::SystemCommand {
                cmd: "cargo".into(),
                args: vec!["install".into(), "cargo-appimage".into()],
            },
        });
        self.reset_status();
        self.show = true;
    }

    pub fn open_for_tar(&mut self) {
        self.active_dependency = Some(Dependency {
            id: "tar".into(),
            name: "tar".into(),
            description_key: "dep-wizard-tar-desc".into(),
            method: InstallMethod::SystemCommand {
                cmd: "pkexec".into(),
                args: vec!["apt-get".into(), "install".into(), "-y".into(), "tar".into()],
            },
        });
        self.reset_status();
        self.show = true;
    }

    pub fn open_for_deb_tools(&mut self) {
        self.active_dependency = Some(Dependency {
            id: "deb".into(),
            name: "Debian Build Tools".into(),
            description_key: "dep-wizard-deb-desc".into(),
            method: InstallMethod::SystemCommand {
                cmd: "pkexec".into(),
                args: vec!["apt-get".into(), "install".into(), "-y".into(), "dpkg-dev".into(), "build-essential".into(), "fakeroot".into()],
            },
        });
        self.reset_status();
        self.show = true;
    }

    pub fn open_for_xwin(&mut self) {
        self.active_dependency = Some(Dependency {
            id: "xwin".into(),
            name: "cargo-xwin".into(),
            description_key: "dep-wizard-xwin-desc".into(),
            method: InstallMethod::SystemCommand {
                cmd: "cargo".into(),
                args: vec!["install".into(), "cargo-xwin".into()],
            },
        });
        self.reset_status();
        self.show = true;
    }

    pub fn open_for_clang(&mut self) {
        self.active_dependency = Some(Dependency {
            id: "clang".into(),
            name: "Clang (LLVM)".into(),
            description_key: "dep-wizard-clang-desc".into(),
            method: InstallMethod::SystemCommand {
                cmd: "pkexec".into(),
                args: vec!["apt-get".into(), "install".into(), "-y".into(), "clang".into()],
            },
        });
        self.reset_status();
        self.show = true;
    }

    pub fn open_for_lld(&mut self) {
        self.active_dependency = Some(Dependency {
            id: "lld".into(),
            name: "LLD (LLVM Linker)".into(),
            description_key: "dep-wizard-lld-desc".into(),
            method: InstallMethod::SystemCommand {
                cmd: "pkexec".into(),
                args: vec!["apt-get".into(), "install".into(), "-y".into(), "lld".into()],
            },
        });
        self.reset_status();
        self.show = true;
    }

    pub fn open_for_windows_target(&mut self) {
        self.active_dependency = Some(Dependency {
            id: "windows-target".into(),
            name: "Windows Target".into(),
            description_key: "dep-wizard-windows-target-desc".into(),
            method: InstallMethod::SystemCommand {
                cmd: "rustup".into(),
                args: vec!["target".into(), "add".into(), "x86_64-pc-windows-msvc".into()],
            },
        });
        self.reset_status();
        self.show = true;
    }

    fn reset_status(&mut self) {
        let mut s = self.status.lock().unwrap();
        *s = InstallStatus::Pending;
        self.is_busy.store(false, Ordering::SeqCst);
    }

    pub fn render(&mut self, ctx: &egui::Context, i18n: &I18n) {
        if !self.show {
            return;
        }

        let dep = match self.active_dependency.clone() {
            Some(d) => d,
            None => return,
        };

        let modal = StandardModal::new(
            tr!(i18n, "dep-wizard-title"),
            "dependency_wizard",
        ).with_size(500.0, 320.0);

        let mut local_show = self.show;
        modal.show(ctx, &mut local_show, |ui| {
            modal.ui_body(ui, |ui| {
                ui.vertical_centered(|ui| {
                    ui.add_space(10.0);
                    ui.heading(&dep.name);
                    ui.add_space(10.0);

                    let status = self.status.lock().unwrap().clone();
                    match status {
                        InstallStatus::Pending => {
                            ui.label(tr!(i18n, &dep.description_key, tool = &dep.name));
                            ui.add_space(10.0);
                            match &dep.method {
                                InstallMethod::Download { target, .. } => {
                                    ui.label(tr!(i18n, "dep-wizard-install-question", path = target.display().to_string()));
                                }
                                InstallMethod::SystemCommand { .. } => {
                                    ui.label(tr!(i18n, "dep-wizard-install-cmd-question"));
                                }
                            }
                        }
                        InstallStatus::Downloading(p) => {
                            ui.label(tr!(i18n, "dep-wizard-status-downloading"));
                            ui.add(egui::ProgressBar::new(p).show_percentage());
                        }
                        InstallStatus::RunningCommand => {
                            ui.label(tr!(i18n, "dep-wizard-status-running"));
                            ui.add(egui::Spinner::new());
                        }
                        InstallStatus::Success => {
                            ui.colored_label(egui::Color32::GREEN, tr!(i18n, "dep-wizard-status-success"));
                        }
                        InstallStatus::Error(e) => {
                            ui.colored_label(egui::Color32::RED, tr!(i18n, "dep-wizard-status-error", error = e));
                        }
                    }
                });
            });

            modal.ui_footer(ui, |ui| {
                let status = self.status.lock().unwrap().clone();
                match status {
                    InstallStatus::Pending => {
                        let btn_text = match &dep.method {
                            InstallMethod::Download { .. } => tr!(i18n, "dep-wizard-btn-install"),
                            InstallMethod::SystemCommand { .. } => tr!(i18n, "dep-wizard-btn-run-cmd"),
                        };
                        if ui.button(btn_text).clicked() {
                            self.start_install(ctx.clone());
                        }
                        if ui.button(tr!(i18n, "btn-cancel")).clicked() {
                            self.show = false;
                        }
                    }
                    InstallStatus::Downloading(_) | InstallStatus::RunningCommand => {
                        ui.add(egui::Spinner::new());
                    }
                    InstallStatus::Success | InstallStatus::Error(_) => {
                        if ui.button(tr!(i18n, "btn-close")).clicked() {
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

        std::thread::spawn(move || {
            let rt = tokio::runtime::Runtime::new().unwrap();
            rt.block_on(async {
                match dep.method {
                    InstallMethod::Download { url, target } => {
                        {
                            let mut s = status_arc.lock().unwrap();
                            *s = InstallStatus::Downloading(0.1);
                        }
                        ctx.request_repaint();

                        if let Some(parent) = target.parent() {
                            let _ = std::fs::create_dir_all(parent);
                        }

                        let output = tokio::process::Command::new("curl")
                            .arg("-L").arg("-o").arg(&target).arg(&url).output().await;

                        match output {
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
                                    // Optionally run the installer on Windows
                                    let _ = std::process::Command::new(&target).spawn();
                                }

                                *status_arc.lock().unwrap() = InstallStatus::Success;
                            }
                            _ => *status_arc.lock().unwrap() = InstallStatus::Error("Download failed".into()),
                        }
                    }
                    InstallMethod::SystemCommand { cmd, args } => {
                        {
                            let mut s = status_arc.lock().unwrap();
                            *s = InstallStatus::RunningCommand;
                        }
                        ctx.request_repaint();

                        let output = tokio::process::Command::new(cmd).args(args).output().await;
                        match output {
                            Ok(out) if out.status.success() => *status_arc.lock().unwrap() = InstallStatus::Success,
                            Ok(out) => *status_arc.lock().unwrap() = InstallStatus::Error(String::from_utf8_lossy(&out.stderr).to_string()),
                            Err(e) => *status_arc.lock().unwrap() = InstallStatus::Error(e.to_string()),
                        }
                    }
                }
                is_busy.store(false, Ordering::SeqCst);
                ctx.request_repaint();
            });
        });
    }
}
