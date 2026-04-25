use std::env;
use crate::helpers::capitalize;

// Get the active terminal
pub fn terminal() -> String {
    // Check for specific terminal environment variables first
    if env::var("GHOSTTY_RESOURCES_DIR").is_ok() {
        return "Ghostty".to_string();
    }
    if env::var("KITTY_PID").is_ok() {
        return "Kitty".to_string();
    }
    if env::var("WEZTERM_PANE").is_ok() {
        return "WezTerm".to_string();
    }
    if env::var("PTYXIS_VERSION").is_ok() {
        return "Ptyxis".to_string();
    }
    if env::var("ALACRITTY_SOCKET").is_ok() || env::var("ALACRITTY_LOG").is_ok() {
        return "Alacritty".to_string();
    }
    if env::var("KONSOLE_VERSION").is_ok() {
        return "Konsole".to_string();
    }
    if env::var("GNOME_TERMINAL_SCREEN").is_ok() {
        return "Gnome Terminal".to_string();
    }
    if env::var("FOOT_SERVER_SOCKET").is_ok() {
        return "Foot".to_string();
    }
    if let Ok(term_program) = env::var("TERM_PROGRAM") {
        if term_program == "kgx" {
            return "GNOME Console".to_string();
        }
    }
    if let Ok(term) = env::var("TERM") {
        if term == "foot" || term == "foot-extra" {
            return "Foot".to_string();
        }
    }

    // Fallback to TERM_PROGRAM or TERM
    let term = env::var("TERM_PROGRAM")
        .or_else(|_| env::var("TERM"))
        .unwrap_or_else(|_| "unknown".to_string());

    // Clean up common suffixes like -256color
    let name = term.split("-256color").next().unwrap_or(&term);
    let name = name.split("-color").next().unwrap_or(name);

    capitalize(name)
}