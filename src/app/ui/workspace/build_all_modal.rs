use crate::app::ui::widgets::modal::StandardModal;
use eframe::egui;
use std::cell::Cell;
use std::io::BufRead;
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};

// ---------------------------------------------------------------------------
// ANSI escape stripping
// ---------------------------------------------------------------------------

fn strip_ansi(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    let mut chars = s.chars().peekable();
    while let Some(c) = chars.next() {
        if c == '\x1b' {
            if chars.peek() == Some(&'[') {
                chars.next();
                for ch in chars.by_ref() {
                    if ('\x40'..='\x7e').contains(&ch) {
                        break;
                    }
                }
            }
        } else {
            out.push(c);
        }
    }
    out
}

// ---------------------------------------------------------------------------
// PackageSelection
// ---------------------------------------------------------------------------

#[derive(Clone, PartialEq, Default)]
pub enum PackageSelection {
    #[default]
    All,
    Deb,
    Rpm,
    Flatpak,
    Snap,
    AppImage,
    Exe,
    FreeBsd,
}

impl PackageSelection {
    fn label(&self) -> &'static str {
        match self {
            Self::All => "Vše — všechny balíčky",
            Self::Deb => ".deb — Debian / Ubuntu",
            Self::Rpm => ".rpm — Fedora",
            Self::Flatpak => ".flatpak",
            Self::Snap => ".snap",
            Self::AppImage => ".AppImage",
            Self::Exe => ".exe — Windows",
            Self::FreeBsd => ".pkg — FreeBSD",
        }
    }

    fn script_arg(&self) -> &'static str {
        match self {
            Self::All => "",
            Self::Deb => "--only=deb",
            Self::Rpm => "--only=rpm",
            Self::Flatpak => "--only=flatpak",
            Self::Snap => "--only=snap",
            Self::AppImage => "--only=appimage",
            Self::Exe => "--only=exe",
            Self::FreeBsd => "--only=freebsd",
        }
    }
}

// ---------------------------------------------------------------------------
// BuildAllModal
// ---------------------------------------------------------------------------

pub struct BuildAllModal {
    pub show: bool,
    pub output_lines: Arc<Mutex<Vec<String>>>,
    pub is_running: Arc<AtomicBool>,
    pub exit_code: Arc<Mutex<Option<i32>>>,
    pub selected: PackageSelection,
    root_path: Option<PathBuf>,
}

impl BuildAllModal {
    pub fn new() -> Self {
        Self {
            show: false,
            output_lines: Arc::new(Mutex::new(Vec::new())),
            is_running: Arc::new(AtomicBool::new(false)),
            exit_code: Arc::new(Mutex::new(None)),
            selected: PackageSelection::All,
            root_path: None,
        }
    }

    /// Otevře modal — nespustí build automaticky.
    pub fn open(&mut self, root_path: PathBuf) {
        self.root_path = Some(root_path);
        self.output_lines.lock().unwrap().clear();
        *self.exit_code.lock().unwrap() = None;
        self.show = true;
    }

    fn run(&mut self, ctx: egui::Context) {
        let Some(root_path) = self.root_path.clone() else {
            return;
        };
        self.output_lines.lock().unwrap().clear();
        *self.exit_code.lock().unwrap() = None;
        self.is_running.store(true, Ordering::SeqCst);

        let arg = self.selected.script_arg().to_string();
        let lines = Arc::clone(&self.output_lines);
        let running = Arc::clone(&self.is_running);
        let exit_code_arc = Arc::clone(&self.exit_code);

        std::thread::spawn(move || {
            let script = if arg.is_empty() {
                "./scripts/build-all.sh 2>&1".to_string()
            } else {
                format!("./scripts/build-all.sh {} 2>&1", arg)
            };

            let result = std::process::Command::new("bash")
                .arg("-c")
                .arg(&script)
                .current_dir(&root_path)
                .stdout(std::process::Stdio::piped())
                .stderr(std::process::Stdio::null())
                .spawn();

            match result {
                Err(e) => {
                    lines
                        .lock()
                        .unwrap()
                        .push(format!("ERROR: nelze spustit skript: {e}"));
                    *exit_code_arc.lock().unwrap() = Some(-1);
                    running.store(false, Ordering::SeqCst);
                    ctx.request_repaint();
                }
                Ok(mut child) => {
                    if let Some(stdout) = child.stdout.take() {
                        let reader = std::io::BufReader::new(stdout);
                        for line in reader.lines() {
                            match line {
                                Ok(l) => {
                                    lines.lock().unwrap().push(strip_ansi(&l));
                                    ctx.request_repaint();
                                }
                                Err(_) => break,
                            }
                        }
                    }
                    let code = child.wait().map(|s| s.code().unwrap_or(-1)).unwrap_or(-1);
                    *exit_code_arc.lock().unwrap() = Some(code);
                    running.store(false, Ordering::SeqCst);
                    ctx.request_repaint();
                }
            }
        });
    }

