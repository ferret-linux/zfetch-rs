// Configuration loader for zfetch
// Loads settings from config.toml

use std::fs;
use std::path::PathBuf;
use memchr::{memchr, memchr_iter};
pub use crate::themes::ThemePreset;

// Embed the default config file at compile time
const DEFAULT_CONFIG: &str = include_str!("config.toml");


// OS art setting - can be disabled, auto-detect, or specific OS
#[derive(Debug, Clone)]
pub enum OsArtSetting {
    Disabled,
    Auto,
    Specific(String),
}

// Theme color - can be RGB or ANSI (0-15)
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ThemeColor {
    Rgb(u8, u8, u8),
    Ansi(u8), // ANSI color code 0-15
}

// Color configuration - theme colors can be RGB or ANSI
// Art colors default to ANSI (terminal scheme) but can be overridden with RGB hex
#[derive(Debug, Clone)]
pub struct ColorConfig {
    // Theme colors
    pub border: ThemeColor,
    pub title: ThemeColor,
    pub key: ThemeColor,
    pub value: ThemeColor,
    // ASCII art colors (1-8) - inkline uses 0-indexed placeholders {0} through {7}
    pub art_1: ThemeColor,
    pub art_2: ThemeColor,
    pub art_3: ThemeColor,
    pub art_4: ThemeColor,
    pub art_5: ThemeColor,
    pub art_6: ThemeColor,
    pub art_7: ThemeColor,
    pub art_8: ThemeColor,
}

// Toggle settings for Core section keys
#[derive(Debug, Clone)]
pub struct CoreToggles {
    pub os: bool,
    pub kernel: bool,
    pub platform: bool,
    pub uptime: bool,
    pub init: bool,
    pub os_age: bool,
}

impl Default for CoreToggles {
    fn default() -> Self {
        Self {
            os: true,
            kernel: true,
            platform: true,
            uptime: true,
            init: true,
            os_age: true,
        }
    }
}

// Toggle settings for Hardware section keys
#[derive(Debug, Clone)]
pub struct HardwareToggles {
    pub cpu: bool,
    pub gpu: bool,
    pub gpu_display: GpuDisplayMode,
    pub memory: bool,
    pub storage: bool,
    pub battery: bool,
    pub screen: bool,
}

impl Default for HardwareToggles {
    fn default() -> Self {
        Self {
            cpu: true,
            gpu: true,
            gpu_display: GpuDisplayMode::default(),
            memory: true,
            storage: true,
            battery: true,
            screen: true,
        }
    }
}

// Toggle settings for Userspace section keys
#[derive(Debug, Clone)]
pub struct UserspaceToggles {
    pub packages: bool,
    pub terminal: bool,
    pub shell: bool,
    pub wm: bool,
    pub ui: bool,
    pub editor: bool,
    pub terminal_font: bool,
}

impl Default for UserspaceToggles {
    fn default() -> Self {
        Self {
            packages: true,
            terminal: true,
            shell: true,
            wm: true,
            ui: true,
            editor: true,
            terminal_font: true,
        }
    }
}

// Default art colors - ANSI codes that respect terminal color scheme
// 8 colors for inkline's 0-indexed {0} through {7} placeholders
pub const DEFAULT_ART_COLORS: [ThemeColor; 8] = [
    ThemeColor::Ansi(9),  // Bright Red
    ThemeColor::Ansi(11), // Bright Yellow
    ThemeColor::Ansi(10), // Bright Green
    ThemeColor::Ansi(14), // Bright Cyan
    ThemeColor::Ansi(12), // Bright Blue
    ThemeColor::Ansi(13), // Bright Magenta
    ThemeColor::Ansi(1),  // Red
    ThemeColor::Ansi(5),  // Magenta
];

impl Default for ColorConfig {
    fn default() -> Self {
        let (border, title, key, value) = ThemePreset::default().colors();
        Self {
            border,
            title,
            key,
            value,
            art_1: DEFAULT_ART_COLORS[0],
            art_2: DEFAULT_ART_COLORS[1],
            art_3: DEFAULT_ART_COLORS[2],
            art_4: DEFAULT_ART_COLORS[3],
            art_5: DEFAULT_ART_COLORS[4],
            art_6: DEFAULT_ART_COLORS[5],
            art_7: DEFAULT_ART_COLORS[6],
            art_8: DEFAULT_ART_COLORS[7],
        }
    }
}

