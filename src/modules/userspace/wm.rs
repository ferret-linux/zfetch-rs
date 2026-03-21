use std::env;
use std::fs;
use std::path::Path;
use memchr::memmem;
use crate::helpers::capitalize;

// Get the Window Manager (using /proc instead of subprocess)
pub fn wm() -> String {
    // Check environment variables first - much faster than /proc scan
    if let Ok(desktop) = env::var("XDG_CURRENT_DESKTOP") {
        // Map common desktop values to their WM names
        let wm = match desktop.to_lowercase().as_str() {
            "hyprland" => "Hyprland",
            "sway" => "Sway",
            "kde" | "plasma" => "KWin",
            "gnome" => "Mutter",
            "xfce" => "Xfwm4",
            "i3" => "i3",
            "bspwm" => "bspwm",
            "awesome" => "Awesome",
            "qtile" => "Qtile",
            "niri" => "Niri",
            _ => return desktop,
        };
        return wm.to_string();
    }

    if let Ok(session) = env::var("DESKTOP_SESSION") {
        return capitalize(&session);
    }

    // Fallback: scan /proc for WM processes
    // Known WMs to search for (search term -> display name)
    // Pre-compiled searchers for SIMD-accelerated matching
    let wm_list: &[(&[u8], &str)] = &[
        (b"mutter", "Mutter"),
        (b"kwin", "KWin"),
        (b"sway", "Sway"),
        (b"hyprland", "Hyprland"),
        (b"river", "River"),
        (b"wayfire", "Wayfire"),
        (b"labwc", "LabWC"),
        (b"dwl", "dwl"),
        (b"niri", "Niri"),
        (b"openbox", "Openbox"),
        (b"i3", "i3"),
        (b"bspwm", "bspwm"),
        (b"dwm", "dwm"),
        (b"awesome", "Awesome"),
        (b"xfwm4", "Xfwm4"),
        (b"marco", "Marco"),
        (b"metacity", "Metacity"),
        (b"compiz", "Compiz"),
        (b"enlightenment", "Enlightenment"),
        (b"fluxbox", "Fluxbox"),
        (b"icewm", "IceWM"),
        (b"xmonad", "XMonad"),
        (b"qtile", "Qtile"),
        (b"herbstluftwm", "herbstluftwm"),
        (b"weston", "Weston"),
        (b"cage", "Cage"),
        (b"gamescope", "Gamescope"),
    ];

    // Read /proc directly instead of spawning ps | grep (saves 0.3ish ms)
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
                for (wm_search, wm_display) in wm_list {
                    if memmem::find(&cmdline, wm_search).is_some() {
                        return wm_display.to_string();
                    }
                }
            }
        }
    }

    "unknown".to_string()
}