use std::env;
use std::fs;
use std::path::Path;
use memchr::memmem;
use crate::helpers::{capitalize, get_cached_is_nerd_font, get_dms_theme, get_noctalia_scheme};

// Get the active UI/Shell, i dont know what to call this shit because i already used shell for the terminal shell
pub fn ui() -> String {
    // Scan /proc for custom shells first (noctalia, dms) - these take priority over env vars
    let proc_path = Path::new("/proc");
    if let Ok(entries) = fs::read_dir(proc_path) {
        for entry in entries.flatten() {
            // Fast check: first byte must be a digit (PID directories)
            let name = entry.file_name();
            let name_bytes = name.as_encoded_bytes();
            if name_bytes.is_empty() || !name_bytes[0].is_ascii_digit() {
                continue;
            }

            let cmdline_path = entry.path().join("cmdline");
            // Read as bytes to avoid UTF-8 conversion overhead
            if let Ok(cmdline) = fs::read(&cmdline_path) {
                if memmem::find(&cmdline, b"noctalia-shell").is_some() {
                    let mut name = "Noctalia Shell".to_string();
                    if let Some(scheme) = get_noctalia_scheme() {
                        let icon = if get_cached_is_nerd_font() { "" } else { "Theme:" };
                        name = format!("{} | {} {}", name, icon, capitalize(&scheme));
                    }
                    return name;
                }
                if memmem::find(&cmdline, b"dms").is_some() {
                    let mut name = "DMS".to_string();
                    if let Some(theme) = get_dms_theme() {
                        let formatted_theme = theme
                            .replace("cat-", "Catppuccin (")
                            + if theme.starts_with("cat-") { ")" } else { "" };
                        let icon = if get_cached_is_nerd_font() { "" } else { "Theme:" };
                        name = format!("{} | {} {}", name, icon, capitalize(&formatted_theme));
                    }
                    return name;
                }
                // Fallback: check for common shell processes
                if memmem::find(&cmdline, b"plasmashell").is_some() {
                    return "Plasma Shell".to_string();
                }
                if memmem::find(&cmdline, b"gnome-shell").is_some() {
                    return "Gnome Shell".to_string();
                }
                if memmem::find(&cmdline, b"waybar").is_some() {
                    return "Custom Waybar setup".to_string();
                }
            }
        }
    }

    // Fallback: check env vars for common desktop shells
    if let Ok(desktop) = env::var("XDG_CURRENT_DESKTOP") {
        match desktop.to_lowercase().as_str() {
            "kde" | "plasma" => return "Plasma Shell".to_string(),
            "gnome" => return "Gnome Shell".to_string(),
            "hyprland" => return "Hyprland".to_string(),
            "sway" => return "Sway".to_string(),
            _ => return capitalize(&desktop),
        }
    }

    if let Ok(session) = env::var("DESKTOP_SESSION") {
        return capitalize(&session);
    }

    "unknown".to_string()
}