// Nerd font setting - can be auto-detect, forced on, or forced off
#[derive(Debug, Clone, Copy)]
pub enum NerdFontSetting {
    Auto,
    ForceOn,
    ForceOff,
}

// GPU display mode - controls which GPU(s) to show
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub enum GpuDisplayMode {
    #[default]
    Auto,       // Show dGPU if present, else iGPU
    Integrated, // Show only integrated GPU
    Discrete,   // Show only discrete GPU
    Both,       // Show both GPUs in tree-style format
}

impl GpuDisplayMode {
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "auto" => Some(Self::Auto),
            "igpu" | "integrated" => Some(Self::Integrated),
            "dgpu" | "discrete" => Some(Self::Discrete),
            "both" => Some(Self::Both),
            _ => None,
        }
    }
}

// Box corner style - rounded or square
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub enum BoxStyle {
    #[default]
    Rounded,
    Square,
}

impl BoxStyle {
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "rounded" => Some(Self::Rounded),
            "square" => Some(Self::Square),
            _ => None,
        }
    }

/*     pub fn as_str(&self) -> &'static str {
        match self {
            Self::Rounded => "rounded",
            Self::Square => "square",
        }
    } */

    // Get the box drawing characters for this style (for solid/dotted lines)
    pub fn corners(&self) -> (&'static str, &'static str, &'static str, &'static str) {
        match self {
            Self::Rounded => ("╭", "╮", "╰", "╯"),
            Self::Square => ("┌", "┐", "└", "┘"),
        }
    }

    // Get the box drawing characters for double lines
    pub fn corners_double(&self) -> (&'static str, &'static str, &'static str, &'static str) {
        // Double lines always use square-style corners (rounded doesn't exist for double lines)
        ("╔", "╗", "╚", "╝")
    }
}

// Border line style - solid, dotted, or double
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub enum BorderLineStyle {
    #[default]
    Solid,
    Dotted,
    Double,
}

impl BorderLineStyle {
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "solid" => Some(Self::Solid),
            "dotted" => Some(Self::Dotted),
            "double" => Some(Self::Double),
            _ => None,
        }
    }

/*     pub fn as_str(&self) -> &'static str {
        match self {
            Self::Solid => "solid",
            Self::Dotted => "dotted",
            Self::Double => "double",
        }
    } */

    // Get the horizontal and vertical line characters for this style
    pub fn lines(&self) -> (&'static str, &'static str) {
        match self {
            Self::Solid => ("─", "│"),
            Self::Dotted => ("╌", "┊"),
            Self::Double => ("═", "║"),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Config {
    pub os_art: OsArtSetting,
    pub colors: ColorConfig,
    pub custom_art: Option<String>,
    pub image: bool,
    pub image_path: Option<String>,
    pub nerd_fonts: NerdFontSetting,
    pub box_style: BoxStyle,
    pub border_line_style: BorderLineStyle,
    pub core: CoreToggles,
    pub hardware: HardwareToggles,
    pub userspace: UserspaceToggles,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            os_art: OsArtSetting::Disabled,
            colors: ColorConfig::default(),
            custom_art: None,
            image: false,
            image_path: None,
            nerd_fonts: NerdFontSetting::Auto,
            box_style: BoxStyle::default(),
            border_line_style: BorderLineStyle::default(),
            core: CoreToggles::default(),
            hardware: HardwareToggles::default(),
            userspace: UserspaceToggles::default(),
        }
    }
}

// Parse a hex color string like "#FF79C6" or "FF79C6" into RGB tuple
// Handles inline comments like: art_1 = "#FF0000"  # comment
fn parse_hex_color(hex: &str) -> Option<(u8, u8, u8)> {
    let hex = hex.trim();

    // Handle quoted strings: extract content between quotes, ignoring anything after
    let hex = if hex.starts_with('"') {
        // Find closing quote
        let end = hex[1..].find('"')?;
        &hex[1..1 + end]
    } else {
        // Unquoted: strip inline comments (anything after whitespace or #)
        hex.split_whitespace().next().unwrap_or(hex)
    };

    let hex = hex.strip_prefix('#').unwrap_or(hex);

    if hex.len() != 6 {
        return None;
    }

    let r = u8::from_str_radix(&hex[0..2], 16).ok()?;
    let g = u8::from_str_radix(&hex[2..4], 16).ok()?;
    let b = u8::from_str_radix(&hex[4..6], 16).ok()?;

    Some((r, g, b))
}

