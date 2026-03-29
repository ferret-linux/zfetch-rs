// Font finder module for zfetch.
// Parses terminal configs to find the in-use font.

use std::fs;
use std::env;
use memchr::memmem;
use super::userspace::terminal;

// Get the terminal font by parsing config files
pub fn find_font() -> String {
    let term = terminal();
    let home = env::var("HOME").unwrap_or_default();
    let config_home = env::var("XDG_CONFIG_HOME")
        .unwrap_or_else(|_| format!("{}/.config", home));
    let term_lower = term.to_lowercase();
    let result = if term_lower.contains("kitty") {
        font_from_kitty(&config_home)
    } else if term_lower.contains("alacritty") {
        font_from_alacritty(&config_home)
    } else if term_lower.starts_with("foot") {
        font_from_foot(&config_home)
    } else if term_lower.contains("ghostty") {
        font_from_ghostty(&config_home)
    } else if term_lower.contains("gnome terminal") || term_lower.contains("gnome-terminal") {
        font_from_gnome_terminal()
    } else if term_lower.contains("gnome console") || term_lower.contains("kgx") {
        font_from_gnome_console()
    } else if term_lower.contains("ptyxis") {
        font_from_ptyxis()
    } else if term_lower.contains("konsole") {
        font_from_konsole(&home, &config_home)
    } else if term_lower.contains("wezterm") {
        font_from_wezterm(&config_home)
    } else {
        None
    };

    result
        .or_else(font_from_gsettings_monospace)
        .unwrap_or_else(|| "unknown".to_string())
}

// Parse Kitty config (~/.config/kitty/kitty.conf)
fn font_from_kitty(config_home: &str) -> Option<String> {
    if config_home.is_empty() { return None; }
    let path = format!("{}/kitty/kitty.conf", config_home);
    let content = fs::read_to_string(path).ok()?;

    for line in content.lines() {
        let line = line.trim();
        if line.starts_with('#') { continue; }
        if let Some(val) = line.strip_prefix("font_family") {
            let font = val.trim();
            if !font.is_empty() {
                return Some(clean_font_name(font));
            }
        }
    }
    None
}

// Parse Alacritty config (~/.config/alacritty/alacritty.toml)
fn font_from_alacritty(config_home: &str) -> Option<String> {
    if config_home.is_empty() { return None; }
    let path = format!("{}/alacritty/alacritty.toml", config_home);
    let content = fs::read_to_string(&path).ok()?;

    // Find [font.normal] section then grab the first family= within it
    let mut in_font_section = false;
    for line in content.lines() {
        let line = line.trim();
        if line.starts_with('[') {
            in_font_section = line == "[font]" || line == "[font.normal]";
            continue;
        }
        if !in_font_section { continue; }
        if line.starts_with('#') { continue; }

        // Handle inline table: normal = { family = "Hack Nerd Font", style = "Regular" }
        if line.starts_with("normal") {
            if let Some(fam_pos) = line.find("family") {
                let after = &line[fam_pos + 6..]; // skip "family"
                if let Some(eq_pos) = after.find('=') {
                    let val = after[eq_pos + 1..].trim().trim_matches(['"', '\'']);
                    let val = val.split([',', '}']).next().unwrap_or("").trim().trim_matches(['"', '\'']);
                    if !val.is_empty() {
                        return Some(clean_font_name(val));
                    }
                }
            }
        }

        // Handle standalone: family = "Font Name"
        if let Some(val) = line.strip_prefix("family =").or_else(|| line.strip_prefix("family=")) {
            let font = val.trim().trim_matches(['"', '\'']);
            if !font.is_empty() {
                return Some(clean_font_name(font));
            }
        }
    }
    None
}

// Parse Foot config (~/.config/foot/foot.ini)
fn font_from_foot(config_home: &str) -> Option<String> {
    if config_home.is_empty() { return None; }
    let path = format!("{}/foot/foot.ini", config_home);
    let content = fs::read_to_string(&path).ok()?;

    for line in content.lines() {
        let line = line.trim();
        if line.starts_with('#') {
            continue;
        }
        if let Some(val) = line.strip_prefix("font=") {
            // Strip size specifier e.g. "JetBrains Mono:size=12"
            let font = val.split(':').next().unwrap_or(val).trim();
            if !font.is_empty() {
                return Some(clean_font_name(font));
            }
        }
    }
    None
}

// Parse Ghostty config (~/.config/ghostty/config.ghostty)
fn font_from_ghostty(config_home: &str) -> Option<String> {
    if config_home.is_empty() { return None; }
    let path = format!("{}/ghostty/config.ghostty", config_home);
    let content = fs::read_to_string(path).ok()?;

    for line in content.lines() {
        let line = line.trim();
        if line.starts_with('#') { continue; }
        if let Some(val) = line.strip_prefix("font-family =")
            .or_else(|| line.strip_prefix("font-family=")) {
            let font = val.trim().trim_matches(['"', '\'']);
            if !font.is_empty() {
                return Some(clean_font_name(font));
            }
        }
    }
    None
}

