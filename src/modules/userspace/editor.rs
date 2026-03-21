use std::env;
use crate::helpers::{capitalize, get_cached_is_nerd_font};

// Get the user's preferred editor from environment variables.
// Returns empty string if unset or set to nano (dont @ me)
pub fn editor() -> String {
    let visual = env::var("VISUAL").ok();
    let editor = env::var("EDITOR").ok();

    // Helper to extract and format editor name
    let format_editor = |path: &str| -> Option<String> {
        let name = path.split('/').last().unwrap_or(path);
        if name == "nano" {
            None
        } else {
            Some(capitalize(name))
        }
    };

    match (visual.as_deref().and_then(format_editor), editor.as_deref().and_then(format_editor)) {
        (Some(v), Some(e)) if v != e => {
            let (icon1, icon2) = if get_cached_is_nerd_font() { ("󰍹", "") } else { ("GUI:", "TUI:") };
            format!("{} {} | {} {}", icon1, v, icon2, e)
        }
        (Some(v), _) => v,
        (None, Some(e)) => e,
        (None, None) => String::new()
    }
}