// Get the config directory path
fn get_config_dir() -> Option<PathBuf> {
    // Prefer XDG_CONFIG_HOME if set
    if let Ok(xdg_config) = std::env::var("XDG_CONFIG_HOME") {
        return Some(PathBuf::from(xdg_config).join("zfetch"));
    }

    // Fall back to ~/.config/zfetch
    if let Ok(home) = std::env::var("HOME") {
        return Some(PathBuf::from(&home).join(".config/zfetch"));
    }

    None
}

// Get the config file path, checking common locations
fn get_config_path() -> Option<PathBuf> {
    // Check XDG_CONFIG_HOME/zfetch/config.toml first
    if let Ok(xdg_config) = std::env::var("XDG_CONFIG_HOME") {
        let path = PathBuf::from(xdg_config).join("zfetch/config.toml");
        if path.exists() {
            return Some(path);
        }
    }

    // Check ~/.config/zfetch/config.toml
    if let Ok(home) = std::env::var("HOME") {
        let path = PathBuf::from(&home).join(".config/zfetch/config.toml");
        if path.exists() {
            return Some(path);
        }
    }

    // Check config.toml in current directory (for development)
    let local_path = PathBuf::from("config.toml");
    if local_path.exists() {
        return Some(local_path);
    }

    None
}

// Install the default config file to ~/.config/zfetch/config.toml
fn install_default_config() -> Option<PathBuf> {
    let config_dir = get_config_dir()?;
    let config_path = config_dir.join("config.toml");

    // Create the config directory if it doesn't exist
    if !config_dir.exists() {
        if fs::create_dir_all(&config_dir).is_err() {
            eprintln!(
                "Warning: Could not create config directory: {:?}",
                config_dir
            );
            return None;
        }
    }

    // Write the default config file
    if fs::write(&config_path, DEFAULT_CONFIG).is_err() {
        eprintln!("Warning: Could not write config file: {:?}", config_path);
        return None;
    }

    eprintln!("Installed default config to: {:?}", config_path);
    Some(config_path)
}

// Load configuration from file
pub fn load_config() -> Config {
    // Try to find an existing config file
    let path = match get_config_path() {
        Some(p) => p,
        None => {
            // No config found, install the default one
            match install_default_config() {
                Some(p) => p,
                None => return Config::default(),
            }
        }
    };

    let content = match fs::read_to_string(&path) {
        Ok(c) => c,
        Err(_) => return Config::default(),
    };

    parse_config(&content)
}

// Track which section currently being parsed
#[derive(Debug, Clone, Copy, PartialEq)]
enum ConfigSection {
    None,
    Display,
    Colors,
    Core,
    Hardware,
    Userspace,
}

