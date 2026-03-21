// Navigation and state management for prettyconfig TUI
// Contains FocusArea enum, App struct, and navigation methods

use crate::configloader::{
    BorderLineStyle, BoxStyle, Config, CoreToggles, GpuDisplayMode, HardwareToggles, NerdFontSetting, OsArtSetting, ThemePreset, UserspaceToggles,
};
use crate::dostuff;
use crate::modules::ascii;
use crate::prettyconfig::helpers::{theme_color_to_ratatui};
use crate::prettyconfig::preview::detect_theme_from_config;
use crate::visuals::renderer::Section;

use ratatui::layout::Rect;
use ratatui::style::Color;

// Focus areas for Tab navigation
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum FocusArea {
    General,   // Theme, Nerd Fonts toggle
    Art,       // OS Art, Custom Art path, Image toggle, Image path
    Core,      // Core toggles
    Hardware,  // Hardware toggles
    Userspace, // Userspace toggles
}

impl FocusArea {
    pub fn next(self) -> Self {
        match self {
            Self::General => Self::Art,
            Self::Art => Self::Core,
            Self::Core => Self::Hardware,
            Self::Hardware => Self::Userspace,
            Self::Userspace => Self::General,
        }
    }

    pub fn prev(self) -> Self {
        match self {
            Self::General => Self::Userspace,
            Self::Art => Self::General,
            Self::Core => Self::Art,
            Self::Hardware => Self::Core,
            Self::Userspace => Self::Hardware,
        }
    }

    pub fn max_index(self) -> usize {
        match self {
            Self::General => 4,   // Theme, Nerd Fonts, Box Style, Border Lines, GPU Display
            Self::Art => 3,       // OS Art, Custom Art, Image Enabled, Image Path
            Self::Core => 4,      // OS, Kernel, Uptime, Init, OS Age
            Self::Hardware => 5,  // CPU, GPU, Memory, Storage, Battery, Screen
            Self::Userspace => 6, // Packages, Terminal, Shell, WM, UI, Editor, Term Font
        }
    }
}

// Application state
pub struct App {
    // Config values
    pub theme: ThemePreset,
    pub nerd_fonts: NerdFontSetting,
    pub os_art: OsArtSetting,
    pub custom_art: Option<String>,
    pub image: bool,
    pub image_path: Option<String>,
    pub box_style: BoxStyle,
    pub border_line_style: BorderLineStyle,
    pub gpu_display: GpuDisplayMode,
    pub core: CoreToggles,
    pub hardware: HardwareToggles,
    pub userspace: UserspaceToggles,

    // Navigation - Tab switches focus area, Up/Down selects within area
    pub focus: FocusArea,
    pub index: usize,

    // Text editing
    pub editing: bool,
    pub edit_buffer: String,
    pub cursor_pos: usize,

    // Cached preview data
    pub preview_lines: Vec<String>,
    pub sections_only_lines: Vec<String>, // For image mode: just the sections without art
    pub cached_sections: (Section, Section, Section),

    // Status message
    pub status_message: Option<String>,

    // Exit flag
    pub should_exit: bool,

    // Layout regions for mouse hit-testing
    pub layout: LayoutRegions,
}

// Cached layout regions for mouse click detection
#[derive(Default, Clone)]
pub struct LayoutRegions {
    pub general_box: Rect,
    pub art_box: Rect,
    pub core_box: Rect,
    pub hardware_box: Rect,
    pub userspace_box: Rect,
    pub save_button: Rect,
    pub cancel_button: Rect,
}

impl App {
    pub fn from_config(config: &Config) -> Self {
        let theme = detect_theme_from_config(config);

        // Load sections with ALL toggles enabled for preview
        let mut full_config = config.clone();
        full_config.core = CoreToggles { os: true, kernel: true, uptime: true, init: true, os_age: true, platform: true };
        full_config.hardware = HardwareToggles {
            cpu: true, gpu: true, gpu_display: config.hardware.gpu_display, memory: true, storage: true, battery: true, screen: true,
        };
        full_config.userspace = UserspaceToggles {
            packages: true, terminal: true, shell: true, wm: true, ui: true, editor: true, terminal_font: true,
        };
        let sections = dostuff::load_sections(&full_config);

        let mut app = Self {
            theme,
            nerd_fonts: config.nerd_fonts,
            os_art: config.os_art.clone(),
            custom_art: config.custom_art.clone(),
            image: config.image,
            image_path: config.image_path.clone(),
            box_style: config.box_style,
            border_line_style: config.border_line_style,
            gpu_display: config.hardware.gpu_display,
            core: config.core.clone(),
            hardware: config.hardware.clone(),
            userspace: config.userspace.clone(),

            focus: FocusArea::General,
            index: 0,

            editing: false,
            edit_buffer: String::new(),
            cursor_pos: 0,

            preview_lines: Vec::new(),
            sections_only_lines: Vec::new(),
            cached_sections: sections,

            status_message: None,
            should_exit: false,

            layout: LayoutRegions::default(),
        };

        app.update_preview();
        app
    }