// Parse Konsole profile (~/.local/share/konsole/*.profile)
fn font_from_konsole(home: &str, config_home: &str) -> Option<String> {
    if home.is_empty() { return None; }
    let read_font = |path: &std::path::Path| -> Option<String> {
        let content = fs::read(path).ok()?;
        // Handle Font= at start of file or after newline
        let pos = if content.starts_with(b"Font=") {
            Some(0)
        } else {
            memmem::find(&content, b"\nFont=").map(|p| p + 1)
        }?;
        let after = &content[pos + 5..];
        let end = memchr::memchr(b'\n', after).unwrap_or(after.len());
        let comma = memchr::memchr(b',', after).unwrap_or(end);
        let font = std::str::from_utf8(&after[..end.min(comma)]).ok()?;
        Some(clean_font_name(font.trim()))
    };
    let from_rc = fs::read_to_string(format!("{}/konsolerc", config_home)).ok()
        .and_then(|rc| {
            let name = rc.lines()
                .find(|l| l.starts_with("DefaultProfile="))
                .and_then(|l| l.strip_prefix("DefaultProfile="))
                .map(|s| s.trim().to_string())?;
            read_font(std::path::Path::new(&format!("{}/.local/share/konsole/{}", home, name)))
                .or_else(|| read_font(std::path::Path::new(&format!("/usr/share/konsole/{}", name))))
        });

    from_rc.or_else(|| {
        fs::read_dir("/usr/share/konsole").ok()?.flatten()
            .filter(|e| e.path().extension().is_some_and(|x| x == "profile"))
            .find_map(|e| read_font(&e.path()))
    })
}

// Parse GNOME Terminal via dconf
fn font_from_gnome_terminal() -> Option<String> {
    // Get the default profile UUID first
    let default_profile = run_command("dconf", &["read", "/org/gnome/terminal/legacy/profiles:/default"])
        .map(|s| s.trim_matches('\'').to_string())
        .filter(|s| !s.is_empty());

    // If we have a default profile UUID, read just that profile
    let profile_path = if let Some(uuid) = default_profile {
        format!("/org/gnome/terminal/legacy/profiles:/:{}/", uuid)
    } else {
        // Fallback: no default set, dump all and use first custom-font profile
        return font_from_gnome_terminal_scan();
    };

    let use_system = run_command("dconf", &["read", &format!("{}use-system-font", profile_path)]);

    if matches!(use_system.as_deref(), Some("true") | None | Some("")) {
        return font_from_gsettings_monospace();
    }

    let font_raw = run_command("dconf", &["read", &format!("{}font", profile_path)])
        .map(|s| s.trim_matches('\'').to_string())
        .filter(|s| !s.is_empty())?;

    let font = font_raw.rsplit_once(' ').map(|(name, _)| name).unwrap_or(&font_raw);
    if font.is_empty() {
        return font_from_gsettings_monospace();
    }
    Some(clean_font_name(font))
}

// Fallback: scan all profiles and return first one with a custom font
fn font_from_gnome_terminal_scan() -> Option<String> {
    let content = match run_command("dconf", &["dump", "/org/gnome/terminal/legacy/profiles:/"]) {
        Some(s) => s,
        None => return font_from_gsettings_monospace(),
    };
    let content = content.as_str();
    let mut use_system = true;
    let mut found_font: Option<String> = None;
    for line in content.lines() {
        let line = line.trim();
        if line.starts_with('[') {
            if !use_system {
                if let Some(f) = found_font {
                    return Some(clean_font_name(&f));
                }
            }
            use_system = true;
            found_font = None;
        } else if line.starts_with("use-system-font=") {
            use_system = line.ends_with("true");
        } else if line.starts_with("font=") {
            let font = line.trim_start_matches("font=").trim_matches('\'');
            let font = font.rsplit_once(' ').map(|(name, _)| name).unwrap_or(font);
            if !font.is_empty() {
                found_font = Some(font.to_string());
            }
        }
    }
    if !use_system {
        if let Some(f) = found_font {
            return Some(clean_font_name(&f));
        }
    }

    font_from_gsettings_monospace()
}

