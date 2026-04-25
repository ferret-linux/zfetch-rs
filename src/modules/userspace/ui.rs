use std::env;
use std::fs;
use std::path::Path;
use memchr::memmem;
use crate::helpers::capitalize;

// Get the active UI/Shell
pub fn ui() -> String {
    // Check env vars first — single syscall, no /proc scan needed for most users
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

    // Fallback: scan /proc for process-based shell detection
    let proc_path = Path::new("/proc");
    if let Ok(entries) = fs::read_dir(proc_path) {
        for entry in entries.flatten() {
            let name = entry.file_name();
            let name_bytes = name.as_encoded_bytes();
            if name_bytes.is_empty() || !name_bytes[0].is_ascii_digit() {
                continue;
            }

            let cmdline_path = entry.path().join("cmdline");
            if let Ok(cmdline) = fs::read(&cmdline_path) {
                if memmem::find(&cmdline, b"plasmashell").is_some() {
                    return "Plasma Shell".to_string();
                }
                if memmem::find(&cmdline, b"gnome-shell").is_some() {
                    return "Gnome Shell".to_string();
                }
                if memmem::find(&cmdline, b"waybar").is_some() {
                    return "Waybar".to_string();
                }
            }
        }
    }

    "unknown".to_string()
}