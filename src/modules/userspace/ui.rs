use std::env;
use std::fs;
use std::path::Path;
use memchr::memmem;
use crate::helpers::capitalize;

// Names that are WMs, not shells — don't report these as UI
const WM_NAMES: &[&str] = &[
    "hyprland", "sway", "niri", "river", "wayfire", "labwc", "dwl",
    "kwin", "mutter", "openbox", "i3", "bspwm", "dwm", "awesome",
    "xfwm4", "qtile", "xmonad", "herbstluftwm", "weston",
];

fn is_wm_name(s: &str) -> bool {
    let lower = s.to_lowercase();
    WM_NAMES.iter().any(|wm| lower == *wm)
}

// Get the active UI/Shell
pub fn ui() -> String {
    // Check env vars first — single syscall, no /proc scan needed for most users
    if let Ok(desktop) = env::var("XDG_CURRENT_DESKTOP") {
        let result = match desktop.to_lowercase().as_str() {
            "kde" | "plasma" => "Plasma Shell".to_string(),
            "gnome" => "Gnome Shell".to_string(),
            _ => capitalize(&desktop),
        };
        // Skip if it's just a WM name (already shown in WM row)
        if !is_wm_name(&result) {
            return result;
        }
    }

    if let Ok(session) = env::var("DESKTOP_SESSION") {
        let result = capitalize(&session);
        if !is_wm_name(&result) {
            return result;
        }
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