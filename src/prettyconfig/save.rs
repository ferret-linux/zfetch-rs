// Config file saving for prettyconfig
// Generates TOML content matching the original config.toml format

use crate::configloader::{
    BorderLineStyle, BoxStyle, CoreToggles, GpuDisplayMode, HardwareToggles, NerdFontSetting, OsArtSetting, ThemePreset, UserspaceToggles,
};
use std::fs;
use std::path::PathBuf;

// Get the config file path for saving
fn get_config_path() -> Option<PathBuf> {
    // Prefer XDG_CONFIG_HOME if set
    if let Ok(xdg_config) = std::env::var("XDG_CONFIG_HOME") {
        return Some(PathBuf::from(xdg_config).join("zfetch/config.toml"));
    }

    // Fall back to ~/.config/zfetch
    if let Ok(home) = std::env::var("HOME") {
        return Some(PathBuf::from(&home).join(".config/zfetch/config.toml"));
    }

    None
}

// Format a ThemePreset as TOML
fn format_theme(preset: ThemePreset) -> &'static str {
    match preset {
        ThemePreset::Default => "default",
        ThemePreset::Dracula => "dracula",
        ThemePreset::Catppuccin => "catppuccin",
        ThemePreset::Nord => "nord",
        ThemePreset::Gruvbox => "gruvbox",
        ThemePreset::Eldritch => "eldritch",
        ThemePreset::Kanagawa => "kanagawa",
        ThemePreset::RosePine => "rosepine",
        ThemePreset::TokyoNight => "tokyonight",
        ThemePreset::OneDark => "onedark",
    }
}

/// Collapse path with ~ if in home directory
fn collapse_home_path(path: &str) -> String {
    if let Ok(home) = std::env::var("HOME") {
        if path.starts_with(&home) {
            return path.replacen(&home, "~", 1);
        }
    }
    path.to_string()
}

