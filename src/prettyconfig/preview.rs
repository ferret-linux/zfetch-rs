// Preview generation for prettyconfig TUI
// Handles live preview updates and section filtering

use crate::configloader::{Config, CoreToggles, HardwareToggles, NerdFontSetting, ThemePreset, UserspaceToggles};
use crate::helpers;
use crate::prettyconfig::navigation::App;
use crate::visuals::colorcontrol;
use crate::visuals::renderer::{self, Section};

// Update the preview based on current app state
pub fn update_preview(app: &mut App) {
    // Set nerd font override based on current setting
    helpers::set_nerd_font_override(match app.nerd_fonts {
        NerdFontSetting::Auto => 0,
        NerdFontSetting::ForceOn => 1,
        NerdFontSetting::ForceOff => 2,
    });

    // Set preview colors based on current theme
    let (border, title, key, value) = app.theme.colors();

    // Get art colors from theme (or use defaults for Default theme)
    let art_colors = app.theme.art_colors();
    let default_art = crate::configloader::DEFAULT_ART_COLORS;

    let preview_colors = crate::configloader::ColorConfig {
        border,
        title,
        key,
        value,
        art_1: art_colors.map_or(default_art[0], |a| a[0]),
        art_2: art_colors.map_or(default_art[1], |a| a[1]),
        art_3: art_colors.map_or(default_art[2], |a| a[2]),
        art_4: art_colors.map_or(default_art[3], |a| a[3]),
        art_5: art_colors.map_or(default_art[4], |a| a[4]),
        art_6: art_colors.map_or(default_art[5], |a| a[5]),
        art_7: art_colors.map_or(default_art[6], |a| a[6]),
        art_8: art_colors.map_or(default_art[7], |a| a[7]),
    };
    colorcontrol::set_preview_colors(Some(preview_colors));

    // Set preview box styles
    renderer::set_preview_box_styles(Some(app.box_style), Some(app.border_line_style));

    // Filter sections based on toggles
    let core = filter_core_section(&app.cached_sections.0, &app.core);
    let hardware = filter_hardware_section(&app.cached_sections.1, &app.hardware);
    let userspace = filter_userspace_section(&app.cached_sections.2, &app.userspace);

    let sections: Vec<_> = [core, hardware, userspace]
        .into_iter()
        .filter(|s| !s.lines.is_empty())
        .collect();

    // Generate full preview (art + sections)
    let (wide, narrow, small) = app.get_art_for_preview();
    let output = renderer::draw_layout(&wide, &narrow, &sections, small.as_deref());
    app.preview_lines = output.lines().map(String::from).collect();

    // Generate sections-only preview for image mode
    app.sections_only_lines = renderer::build_sections_lines(&sections, None);
}

// Detect theme from config colors
pub fn detect_theme_from_config(config: &Config) -> ThemePreset {
    let presets = [
        ThemePreset::Default,
        ThemePreset::Dracula,
        ThemePreset::Catppuccin,
        ThemePreset::Nord,
        ThemePreset::Gruvbox,
        ThemePreset::Eldritch,
        ThemePreset::Kanagawa,
    ];

    for preset in presets {
        let (border, _, _, _) = preset.colors();
        if std::mem::discriminant(&border) == std::mem::discriminant(&config.colors.border) {
            match (border, config.colors.border) {
                (crate::configloader::ThemeColor::Rgb(r1, g1, b1), crate::configloader::ThemeColor::Rgb(r2, g2, b2)) => {
                    if r1 == r2 && g1 == g2 && b1 == b2 {
                        return preset;
                    }
                }
                (crate::configloader::ThemeColor::Ansi(c1), crate::configloader::ThemeColor::Ansi(c2)) => {
                    if c1 == c2 {
                        return preset;
                    }
                }
                _ => {}
            }
        }
    }

    ThemePreset::Default
}

// Filter core section based on toggles
fn filter_core_section(section: &Section, toggles: &CoreToggles) -> Section {
    let lines: Vec<_> = section.lines.iter()
        .filter(|(key, _)| match key.as_str() {
            "OS" => toggles.os,
            "Kernel" => toggles.kernel,
            "Platform" => toggles.platform,
            "Uptime" => toggles.uptime,
            "Init" => toggles.init,
            "OS Age" => toggles.os_age,
            _ => true,
        })
        .cloned()
        .collect();
    Section::new(&section.title, lines)
}

// Filter hardware section based on toggles
fn filter_hardware_section(section: &Section, toggles: &HardwareToggles) -> Section {
    // Branch keys (├─, ╰─) are shared between GPU Both-mode trees and multi-display
    // trees. Track which parent header owns the current branch run to apply the
    // correct toggle instead of blindly mapping all branches to toggles.screen.
    #[derive(Clone, Copy)]
    enum TreeOwner { Gpu, Screen, None }
    let mut owner = TreeOwner::None;
    let mut lines: Vec<(String, String)> = Vec::with_capacity(section.lines.len());

    for (key, value) in &section.lines {
        let keep = match key.as_str() {
            "CPU"                => { owner = TreeOwner::None;   toggles.cpu     }
            "GPU"                => { owner = TreeOwner::None;   toggles.gpu     }
            "GPUs"               => { owner = TreeOwner::Gpu;    toggles.gpu     }
            "Memory"             => { owner = TreeOwner::None;   toggles.memory  }
            "Storage"            => { owner = TreeOwner::None;   toggles.storage }
            "Battery"            => { owner = TreeOwner::None;   toggles.battery }
            "Display" | "Displays" => { owner = TreeOwner::Screen; toggles.screen }
            k if k.starts_with("├") || k.starts_with("╰") => match owner {
                TreeOwner::Gpu    => toggles.gpu,
                TreeOwner::Screen => toggles.screen,
                TreeOwner::None   => true,
            },
            _ => true,
        };

        if keep {
            lines.push((key.clone(), value.clone()));
        } else if matches!(key.as_str(), "GPUs" | "Displays") {
            // Header was filtered — reset owner so its branches don't survive
            owner = TreeOwner::None;
        }
    }

    // Guard: if a "GPUs" header lost all its children somehow, strip the orphan
    if lines.last().map(|(k, _)| k.as_str()) == Some("GPUs") {
        lines.pop();
    }

    Section::new(&section.title, lines)
}

// Filter userspace section based on toggles
fn filter_userspace_section(section: &Section, toggles: &UserspaceToggles) -> Section {
    let lines: Vec<_> = section.lines.iter()
        .filter(|(key, _)| match key.as_str() {
            "Packages" => toggles.packages,
            "Terminal" => toggles.terminal,
            "Shell" => toggles.shell,
            "WM" => toggles.wm,
            "UI" => toggles.ui,
            "Editor" => toggles.editor,
            "Terminal Font" => toggles.terminal_font,
            _ => true,
        })
        .cloned()
        .collect();
    Section::new(&section.title, lines)
}
