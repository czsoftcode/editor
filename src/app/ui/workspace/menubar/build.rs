use super::super::state::WorkspaceState;
use super::MenuActions;
use eframe::egui;

pub fn render(
    ui: &mut egui::Ui,
    actions: &mut MenuActions,
    ws: &WorkspaceState,
    i18n: &crate::i18n::I18n,
) {
    let can_build = !ws.build_in_sandbox && ws.sandbox_staged_files.is_empty();
    let build_resp = ui
        .add_enabled_ui(can_build, |ui| {
            ui.menu_button(i18n.get("menu-build"), |ui| {
                let get_icon = |id: &str| {
                    if *ws.win_tool_available.get(id).unwrap_or(&false) {
                        "✅"
                    } else {
                        "❌"
                    }
                };

                ui.menu_button(i18n.get("menu-build-debian"), |ui| {
                    if ui
                        .button(format!(
                            "{} {}",
                            get_icon("deb"),
                            i18n.get("menu-build-deb")
                        ))
                        .clicked()
                    {
                        actions.build_deb = true;
                        ui.close_menu();
                    }
                    ui.separator();
                    if ui
                        .button(format!(
                            "{} {}",
                            get_icon("deb"),
                            i18n.get("command-name-install-deb-tools")
                        ))
                        .clicked()
                    {
                        actions.install_deb_tools = true;
                        ui.close_menu();
                    }
                });

                ui.menu_button(i18n.get("menu-build-freebsd"), |ui| {
                    if ui
                        .button(format!(
                            "{} {}",
                            get_icon("freebsd-target"),
                            i18n.get("menu-build-freebsd-pkg")
                        ))
                        .clicked()
                    {
                        actions.build_freebsd = true;
                        ui.close_menu();
                    }
                    ui.separator();
                    if ui
                        .button(format!(
                            "{} {}",
                            get_icon("freebsd-target"),
                            i18n.get("command-name-install-freebsd-target")
                        ))
                        .clicked()
                    {
                        actions.install_freebsd_target = true;
                        ui.close_menu();
                    }
                    if ui
                        .button(format!(
                            "{} {}",
                            get_icon("cross"),
                            i18n.get("command-name-install-cross")
                        ))
                        .clicked()
                    {
                        actions.install_cross = true;
                        ui.close_menu();
                    }
                    if ui
                        .button(format!(
                            "{} {}",
                            get_icon("fpm"),
                            i18n.get("command-name-install-fpm")
                        ))
                        .clicked()
                    {
                        actions.install_fpm = true;
                        ui.close_menu();
                    }
                    if ui
                        .button(format!(
                            "{} {}",
                            get_icon("podman"),
                            i18n.get("command-name-install-podman")
                        ))
                        .clicked()
                    {
                        actions.install_podman = true;
                        ui.close_menu();
                    }
                });

                ui.menu_button(i18n.get("menu-build-flatpak-sub"), |ui| {
                    if ui
                        .button(format!(
                            "{} {}",
                            get_icon("flatpak"),
                            i18n.get("menu-build-flatpak")
                        ))
                        .clicked()
                    {
                        actions.build_flatpak = true;
                        ui.close_menu();
                    }
                    ui.separator();
                    if ui
                        .button(format!(
                            "{} {}",
                            get_icon("flatpak"),
                            i18n.get("command-name-install-flatpak")
                        ))
                        .clicked()
                    {
                        actions.install_flatpak = true;
                        ui.close_menu();
                    }
                });

                ui.menu_button(i18n.get("menu-build-snap-sub"), |ui| {
                    if ui
                        .button(format!(
                            "{} {}",
                            get_icon("snap"),
                            i18n.get("menu-build-snap")
                        ))
                        .clicked()
                    {
                        actions.build_snap = true;
                        ui.close_menu();
                    }
                    ui.separator();
                    if ui
                        .button(format!(
                            "{} {}",
                            get_icon("snap"),
                            i18n.get("command-name-install-snap")
                        ))
                        .clicked()
                    {
                        actions.install_snap = true;
                        ui.close_menu();
                    }
                    if ui
                        .button(format!(
                            "{} {}",
                            get_icon("snap"),
                            i18n.get("command-name-configure-lxd")
                        ))
                        .clicked()
                    {
                        actions.configure_lxd = true;
                        ui.close_menu();
                    }
                });


                ui.menu_button(i18n.get("menu-build-fedora"), |ui| {
                    if ui
                        .button(format!(
                            "{} {}",
                            get_icon("generate-rpm"),
                            i18n.get("menu-build-rpm")
                        ))
                        .clicked()
                    {
                        actions.build_rpm = true;
                        ui.close_menu();
                    }
                    ui.separator();
                    if ui
                        .button(format!(
                            "{} {}",
                            get_icon("generate-rpm"),
                            i18n.get("command-name-install-generate-rpm")
                        ))
                        .clicked()
                    {
                        actions.install_generate_rpm = true;
                        ui.close_menu();
                    }
                    if ui
                        .button(format!(
                            "{} {}",
                            get_icon("rpm"),
                            i18n.get("command-name-install-rpm")
                        ))
                        .clicked()
                    {
                        actions.install_rpm = true;
                        ui.close_menu();
                    }
                });

                ui.menu_button(i18n.get("menu-build-appimage-sub"), |ui| {
                    if ui
                        .button(format!(
                            "{} {}",
                            get_icon("appimage"),
                            i18n.get("menu-build-appimage")
                        ))
                        .clicked()
                    {
                        actions.build_appimage = true;
                        ui.close_menu();
                    }
                    ui.separator();
                    if ui
                        .button(format!(
                            "{} {}",
                            get_icon("appimage"),
                            i18n.get("command-name-install-appimage")
                        ))
                        .clicked()
                    {
                        actions.install_appimage = true;
                        ui.close_menu();
                    }
                    if ui
                        .button(format!(
                            "{} {}",
                            get_icon("appimagetool"),
                            i18n.get("command-name-install-appimagetool")
                        ))
                        .clicked()
                    {
                        actions.install_appimagetool = true;
                        ui.close_menu();
                    }
                });

                ui.menu_button(i18n.get("menu-build-windows"), |ui| {
                    if ui
                        .button(format!(
                            "{} {}",
                            get_icon("xwin"),
                            i18n.get("menu-build-exe")
                        ))
                        .clicked()
                    {
                        actions.build_exe = true;
                        ui.close_menu();
                    }
                    ui.separator();
                    if ui
                        .button(format!(
                            "{} {}",
                            get_icon("windows-target"),
                            i18n.get("command-name-install-windows-target")
                        ))
                        .clicked()
                    {
                        actions.install_windows_target = true;
                        ui.close_menu();
                    }
                    if ui
                        .button(format!(
                            "{} {}",
                            get_icon("xwin"),
                            i18n.get("command-name-install-xwin")
                        ))
                        .clicked()
                    {
                        actions.install_xwin = true;
                        ui.close_menu();
                    }
                    if ui
                        .button(format!(
                            "{} {}",
                            get_icon("clang"),
                            i18n.get("command-name-install-clang")
                        ))
                        .clicked()
                    {
                        actions.install_clang = true;
                        ui.close_menu();
                    }
                    if ui
                        .button(format!(
                            "{} {}",
                            get_icon("lld"),
                            i18n.get("command-name-install-lld")
                        ))
                        .clicked()
                    {
                        actions.install_lld = true;
                        ui.close_menu();
                    }
                    if ui
                        .button(format!(
                            "{} {}",
                            get_icon("nsis"),
                            i18n.get("command-name-install-nsis")
                        ))
                        .clicked()
                    {
                        actions.install_nsis = true;
                        ui.close_menu();
                    }
                });
                ui.separator();
                if ui
                    .button(i18n.get("menu-build-all"))
                    .clicked()
                {
                    actions.build_all = true;
                    ui.close_menu();
                }
            });
        })
        .response;

    if !can_build {
        build_resp.on_hover_text(i18n.get("hover-build-menu-disabled"));
    }
}