// Parse the TOML config content using byte-level operations
fn parse_config(content: &str) -> Config {
    let mut config = Config::default();
    let bytes = content.as_bytes();
    let mut current_section = ConfigSection::None;

    // First pass: look for theme preset (so individual colors can override it)
    let mut start = 0;
    for end in memchr_iter(b'\n', bytes).chain(std::iter::once(bytes.len())) {
        let line = &bytes[start..end];
        start = end + 1;

        if line.is_empty() {
            continue;
        }

        let first_nonws = match line.iter().position(|&b| b != b' ' && b != b'\t') {
            Some(p) => p,
            None => continue,
        };
        let line = &line[first_nonws..];

        if line.first() == Some(&b'#') || line.first() == Some(&b'[') {
            continue;
        }

        let eq_pos = match memchr(b'=', line) {
            Some(p) => p,
            None => continue,
        };

        let key = trim_bytes(&line[..eq_pos]);
        let value = trim_bytes(&line[eq_pos + 1..]);

        if key == b"theme" {
            if value.first() == Some(&b'"') && value.last() == Some(&b'"') && value.len() > 2 {
                if let Ok(theme_name) = std::str::from_utf8(&value[1..value.len() - 1]) {
                    if let Some(preset) = ThemePreset::from_str(theme_name) {
                        // Apply UI colors
                        let (border, title, key, val) = preset.colors();
                        config.colors.border = border;
                        config.colors.title = title;
                        config.colors.key = key;
                        config.colors.value = val;
                        // Apply art colors (if theme provides them)
                        if let Some(art) = preset.art_colors() {
                            config.colors.art_1 = art[0];
                            config.colors.art_2 = art[1];
                            config.colors.art_3 = art[2];
                            config.colors.art_4 = art[3];
                            config.colors.art_5 = art[4];
                            config.colors.art_6 = art[5];
                            config.colors.art_7 = art[6];
                            config.colors.art_8 = art[7];
                        }
                    }
                }
            }
            break;
        }
    }

    // Second pass: process all settings (individual colors override theme)
    start = 0;
    for end in memchr_iter(b'\n', bytes).chain(std::iter::once(bytes.len())) {
        let line = &bytes[start..end];
        start = end + 1;

        // Skip empty lines
        if line.is_empty() {
            continue;
        }

        // Find first non-whitespace
        let first_nonws = match line.iter().position(|&b| b != b' ' && b != b'\t') {
            Some(p) => p,
            None => continue,
        };
        let line = &line[first_nonws..];

        // Skip comments
        if line.first() == Some(&b'#') {
            continue;
        }

        // Track sections
        if line.first() == Some(&b'[') {
            current_section = if line == b"[colors]" {
                ConfigSection::Colors
            } else if line == b"[display]" {
                ConfigSection::Display
            } else if line == b"[core]" {
                ConfigSection::Core
            } else if line == b"[hardware]" {
                ConfigSection::Hardware
            } else if line == b"[userspace]" {
                ConfigSection::Userspace
            } else {
                ConfigSection::None
            };
            continue;
        }

        // Find '=' for key=value parsing
        let eq_pos = match memchr(b'=', line) {
            Some(p) => p,
            None => continue,
        };

        let key = &line[..eq_pos];
        let value = &line[eq_pos + 1..];

        // Trim key and value
        let key = trim_bytes(key);
        let value = trim_bytes(value);

        // Parse color settings
        if current_section == ConfigSection::Colors {
            if let (Ok(key_str), Ok(value_str)) = (std::str::from_utf8(key), std::str::from_utf8(value)) {
                if let Some(color) = parse_hex_color(value_str) {
                    match key_str {
                        "border" => config.colors.border = ThemeColor::Rgb(color.0, color.1, color.2),
                        "title" => config.colors.title = ThemeColor::Rgb(color.0, color.1, color.2),
                        "key" => config.colors.key = ThemeColor::Rgb(color.0, color.1, color.2),
                        "value" => config.colors.value = ThemeColor::Rgb(color.0, color.1, color.2),
                        "art_1" => config.colors.art_1 = ThemeColor::Rgb(color.0, color.1, color.2),
                        "art_2" => config.colors.art_2 = ThemeColor::Rgb(color.0, color.1, color.2),
                        "art_3" => config.colors.art_3 = ThemeColor::Rgb(color.0, color.1, color.2),
                        "art_4" => config.colors.art_4 = ThemeColor::Rgb(color.0, color.1, color.2),
                        "art_5" => config.colors.art_5 = ThemeColor::Rgb(color.0, color.1, color.2),
                        "art_6" => config.colors.art_6 = ThemeColor::Rgb(color.0, color.1, color.2),
                        "art_7" => config.colors.art_7 = ThemeColor::Rgb(color.0, color.1, color.2),
                        "art_8" => config.colors.art_8 = ThemeColor::Rgb(color.0, color.1, color.2),
                        _ => {}
                    }
                }
            }
            continue;
        }

        // Parse os_art setting
        if key == b"os_art" {
            if value == b"true" {
                config.os_art = OsArtSetting::Auto;
            } else if value == b"false" {
                config.os_art = OsArtSetting::Disabled;
            } else if value.first() == Some(&b'"') && value.last() == Some(&b'"') && value.len() > 2 {
                // Quoted string: os_art = "arch"
                if let Ok(os_name) = std::str::from_utf8(&value[1..value.len() - 1]) {
                    if !os_name.is_empty() {
                        config.os_art = OsArtSetting::Specific(os_name.to_string());
                    }
                }
            } else if !value.is_empty() {
                // Unquoted string: os_art = arch
                if let Ok(os_name) = std::str::from_utf8(value) {
                    config.os_art = OsArtSetting::Specific(os_name.to_string());
                }
            }
        }

        // Parse custom_art setting
        if key == b"custom_art" {
            if value.first() == Some(&b'"') && value.last() == Some(&b'"') && value.len() > 2 {
                if let Ok(path) = std::str::from_utf8(&value[1..value.len() - 1]) {
                    if !path.is_empty() {
                        let expanded = expand_home_path(path);
                        config.custom_art = Some(expanded);
                    }
                }
            }
        }

        // Parse image toggle (but not image_path)
        if key == b"image" {
            config.image = value == b"true";
        }

        // Parse image_path setting
        if key == b"image_path" {
            if value.first() == Some(&b'"') && value.last() == Some(&b'"') && value.len() > 2 {
                if let Ok(path) = std::str::from_utf8(&value[1..value.len() - 1]) {
                    if !path.is_empty() {
                        let expanded = expand_home_path(path);
                        config.image_path = Some(expanded);
                    }
                }
            }
        }

        // Parse nerd_fonts setting
        if key == b"nerd_fonts" {
            if value == b"true" {
                config.nerd_fonts = NerdFontSetting::ForceOn;
            } else if value == b"false" {
                config.nerd_fonts = NerdFontSetting::ForceOff;
            }
        }

        // Parse box_style setting (rounded or square corners)
        if key == b"box_style" {
            if value.first() == Some(&b'"') && value.last() == Some(&b'"') && value.len() > 2 {
                if let Ok(style_name) = std::str::from_utf8(&value[1..value.len() - 1]) {
                    if let Some(style) = BoxStyle::from_str(style_name) {
                        config.box_style = style;
                    }
                }
            } else if let Ok(style_name) = std::str::from_utf8(value) {
                if let Some(style) = BoxStyle::from_str(style_name) {
                    config.box_style = style;
                }
            }
        }

        // Parse border_line_style setting (solid, dotted, or double lines)
        if key == b"border_line_style" {
            if value.first() == Some(&b'"') && value.last() == Some(&b'"') && value.len() > 2 {
                if let Ok(style_name) = std::str::from_utf8(&value[1..value.len() - 1]) {
                    if let Some(style) = BorderLineStyle::from_str(style_name) {
                        config.border_line_style = style;
                    }
                }
            } else if let Ok(style_name) = std::str::from_utf8(value) {
                if let Some(style) = BorderLineStyle::from_str(style_name) {
                    config.border_line_style = style;
                }
            }
        }

        // Parse core section toggles
        if current_section == ConfigSection::Core {
            let is_true = value == b"true";
            match key {
                b"os" => config.core.os = is_true,
                b"kernel" => config.core.kernel = is_true,
                b"platform" => config.core.platform = is_true,
                b"uptime" => config.core.uptime = is_true,
                b"init" => config.core.init = is_true,
                b"os_age" => config.core.os_age = is_true,
                _ => {}
            }
        }

        // Parse hardware section toggles
        if current_section == ConfigSection::Hardware {
            let is_true = value == b"true";
            match key {
                b"cpu" => config.hardware.cpu = is_true,
                b"gpu" => config.hardware.gpu = is_true,
                b"gpu_display" => {
                    // Parse gpu_display mode (quoted or unquoted)
                    if value.first() == Some(&b'"') && value.last() == Some(&b'"') && value.len() > 2 {
                        if let Ok(mode_name) = std::str::from_utf8(&value[1..value.len() - 1]) {
                            if let Some(mode) = GpuDisplayMode::from_str(mode_name) {
                                config.hardware.gpu_display = mode;
                            }
                        }
                    } else if let Ok(mode_name) = std::str::from_utf8(value) {
                        if let Some(mode) = GpuDisplayMode::from_str(mode_name) {
                            config.hardware.gpu_display = mode;
                        }
                    }
                }
                b"memory" => config.hardware.memory = is_true,
                b"storage" => config.hardware.storage = is_true,
                b"battery" => config.hardware.battery = is_true,
                b"screen" => config.hardware.screen = is_true,
                _ => {}
            }
        }

        // Parse userspace section toggles
        if current_section == ConfigSection::Userspace {
            let is_true = value == b"true";
            match key {
                b"packages" => config.userspace.packages = is_true,
                b"terminal" => config.userspace.terminal = is_true,
                b"shell" => config.userspace.shell = is_true,
                b"wm" => config.userspace.wm = is_true,
                b"ui" => config.userspace.ui = is_true,
                b"editor" => config.userspace.editor = is_true,
                b"terminal_font" => config.userspace.terminal_font = is_true,
                _ => {}
            }
        }
    }

    config
}

