use eframe::egui;
use std::path::PathBuf;
use std::sync::Arc;

/// Configures fonts to support a wider range of Unicode characters (emojis, icons, symbols).
pub fn setup_custom_fonts(ctx: &egui::Context) {
    let mut fonts = egui::FontDefinitions::default();
    let home = dirs::home_dir().unwrap_or_else(|| PathBuf::from("/"));

    let mut search_dirs = vec![];

    #[cfg(target_os = "linux")]
    {
        search_dirs.extend([
            home.join(".local/share/fonts"),
            home.join(".fonts"),
            PathBuf::from("/usr/share/fonts"),
            PathBuf::from("/usr/local/share/fonts"),
        ]);
    }

    #[cfg(target_os = "macos")]
    {
        search_dirs.extend([
            home.join("Library/Fonts"),
            PathBuf::from("/Library/Fonts"),
            PathBuf::from("/System/Library/Fonts"),
        ]);
    }

    #[cfg(target_os = "windows")]
    {
        search_dirs.push(
            PathBuf::from(std::env::var("WINDIR").unwrap_or_else(|_| r"C:\Windows".to_string()))
                .join("Fonts"),
        );
        if let Ok(local) = std::env::var("LOCALAPPDATA") {
            search_dirs.push(PathBuf::from(local).join(r"Microsoft\Windows\Fonts"));
        }
    }

    // Search patterns (normalized: no spaces, lowercase)
    let preferred_mono = [
        "jetbrainsmono",
        "notomono",
        "dejavusansmono",
        "ubuntumono",
        "liberationmono",
        "consolas",     // Windows
        "cascadiamono", // Windows
        "menlo",        // macOS
        "sfmono",       // macOS
        "monaco",       // macOS
    ];
    let preferred_prop = [
        "ubunturegular",
        "notosansregular",
        "dejavusans",
        "notosans",
        "ubuntu",
        "roboto",
        "liberationsans",
        "segoeui",   // Windows
        "arial",     // Windows / macOS
        "sfpro",     // macOS
        "helvetica", // macOS
    ];
    let preferred_symbols = [
        "symbola",
        "nerdfont",
        "notocoloremoji",
        "notoemoji",
        "symbol",
    ];

    let mut loaded_primary_mono = None;
    let mut loaded_primary_prop = None;
    let mut loaded_symbols = Vec::new();

    eprintln!("fonts: scanning for compatible fonts...");

    for dir in search_dirs {
        if !dir.exists() {
            continue;
        }

        for entry in walkdir::WalkDir::new(dir)
            .follow_links(true)
            .into_iter()
            .flatten()
        {
            let path = entry.path();
            let Some(ext) = path.extension() else {
                continue;
            };
            if ext != "ttf" && ext != "otf" {
                continue;
            }

            let filename = path.file_name().unwrap().to_string_lossy();
            let filename_low = filename.to_lowercase();
            let filename_norm = filename_low.replace(" ", "").replace("-", "");

            // 1. IS IT A SYMBOL FONT? (Use only as fallback)
            let mut is_symbol_font = filename_low.contains("symbol")
                || filename_low.contains("nerd")
                || filename_low.contains("icon")
                || filename_low.contains("math")
                || filename_low.contains("emoji");

            if !is_symbol_font {
                for &s in &preferred_symbols {
                    if filename_norm.contains(s) {
                        is_symbol_font = true;
                        break;
                    }
                }
            }

            if is_symbol_font {
                // Skip bitmap/color emoji fonts (CBDT/CBLC) — egui can only render outline fonts
                if filename_low.contains("coloremoji") || filename_low.contains("color-emoji") {
                    eprintln!("fonts: skipping bitmap color font: {}", filename);
                    continue;
                }
                // Skip symbol font variants (bold, italic, etc.) — they don't add new glyphs
                if filename_low.contains("italic")
                    || filename_low.contains("oblique")
                    || filename_low.contains("bold")
                    || filename_low.contains("thin")
                    || filename_low.contains("condensed")
                    || filename_low.contains("extrabold")
                    || filename_low.contains("semibold")
                    || filename_low.contains("medium")
                    || filename_low.contains("extralight")
                    || filename_low.contains("black")
                {
                    continue;
                }
                if let Ok(data) = std::fs::read(path) {
                    let name = format!("fallback_sym_{}", filename);
                    if !fonts.font_data.contains_key(&name) {
                        fonts
                            .font_data
                            .insert(name.clone(), Arc::new(egui::FontData::from_owned(data)));
                        loaded_symbols.push(name);
                        eprintln!("fonts: loaded symbol source: {}", filename);
                    }
                }
                continue;
            }

            // 2. IS IT A VARIANT WE WANT TO SKIP FOR PRIMARY TEXT?
            let is_variant = filename_low.contains("italic")
                || filename_low.contains("oblique")
                || filename_low.contains("bold")
                || filename_low.contains("light")
                || filename_low.contains("thin")
                || filename_low.contains("condensed");

            if !is_variant {
                // Check for primary MONOSPACE
                if loaded_primary_mono.is_none() {
                    for &p in &preferred_mono {
                        if filename_norm.contains(p)
                            && let Ok(data) = std::fs::read(path)
                        {
                            let name = format!("primary_mono_{}", p);
                            fonts
                                .font_data
                                .insert(name.clone(), Arc::new(egui::FontData::from_owned(data)));
                            loaded_primary_mono = Some(name);
                            eprintln!("fonts: selected primary mono: {}", filename);
                            break;
                        }
                    }
                }
                // Check for primary PROPORTIONAL
                if loaded_primary_prop.is_none() {
                    for &p in &preferred_prop {
                        if filename_norm.contains(p)
                            && let Ok(data) = std::fs::read(path)
                        {
                            let name = format!("primary_prop_{}", p);
                            fonts
                                .font_data
                                .insert(name.clone(), Arc::new(egui::FontData::from_owned(data)));
                            loaded_primary_prop = Some(name);
                            eprintln!("fonts: selected primary prop: {}", filename);
                            break;
                        }
                    }
                }
            }
        }
    }

    // Apply to families: [Primary, Symbol1, Symbol2..., egui_defaults...]
    if let Some(name) = loaded_primary_mono {
        let mono = fonts
            .families
            .get_mut(&egui::FontFamily::Monospace)
            .unwrap();
        mono.insert(0, name);
        for (idx, sym) in loaded_symbols.iter().enumerate() {
            mono.insert(idx + 1, sym.clone());
        }
    } else {
        eprintln!("fonts: warning - no primary monospace font found!");
    }

    if let Some(name) = loaded_primary_prop {
        let prop = fonts
            .families
            .get_mut(&egui::FontFamily::Proportional)
            .unwrap();
        prop.insert(0, name);
        for (idx, sym) in loaded_symbols.iter().enumerate() {
            prop.insert(idx + 1, sym.clone());
        }
    } else {
        eprintln!("fonts: warning - no primary proportional font found!");
    }

    ctx.set_fonts(fonts);
}