    pub fn colors(&self) -> (Color, Color, Color, Color) {
        let (border, title, key, value) = self.theme.colors();
        (
            theme_color_to_ratatui(border),
            theme_color_to_ratatui(title),
            theme_color_to_ratatui(key),
            theme_color_to_ratatui(value),
        )
    }

    pub fn get_art_for_preview(&self) -> (Vec<String>, Vec<String>, Option<Vec<String>>) {
        let wide_logo = ascii::get_wide_logo_lines();
        let narrow_logo = ascii::get_narrow_logo_lines();

        if let Some(ref custom_path) = self.custom_art {
            if let Some(custom_art) = ascii::get_custom_art_lines(custom_path) {
                return (custom_art.clone(), custom_art, None);
            }
        }

        let os_name: String = self.cached_sections.0
            .lines
            .iter()
            .find(|(k, _)| k == "OS")
            .map(|(_, v)| v.clone())
            .unwrap_or_default();

        match &self.os_art {
            OsArtSetting::Disabled => (wide_logo, narrow_logo, None),
            OsArtSetting::Auto => {
                if let Some(os_logo) = ascii::get_os_logo_lines(&os_name) {
                    let small_logo = ascii::get_os_logo_lines_small(&os_name);
                    (os_logo.clone(), os_logo, small_logo)
                } else {
                    (wide_logo, narrow_logo, None)
                }
            }
            OsArtSetting::Specific(specific_os) => {
                if let Some(os_logo) = ascii::get_os_logo_lines(specific_os) {
                    let small_logo = ascii::get_os_logo_lines_small(specific_os);
                    (os_logo.clone(), os_logo, small_logo)
                } else {
                    (wide_logo, narrow_logo, None)
                }
            }
        }
    }

    // Navigation methods
    pub fn next_focus(&mut self) {
        self.focus = self.focus.next();
        self.index = 0;
    }

    pub fn prev_focus(&mut self) {
        self.focus = self.focus.prev();
        self.index = 0;
    }

    pub fn move_up(&mut self) {
        if self.index > 0 {
            self.index -= 1;
        }
    }

    pub fn move_down(&mut self) {
        if self.index < self.focus.max_index() {
            self.index += 1;
        }
    }

    pub fn cycle_os_art_next(&mut self) {
        self.os_art = match &self.os_art {
            OsArtSetting::Disabled => OsArtSetting::Auto,
            OsArtSetting::Auto => OsArtSetting::Disabled,
            OsArtSetting::Specific(_) => OsArtSetting::Disabled,
        };
    }

    pub fn cycle_os_art_prev(&mut self) {
        self.os_art = match &self.os_art {
            OsArtSetting::Disabled => OsArtSetting::Auto,
            OsArtSetting::Auto => OsArtSetting::Disabled,
            OsArtSetting::Specific(_) => OsArtSetting::Auto,
        };
    }

    pub fn cycle_nerd_fonts_next(&mut self) {
        self.nerd_fonts = match self.nerd_fonts {
            NerdFontSetting::Auto => NerdFontSetting::ForceOn,
            NerdFontSetting::ForceOn => NerdFontSetting::ForceOff,
            NerdFontSetting::ForceOff => NerdFontSetting::Auto,
        };
        self.reload_sections_for_nerd_fonts();
    }

    pub fn cycle_nerd_fonts_prev(&mut self) {
        self.nerd_fonts = match self.nerd_fonts {
            NerdFontSetting::Auto => NerdFontSetting::ForceOff,
            NerdFontSetting::ForceOn => NerdFontSetting::Auto,
            NerdFontSetting::ForceOff => NerdFontSetting::ForceOn,
        };
        self.reload_sections_for_nerd_fonts();
    }

    pub fn cycle_box_style_next(&mut self) {
        self.box_style = match self.box_style {
            BoxStyle::Rounded => BoxStyle::Square,
            BoxStyle::Square => BoxStyle::Rounded,
        };
    }

    pub fn cycle_box_style_prev(&mut self) {
        self.cycle_box_style_next(); // Only 2 options, so prev == next
    }