// Trim leading and trailing whitespace from bytes
#[inline]
fn trim_bytes(bytes: &[u8]) -> &[u8] {
    let start = bytes.iter().position(|&b| b != b' ' && b != b'\t').unwrap_or(bytes.len());
    let end = bytes.iter().rposition(|&b| b != b' ' && b != b'\t').map(|p| p + 1).unwrap_or(start);
    &bytes[start..end]
}

// Expand ~ to home directory
#[inline]
fn expand_home_path(path: &str) -> String {
    if path.starts_with("~/") {
        if let Ok(home) = std::env::var("HOME") {
            return path.replacen("~", &home, 1);
        }
    }
    path.to_string()
}

// Migrate user config file by adding any new fields from the default config.
// Strategy: Extract user's active settings, write fresh default config, reapply user settings.
// Called manually via -u/--update flag.
pub fn migrate_config() {
    let user_path = match get_config_path() {
        Some(p) => p,
        None => {
            eprintln!("No config file found. Run zfetch once to create one.");
            return;
        }
    };

    let user_content = match fs::read_to_string(&user_path) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("Could not read config file: {}", e);
            return;
        }
    };

    // Extract user's uncommented (active) settings
    let user_settings = extract_active_settings(&user_content);

    // If user has no active settings, just replace with default
    if user_settings.is_empty() {
        if user_content.trim() != DEFAULT_CONFIG.trim() {
            if let Err(e) = fs::write(&user_path, DEFAULT_CONFIG) {
                eprintln!("Could not update config file: {}", e);
            } else {
                eprintln!("Config updated to latest version: {:?}", user_path);
            }
        } else {
            eprintln!("Config is already up to date.");
        }
        return;
    }

    // Apply user settings to fresh default config
    let new_content = apply_settings_to_default(&user_settings);

    if new_content != user_content {
        if let Err(e) = fs::write(&user_path, &new_content) {
            eprintln!("Could not update config file: {}", e);
        } else {
            eprintln!("Config updated to latest version: {:?}", user_path);
        }
    } else {
        eprintln!("Config is already up to date.");
    }
}