// Generate TOML config content
pub fn generate_config_toml(
    theme: ThemePreset,
    nerd_fonts: NerdFontSetting,
    os_art: &OsArtSetting,
    custom_art: &Option<String>,
    image: bool,
    image_path: &Option<String>,
    box_style: BoxStyle,
    border_line_style: BorderLineStyle,
    gpu_display: GpuDisplayMode,
    core: &CoreToggles,
    hardware: &HardwareToggles,
    userspace: &UserspaceToggles,
) -> String {
    let mut output = String::new();

    output.push_str("## zfetch Configuration\n\n");

    // [display] section
    output.push_str("[display]\n");
    output.push_str("## Show OS-specific art instead of default zfetch logo\n");
    output.push_str("## Set to true to auto-detect OS, or specify OS name to force that logo\n");

    match os_art {
        OsArtSetting::Disabled => output.push_str("# os_art = false\n"),
        OsArtSetting::Auto => output.push_str("os_art = true\n"),
        OsArtSetting::Specific(os) => output.push_str(&format!("os_art = \"{}\"\n", os)),
    }

    output.push_str("\n## Custom ASCII art file path. (overrides default and OS art)\n");
    output.push_str("## The file can use {0} through {7} for color placeholders.\n");
    if let Some(path) = custom_art {
        output.push_str(&format!("custom_art = \"{}\"\n", collapse_home_path(path)));
    } else {
        output.push_str("# custom_art = \"~/.config/zfetch/my_art.txt\"\n");
    }

    output.push_str("\n## Display image instead of ASCII art (uses Kitty graphics protocol)\n");
    output.push_str("## Set to true to always show an image (uses default zfetch image if no path set)\n");
    if image {
        output.push_str("image = true\n");
    } else {
        output.push_str("# image = false\n");
    }
    output.push_str("## Optionally set a custom image path (supports ~ for home directory)\n");
    if let Some(path) = image_path {
        output.push_str(&format!("image_path = \"{}\"\n", collapse_home_path(path)));
    } else {
        output.push_str("# image_path = \"~/.config/zfetch/image.png\"\n");
    }

    output.push_str("\n## Nerd Font icons - auto-detect, force on, or force off\n");
    match nerd_fonts {
        NerdFontSetting::Auto => output.push_str("# nerd_fonts = true\n"),
        NerdFontSetting::ForceOn => output.push_str("nerd_fonts = true\n"),
        NerdFontSetting::ForceOff => output.push_str("nerd_fonts = false\n"),
    }

    output.push_str("\n## Box corner style: \"rounded\" (default) or \"square\"\n");
    match box_style {
        BoxStyle::Rounded => output.push_str("# box_style = \"rounded\"\n"),
        BoxStyle::Square => output.push_str("box_style = \"square\"\n"),
    }

    output.push_str("\n## Border line style: \"solid\" (default), \"dotted\", or \"double\"\n");
    match border_line_style {
        BorderLineStyle::Solid => output.push_str("# border_line_style = \"solid\"\n"),
        BorderLineStyle::Dotted => output.push_str("border_line_style = \"dotted\"\n"),
        BorderLineStyle::Double => output.push_str("border_line_style = \"double\"\n"),
    }

    // colorssection
    output.push_str("\n[colors]\n");
    output.push_str("## Theme preset - sets border, title, key, and value colors.\n");
    output.push_str("## Available: \"default\" or \"tty\", \"dracula\", \"catppuccin\", \"nord\", \"gruvbox\", \"eldritch\", \"kanagawa\"\n");
    output.push_str("## Default uses your terminal's color scheme.\n");
    if matches!(theme, ThemePreset::Default) {
        output.push_str("# theme = \"default\"\n");
    } else {
        output.push_str(&format!("theme = \"{}\"\n", format_theme(theme)));
    }

    output.push_str("\n## Individual theme colors - override the presets above (use web hex format)\n");
    output.push_str("# border = \"#FF79C6\"  # Box borders\n");
    output.push_str("# title = \"#FF79C6\"   # Section titles\n");
    output.push_str("# key = \"#BD93F9\"     # Info keys\n");
    output.push_str("# value = \"#8BE9FD\"   # Info values\n");

    output.push_str("\n## ASCII art colors - maps to {0} through {7} in art files.\n");
    output.push_str("## Default: rainbow spectrum.\n\n");
    output.push_str("# art_1 = \"#FF0000\"   # {0} - Red\n");
    output.push_str("# art_2 = \"#FF8000\"   # {1} - Orange\n");
    output.push_str("# art_3 = \"#FFFF00\"   # {2} - Yellow\n");
    output.push_str("# art_4 = \"#00FF00\"   # {3} - Green\n");
    output.push_str("# art_5 = \"#00FFFF\"   # {4} - Cyan\n");
    output.push_str("# art_6 = \"#00BFFF\"   # {5} - Light Blue\n");
    output.push_str("# art_7 = \"#5555FF\"   # {6} - Blue\n");
    output.push_str("# art_8 = \"#AA55FF\"   # {7} - Violet\n");

    // core section
    output.push_str("\n[core]\n");
    output.push_str("## Toggle which items to show in Core.\n");
    write_bool_setting(&mut output, "os", core.os, true);
    write_bool_setting(&mut output, "kernel", core.kernel, true);
    write_bool_setting(&mut output, "platform", core.platform, true);
    write_bool_setting(&mut output, "uptime", core.uptime, true);
    write_bool_setting(&mut output, "init", core.init, true);
    write_bool_setting(&mut output, "os_age", core.os_age, true);

    // ardware section
    output.push_str("\n[hardware]\n");
    output.push_str("## Toggle which items to show in Hardware.\n");
    output.push_str("## Note: battery can be set to true on a desktop, will only display if your device is portable.\n");
    write_bool_setting(&mut output, "cpu", hardware.cpu, true);
    write_bool_setting(&mut output, "gpu", hardware.gpu, true);
    output.push_str("## GPU display mode: \"auto\" (default), \"igpu\", \"dgpu\", or \"both\"\n");
    output.push_str("## - auto: Shows discrete GPU if present, otherwise integrated GPU\n");
    output.push_str("## - igpu/integrated: Shows only integrated GPU\n");
    output.push_str("## - dgpu/discrete: Shows only discrete GPU\n");
    output.push_str("## - both: Shows both GPUs in tree-style format\n");
    match gpu_display {
        GpuDisplayMode::Auto => output.push_str("# gpu_display = \"auto\"\n"),
        GpuDisplayMode::Integrated => output.push_str("gpu_display = \"igpu\"\n"),
        GpuDisplayMode::Discrete => output.push_str("gpu_display = \"dgpu\"\n"),
        GpuDisplayMode::Both => output.push_str("gpu_display = \"both\"\n"),
    }
    write_bool_setting(&mut output, "memory", hardware.memory, true);
    write_bool_setting(&mut output, "storage", hardware.storage, true);
    write_bool_setting(&mut output, "battery", hardware.battery, true);
    write_bool_setting(&mut output, "screen", hardware.screen, true);

    // userspace section
    output.push_str("\n[userspace]\n");
    output.push_str("## Toggle which items to show in Userspace.\n");
    write_bool_setting(&mut output, "packages", userspace.packages, true);
    write_bool_setting(&mut output, "terminal", userspace.terminal, true);
    write_bool_setting(&mut output, "shell", userspace.shell, true);
    write_bool_setting(&mut output, "wm", userspace.wm, true);
    write_bool_setting(&mut output, "ui", userspace.ui, true);
    write_bool_setting(&mut output, "editor", userspace.editor, true);
    write_bool_setting(&mut output, "terminal_font", userspace.terminal_font, true);
    output
}

// Write a boolean setting, commenting it out if it matches default
fn write_bool_setting(output: &mut String, key: &str, value: bool, default: bool) {
    if value == default {
        output.push_str(&format!("# {} = {}\n", key, value));
    } else {
        output.push_str(&format!("{} = {}\n", key, value));
    }
}

//Save config to file
pub fn save_config(
    theme: ThemePreset,
    nerd_fonts: NerdFontSetting,
    os_art: &OsArtSetting,
    custom_art: &Option<String>,
    image: bool,
    image_path: &Option<String>,
    box_style: BoxStyle,
    border_line_style: BorderLineStyle,
    gpu_display: GpuDisplayMode,
    core: &CoreToggles,
    hardware: &HardwareToggles,
    userspace: &UserspaceToggles,
) -> Result<PathBuf, String> {
    let path = get_config_path().ok_or("Could not determine config path")?;

    // Ensure parent directory exists
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|e| format!("Could not create config directory: {}", e))?;
    }

    let content = generate_config_toml(theme, nerd_fonts, os_art, custom_art, image, image_path, box_style, border_line_style, gpu_display, core, hardware, userspace);

    fs::write(&path, content).map_err(|e| format!("Could not write config file: {}", e))?;

    Ok(path)
}