    pub fn cycle_border_line_style_next(&mut self) {
        self.border_line_style = match self.border_line_style {
            BorderLineStyle::Solid => BorderLineStyle::Dotted,
            BorderLineStyle::Dotted => BorderLineStyle::Double,
            BorderLineStyle::Double => BorderLineStyle::Solid,
        };
    }

    pub fn cycle_border_line_style_prev(&mut self) {
        self.border_line_style = match self.border_line_style {
            BorderLineStyle::Solid => BorderLineStyle::Double,
            BorderLineStyle::Dotted => BorderLineStyle::Solid,
            BorderLineStyle::Double => BorderLineStyle::Dotted,
        };
    }

    pub fn cycle_gpu_display_next(&mut self) {
        self.gpu_display = match self.gpu_display {
            GpuDisplayMode::Auto => GpuDisplayMode::Integrated,
            GpuDisplayMode::Integrated => GpuDisplayMode::Discrete,
            GpuDisplayMode::Discrete => GpuDisplayMode::Both,
            GpuDisplayMode::Both => GpuDisplayMode::Auto,
        };
        self.reload_sections_for_gpu_display();
    }

    pub fn cycle_gpu_display_prev(&mut self) {
        self.gpu_display = match self.gpu_display {
            GpuDisplayMode::Auto => GpuDisplayMode::Both,
            GpuDisplayMode::Integrated => GpuDisplayMode::Auto,
            GpuDisplayMode::Discrete => GpuDisplayMode::Integrated,
            GpuDisplayMode::Both => GpuDisplayMode::Discrete,
        };
        self.reload_sections_for_gpu_display();
    }

    fn reload_sections_for_nerd_fonts(&mut self) {
        // Set the nerd font override before reloading sections
        crate::helpers::set_nerd_font_override(match self.nerd_fonts {
            NerdFontSetting::Auto => 0,
            NerdFontSetting::ForceOn => 1,
            NerdFontSetting::ForceOff => 2,
        });

        // Reload sections with all toggles enabled to get new bar styles
        let full_config = Config {
            os_art: self.os_art.clone(),
            colors: crate::configloader::ColorConfig::default(),
            custom_art: self.custom_art.clone(),
            image: self.image,
            image_path: self.image_path.clone(),
            nerd_fonts: self.nerd_fonts,
            box_style: self.box_style,
            border_line_style: self.border_line_style,
            core: CoreToggles { os: true, kernel: true, uptime: true, init: true, os_age: true, platform: true },
            hardware: HardwareToggles {
                cpu: true, gpu: true, gpu_display: self.gpu_display, memory: true, storage: true, battery: true, screen: true,
            },
            userspace: UserspaceToggles {
                packages: true, terminal: true, shell: true, wm: true, ui: true, editor: true, terminal_font: true,
            },
        };
        self.cached_sections = dostuff::load_sections(&full_config);
    }

    fn reload_sections_for_gpu_display(&mut self) {
        // Reload sections with the new gpu_display mode
        let full_config = Config {
            os_art: self.os_art.clone(),
            colors: crate::configloader::ColorConfig::default(),
            custom_art: self.custom_art.clone(),
            image: self.image,
            image_path: self.image_path.clone(),
            nerd_fonts: self.nerd_fonts,
            box_style: self.box_style,
            border_line_style: self.border_line_style,
            core: CoreToggles { os: true, kernel: true, uptime: true, init: true, os_age: true, platform: true },
            hardware: HardwareToggles {
                cpu: true, gpu: true, gpu_display: self.gpu_display, memory: true, storage: true, battery: true, screen: true,
            },
            userspace: UserspaceToggles {
                packages: true, terminal: true, shell: true, wm: true, ui: true, editor: true, terminal_font: true,
            },
        };
        self.cached_sections = dostuff::load_sections(&full_config);
    }

    // Text editing methods
    pub fn start_editing(&mut self, initial: String) {
        self.editing = true;
        self.edit_buffer = initial;
        self.cursor_pos = self.edit_buffer.len();
    }

    pub fn finish_editing(&mut self) {
        self.editing = false;
        let value = if self.edit_buffer.is_empty() {
            None
        } else {
            Some(self.edit_buffer.clone())
        };

        match self.focus {
            FocusArea::Art if self.index == 1 => {
                self.custom_art = value;
                self.update_preview();
            }
            FocusArea::Art if self.index == 3 => {
                self.image_path = value;
                self.update_preview();
            }
            _ => {}
        }

        self.edit_buffer.clear();
    }
}

// Import preview module for update_preview
use crate::prettyconfig::preview;

impl App {
    pub fn update_preview(&mut self) {
        preview::update_preview(self);
    }
}