// Parse WezTerm config (~/.config/wezterm/wezterm.lua)
fn font_from_wezterm(config_home: &str) -> Option<String> {
    if config_home.is_empty() { return None; }
    let home = env::var("HOME").unwrap_or_default();
    let content = fs::read(format!("{}/wezterm/wezterm.lua", config_home))
        .or_else(|_| fs::read(format!("{}/.wezterm.lua", home)))
        .ok()?;

    // Look for wezterm.font("Font Name"), wezterm.font { family = "Font Name" },
    // or wezterm.font_with_fallback variations
    let pos = memmem::find(&content, b"wezterm.font")
        .filter(|&p| {
            // Make sure it's wezterm.font( or wezterm.font{ or wezterm.font_with_fallback
            let after = &content[p + 12..];
            matches!(after.first(), Some(b'(' | b' ' | b'{' | b'_'))
        })?;
    let after = &content[pos + 12..];

    // Try to find family = "..." pattern (table syntax)
    if let Some(fam_pos) = memmem::find(after, b"family") {
        let after_fam = &after[fam_pos + 6..];
        // Skip whitespace and '='
        let rest = after_fam.iter().position(|&b| b == b'=')?;
        let after_eq = &after_fam[rest + 1..];
        let start = memchr::memchr2(b'"', b'\'', after_eq)?;
        let quote_char = after_eq[start];
        let inner = &after_eq[start + 1..];
        let end = memchr::memchr(quote_char, inner)?;
        let font = std::str::from_utf8(&inner[..end]).ok()?.trim();
        if !font.is_empty() {
            return Some(clean_font_name(font));
        }
    }

    // Fallback: direct string argument wezterm.font("Font Name")
    let start = memchr::memchr2(b'"', b'\'', after)?;
    let quote_char = after[start];
    let inner = &after[start + 1..];
    let end = memchr::memchr(quote_char, inner)?;
    let font = std::str::from_utf8(&inner[..end]).ok()?.trim();
    if !font.is_empty() {
        return Some(clean_font_name(font));
    }
    None
}

// Run a command and return stdout as a trimmed string if successful
fn run_command(cmd: &str, args: &[&str]) -> Option<String> {
    let output = std::process::Command::new(cmd)
        .args(args)
        .output()
        .ok()?;
    if !output.status.success() {
        return None;
    }
    let s = String::from_utf8_lossy(&output.stdout).trim().to_string();
    if s.is_empty() { None } else { Some(s) }
}

// Get a raw gsettings value as a string (no font processing)
fn gsettings_get_raw(schema: &str, key: &str) -> Option<String> {
    run_command("gsettings", &["get", schema, key])
}

// Get font from a gsettings key, stripping trailing size component
fn gsettings_get_font(schema: &str, key: &str) -> Option<String> {
    let raw = gsettings_get_raw(schema, key)?;
    let font = raw.trim_matches('\'');
    let font = font.rsplit_once(' ').map(|(name, _)| name).unwrap_or(font);
    if font.is_empty() { None } else { Some(clean_font_name(font)) }
}

// Get system monospace font via gsettings
fn font_from_gsettings_monospace() -> Option<String> {
    gsettings_get_font("org.gnome.desktop.interface", "monospace-font-name")
}

// Parse font for apps that have a use-system-font toggle in gsettings
fn font_from_gsettings_app(schema: &str, font_key: &str) -> Option<String> {
    let use_system = gsettings_get_raw(schema, "use-system-font");
    match use_system.as_deref() {
        Some("true") | None => font_from_gsettings_monospace(),
        _ => gsettings_get_font(schema, font_key).or_else(font_from_gsettings_monospace),
    }
}

fn font_from_gnome_console() -> Option<String> {
    font_from_gsettings_app("org.gnome.Console", "custom-font")
}

fn font_from_ptyxis() -> Option<String> {
    font_from_gsettings_app("org.gnome.Ptyxis", "font-name")
}

// Check if a font name indicates if its a nerd font
pub fn is_nerd_font(font: &str) -> bool {
    // NF or Nerd Font, this isnt robust because people can set their fonts wrong but its safer than
    // non nerd users getting garbled outputs.
    let lower = font.to_ascii_lowercase();
    lower.contains("nerd") || lower.ends_with(" nf")
}

// Clean up font name - remove style suffixes, normalize, and beautify for display
fn clean_font_name(font: &str) -> String {
    let font = font.trim();

    let lower = font.to_ascii_lowercase();
    let mut result = if matches!(
        lower.as_str(),
        "monospace" | "sans-serif" | "serif" | "mono" | "system-ui"
    ) {
        resolve_font_alias(font)
    } else {
        font.to_string()
    };

    // Remove common style suffixes if they appear at the end (case-insensitive)
    let suffixes: &[&[u8]] = &[
        b" regular", b" medium", b" bold", b" italic", b" light",
        b" thin", b" semibold", b" extrabold", b" black",
    ];

    let lower = result.to_ascii_lowercase();
    for suffix in suffixes {
        if lower.as_bytes().ends_with(suffix) {
            let new_len = result.len() - suffix.len();
            // Safety: suffix is ASCII so boundary is always valid
            if result.is_char_boundary(new_len) {
                result = result[..new_len].to_string();
            }
            break;
        }
    }

    result = result.replace("Nerd Font", "NF");

    if result.len() > 5 && result[result.len() - 5..].eq_ignore_ascii_case(" Mono") {
        result.truncate(result.len() - 5);
    }

    result
}

// Resolve generic font aliases (monospace, sans-serif, etc.) to actual font names
fn resolve_font_alias(font: &str) -> String {
    run_command("fc-match", &[font, "-f", "%{family}"])
        .unwrap_or_else(|| font.to_string())
}