    pub fn render(&mut self, ctx: &egui::Context, i18n: &crate::i18n::I18n) {
        if !self.show {
            return;
        }

        let is_running = self.is_running.load(Ordering::SeqCst);
        let exit_code = *self.exit_code.lock().unwrap();

        let modal = StandardModal::new(i18n.get("menu-build-all"), "build_all_modal")
            .with_size(900.0, 650.0);

        let mut local_show = self.show;
        let close_requested = Cell::new(false);
        let run_requested = Cell::new(false);
        let new_selection: Cell<Option<PackageSelection>> = Cell::new(None);

        modal.show(ctx, &mut local_show, |ui| {
            modal.ui_body(ui, |ui| {
                // ── Horní lišta: combobox + tlačítko + status ────────────
                ui.horizontal(|ui| {
                    // ComboBox výběru balíčku (zakázán při běhu)
                    ui.add_enabled_ui(!is_running, |ui| {
                        egui::ComboBox::from_id_salt("build_pkg_select")
                            .selected_text(self.selected.label())
                            .width(240.0)
                            .show_ui(ui, |ui| {
                                let variants = [
                                    PackageSelection::All,
                                    PackageSelection::Deb,
                                    PackageSelection::Rpm,
                                    PackageSelection::Flatpak,
                                    PackageSelection::Snap,
                                    PackageSelection::AppImage,
                                    PackageSelection::Exe,
                                    PackageSelection::FreeBsd,
                                ];
                                for pkg in variants {
                                    let selected = pkg == self.selected;
                                    if ui.selectable_label(selected, pkg.label()).clicked() {
                                        new_selection.set(Some(pkg));
                                    }
                                }
                            });
                    });

                    // Spustit / Znovu spustit
                    if !is_running {
                        let btn_label = if exit_code.is_some() {
                            i18n.get("build-all-btn-rerun")
                        } else {
                            i18n.get("build-all-btn-run")
                        };
                        if ui.button(btn_label).clicked() {
                            run_requested.set(true);
                        }
                    } else {
                        ui.add(egui::Spinner::new().size(16.0));
                    }

                    // Status vpravo
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if is_running {
                            ui.label(
                                egui::RichText::new(i18n.get("build-all-status-running"))
                                    .strong()
                                    .color(egui::Color32::from_rgb(180, 200, 255)),
                            );
                        } else {
                            match exit_code {
                                Some(0) => {
                                    ui.label(
                                        egui::RichText::new(i18n.get("build-all-status-ok"))
                                            .strong()
                                            .color(egui::Color32::from_rgb(100, 220, 100)),
                                    );
                                }
                                Some(code) => {
                                    ui.label(
                                        egui::RichText::new(i18n.get_args(
                                            "build-all-status-error",
                                            &{
                                                let mut a = fluent_bundle::FluentArgs::new();
                                                a.set("code", code);
                                                a
                                            },
                                        ))
                                        .strong()
                                        .color(egui::Color32::from_rgb(220, 100, 100)),
                                    );
                                }
                                None => {
                                    ui.label(
                                        egui::RichText::new(i18n.get("build-all-not-started"))
                                            .weak(),
                                    );
                                }
                            }
                        }
                    });
                });

                ui.add_space(4.0);
                ui.separator();
                ui.add_space(4.0);

                // ── Aktuální krok ────────────────────────────────────────
                let output = self.output_lines.lock().unwrap().clone();

                if is_running {
                    let step = detect_current_step(&output)
                        .unwrap_or_else(|| self.selected.label().to_string());
                    ui.horizontal(|ui| {
                        ui.add(egui::Spinner::new().size(13.0));
                        ui.label(
                            egui::RichText::new(&step)
                                .strong()
                                .size(13.5)
                                .color(egui::Color32::from_rgb(160, 205, 255)),
                        );
                    });
                    ui.add_space(4.0);
                    ui.separator();
                    ui.add_space(4.0);
                }
                let bg = ui.visuals().extreme_bg_color;
                egui::ScrollArea::vertical()
                    .id_salt("build_all_output")
                    .auto_shrink([false, false])
                    .stick_to_bottom(is_running)
                    .show(ui, |ui| {
                        let rect = ui.max_rect();
                        ui.painter().rect_filled(rect, 2.0, bg);
                        ui.set_min_width(rect.width());
                        if output.is_empty() {
                            let msg = if is_running {
                                i18n.get("build-all-waiting-output")
                            } else {
                                i18n.get("build-all-hint-start")
                            };
                            ui.label(
                                egui::RichText::new(msg)
                                    .monospace()
                                    .size(11.0)
                                    .color(egui::Color32::GRAY),
                            );
                        } else {
                            for line in &output {
                                let color = line_color(line);
                                ui.label(
                                    egui::RichText::new(line)
                                        .monospace()
                                        .size(11.5)
                                        .color(color),
                                );
                            }
                            ui.add_space(64.0);
                        }
                    });
            });

            modal.ui_footer(ui, |ui| {
                if is_running {
                    ui.add_enabled(false, egui::Button::new(i18n.get("build-all-btn-close")));
                } else if ui.button(i18n.get("build-all-btn-close")).clicked() {
                    close_requested.set(true);
                }
                None::<()>
            });
        });

        // Aplikuj změnu výběru (z ComboBox)
        if let Some(sel) = new_selection.take() {
            self.selected = sel;
        }
        // Spusť build
        if run_requested.get() {
            self.run(ctx.clone());
        }
        // Zavření
        if !local_show && !is_running {
            self.show = false;
        }
        if close_requested.get() && !is_running {
            self.show = false;
        }
    }
}

