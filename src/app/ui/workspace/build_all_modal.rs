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
            // skip until final byte in range 0x40–0x7e
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
// BuildAllModal
// ---------------------------------------------------------------------------

pub struct BuildAllModal {
    pub show: bool,
    pub output_lines: Arc<Mutex<Vec<String>>>,
    pub is_running: Arc<AtomicBool>,
    pub exit_code: Arc<Mutex<Option<i32>>>,
}

impl BuildAllModal {
    pub fn new() -> Self {
        Self {
            show: false,
            output_lines: Arc::new(Mutex::new(Vec::new())),
            is_running: Arc::new(AtomicBool::new(false)),
            exit_code: Arc::new(Mutex::new(None)),
        }
    }

    pub fn start(&mut self, root_path: PathBuf, ctx: egui::Context) {
        // Reset state
        self.output_lines.lock().unwrap().clear();
        *self.exit_code.lock().unwrap() = None;
        self.is_running.store(true, Ordering::SeqCst);
        self.show = true;

        let lines = Arc::clone(&self.output_lines);
        let running = Arc::clone(&self.is_running);
        let exit_code = Arc::clone(&self.exit_code);

        std::thread::spawn(move || {
            let result = std::process::Command::new("bash")
                .arg("-c")
                .arg("./scripts/build-all.sh 2>&1")
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
                    *exit_code.lock().unwrap() = Some(-1);
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
                    *exit_code.lock().unwrap() = Some(code);
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

        let modal =
            StandardModal::new(i18n.get("menu-build-all"), "build_all_modal").with_size(900.0, 650.0);

        let mut local_show = self.show;
        // Cell<bool> umožňuje mutaci z closure bez &mut (interior mutability)
        let close_requested = Cell::new(false);
        modal.show(ctx, &mut local_show, |ui| {
            // Status bar
            modal.ui_body(ui, |ui| {
                // Status řádek nahoře
                ui.horizontal(|ui| {
                    if is_running {
                        ui.add(egui::Spinner::new().size(16.0));
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
                                    egui::RichText::new(
                                        i18n.get_args(
                                            "build-all-status-error",
                                            &{
                                                let mut a = fluent_bundle::FluentArgs::new();
                                                a.set("code", code);
                                                a
                                            },
                                        ),
                                    )
                                    .strong()
                                    .color(egui::Color32::from_rgb(220, 100, 100)),
                                );
                            }
                            None => {
                                ui.label(
                                    egui::RichText::new(i18n.get("build-all-status-waiting"))
                                        .weak(),
                                );
                            }
                        }
                    }
                });

                ui.add_space(4.0);
                ui.separator();
                ui.add_space(4.0);

                // Terminálový výpis
                let output = self.output_lines.lock().unwrap().clone();
                let bg = ui.visuals().extreme_bg_color;
                egui::ScrollArea::vertical()
                    .id_salt("build_all_output")
                    .auto_shrink([false, false])
                    .stick_to_bottom(is_running)
                    .show(ui, |ui| {
                        let rect = ui.max_rect();
                        ui.painter().rect_filled(rect, 2.0, bg);
                        ui.set_min_width(rect.width());
                        if output.is_empty() && is_running {
                            ui.label(
                                egui::RichText::new(i18n.get("build-all-waiting-output"))
                                    .monospace()
                                    .size(11.0)
                                    .color(egui::Color32::GRAY),
                            );
                        }
                        for line in &output {
                            let color = line_color(line);
                            ui.label(
                                egui::RichText::new(line)
                                    .monospace()
                                    .size(11.5)
                                    .color(color),
                            );
                        }
                    });
            });

            modal.ui_footer(ui, |ui| {
                if is_running {
                    ui.add_enabled(
                        false,
                        egui::Button::new(i18n.get("build-all-btn-close")),
                    );
                    None::<()>
                } else {
                    if ui.button(i18n.get("build-all-btn-close")).clicked() {
                        close_requested.set(true);
                    }
                    None::<()>
                }
            });
        });

        // Zavření křížkem je povoleno jen pokud neběží
        if !local_show && !is_running {
            self.show = false;
        }
    }
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
    } else if line.contains('┌') || line.contains('│') || line.contains('└')
        || line.contains('╔') || line.contains('║') || line.contains('╚')
        || line.contains('═') || line.contains('━')
    {
        egui::Color32::from_rgb(100, 180, 220)
    } else {
        egui::Color32::from_rgb(200, 200, 200)
    }
}