// Extract active (uncommented) key=value settings from user config.
// Returns Vec of (section, key, full_line) tuples.
fn extract_active_settings(content: &str) -> Vec<(String, String, String)> {
    let mut settings = Vec::new();
    let mut current_section = String::new();

    for line in content.lines() {
        let trimmed = line.trim();

        // Track section headers
        if trimmed.starts_with('[') && trimmed.ends_with(']') {
            current_section = trimmed[1..trimmed.len() - 1].to_string();
            continue;
        }

        // Skip empty lines and comments
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }

        // Parse key = value
        if let Some(eq_pos) = trimmed.find('=') {
            let key = trimmed[..eq_pos].trim();
            if !key.is_empty() && key.chars().all(|c| c.is_alphanumeric() || c == '_') {
                settings.push((current_section.clone(), key.to_string(), line.to_string()));
            }
        }
    }

    settings
}

// Apply user settings to the default config template.
// For each user setting, find and uncomment/replace the first matching line in default.
fn apply_settings_to_default(user_settings: &[(String, String, String)]) -> String {
    use std::collections::HashSet;

    let mut lines: Vec<String> = DEFAULT_CONFIG.lines().map(|s| s.to_string()).collect();
    let mut current_section = String::new();
    // Track which (section, key) pairs have already been applied
    let mut applied: HashSet<(String, String)> = HashSet::new();

    for line in lines.iter_mut() {
        let trimmed = line.trim();

        // Track section headers
        if trimmed.starts_with('[') && trimmed.ends_with(']') {
            current_section = trimmed[1..trimmed.len() - 1].to_string();
            continue;
        }

        // Check if this line (commented or not) matches a user setting
        let check_line = if trimmed.starts_with('#') {
            trimmed.trim_start_matches('#').trim_start()
        } else {
            trimmed
        };

        if let Some(eq_pos) = check_line.find('=') {
            let key = check_line[..eq_pos].trim();
            if !key.is_empty() && key.chars().all(|c| c.is_alphanumeric() || c == '_') {
                let lookup = (current_section.clone(), key.to_string());
                // Skip if already applied a setting for this section+key
                if applied.contains(&lookup) {
                    continue;
                }
                // Look for matching user setting
                for (section, user_key, user_line) in user_settings {
                    if section == &current_section && user_key == key {
                        *line = user_line.clone();
                        applied.insert(lookup);
                        break;
                    }
                }
            }
        }
    }

    lines.join("\n") + "\n"
}