/// Vrátí název posledního detekovaného kroku ze záhlaví boxu (│  X/7  ... │).
fn detect_current_step(lines: &[String]) -> Option<String> {
    for line in lines.iter().rev() {
        let t = line.trim();
        if t.starts_with('│') && t.ends_with('│') && t.len() > 4 {
            let inner = t[1..t.len() - 1].trim();
            if inner.contains('/') && !inner.chars().all(|c| c == '─' || c == '═' || c == ' ') {
                return Some(inner.to_string());
            }
        }
    }
    None
}

/// Barva řádku podle obsahu (bez ANSI kódů).
fn line_color(line: &str) -> egui::Color32 {
    if line.contains('✔') || line.contains("Úspěšně") || line.contains("sestaveno") {
        egui::Color32::from_rgb(100, 220, 120)
    } else if line.contains('✘') || line.contains("Selhalo") || line.contains("ERROR") {
        egui::Color32::from_rgb(230, 100, 100)
    } else if line.contains('⊘') || line.contains("Přeskočeno") || line.contains("chybí") {
        egui::Color32::from_rgb(220, 180, 80)
    } else if line.contains('▶') || line.contains("ruční") {
        egui::Color32::from_rgb(230, 180, 80)
    } else if line.contains('┌')
        || line.contains('│')
        || line.contains('└')
        || line.contains('╔')
        || line.contains('║')
        || line.contains('╚')
        || line.contains('═')
        || line.contains('━')
    {
        egui::Color32::from_rgb(100, 180, 220)
    } else {
        egui::Color32::from_rgb(200, 200, 200)
    }
}
