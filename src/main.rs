//zfetch by Zodium Project

mod cache;
mod configloader;
mod dostuff;
mod helpers;
mod modules;
mod prettyconfig;
mod themes;
mod visuals;

use clap::{Parser,ArgAction};
use configloader::OsArtSetting;

// cmd line args, *claps*
#[derive(Parser)]
#[command(name = "zfetch",
          about = "A beautiful system info fetcher",
          version,
          disable_version_flag = true)]
struct Args {
    // Display OS-specific art. Optionally specify OS name (example: --os arch)
    #[arg(short = 'o', long = "os", num_args = 0..=1, default_missing_value = "", value_name = "")]
    os_art: Option<String>,

    // Force refresh of cached values (OS name and GPU)
    #[arg(short = 'r', long = "refresh")]
    refresh: bool,

    // Display image instead of ASCII art (uses Kitty graphics protocol)
    #[arg(short = 'i', long = "image", num_args = 0..=1, default_missing_value = "", value_name = "")]
    image: Option<String>,

    // Launch TUI configuration editor
    #[arg(short = 'c', long = "config")]
    config: bool,

    // Update config file to latest version (preserves user settings)
    #[arg(short = 'u', long = "update")]
    update: bool,

    // Prints Installed Version
    #[arg(short = 'v', long = "version", action = ArgAction::Version)]
    version: Option<bool>,
}

fn main() {
    let args = Args::parse();

    // Launch TUI config editor if --config/-c was passed
    if args.config {
        if let Err(e) = prettyconfig::run() {
            eprintln!("Error running config editor: {}", e);
            std::process::exit(1);
        }
        return;
    }

    // Update config file if --update/-u was passed
    if args.update {
        configloader::migrate_config();
        return;
    }

    // Set cache refresh flag if --refresh/-r was passed
    if args.refresh {
        cache::set_force_refresh(true);
    }

    // Load config first and initialize colors/styles before spawning threads
    let config = configloader::load_config();
    visuals::colorcontrol::init_colors(config.colors.clone());
    visuals::renderer::init_box_styles(config.box_style, config.border_line_style);

    // Set nerd font override from config
    helpers::set_nerd_font_override(match config.nerd_fonts {
        configloader::NerdFontSetting::Auto => 0,
        configloader::NerdFontSetting::ForceOn => 1,
        configloader::NerdFontSetting::ForceOff => 2,
    });

    // Load all sections based on config
    let (core, hardware, userspace) = dostuff::load_sections(&config);

    // Extract OS name before filtering (needed for OS art detection)
    let os_name: String = core
        .lines
        .iter()
        .find(|(k, _)| k == "OS")
        .map(|(_, v)| v.clone())
        .unwrap_or_default();

    // Filter out empty sections
    let sections: Vec<_> = [core, hardware, userspace]
        .into_iter()
        .filter(|s| !s.lines.is_empty())
        .collect();

    // Load ASCII art synchronously - just reading static data
    let wide_logo = modules::ascii::get_wide_logo_lines();
    let narrow_logo = modules::ascii::get_narrow_logo_lines();

    // Check if image mode is requested (CLI arg or config) AND terminal supports it
    let use_image = args.image.is_some() || config.image;

    if use_image {
        // Determine image path:
        // 1. CLI arg with explicit path takes highest priority
        // 2. CLI arg empty (-i/--image) uses config.image_path if set, else embedded default
        // 3. Config image=true uses config.image_path if set, else embedded default
        let image_path: Option<std::path::PathBuf> = if let Some(ref image_arg) = args.image {
            if image_arg.is_empty() {
                // CLI flag without path - use config image_path if available
                config.image_path.as_ref().map(std::path::PathBuf::from)
            } else if image_arg.starts_with("~/") {
                // CLI flag with explicit path (expand ~)
                Some(if let Some(home) = std::env::var_os("HOME") {
                    std::path::PathBuf::from(home).join(&image_arg[2..])
                } else {
                    std::path::PathBuf::from(image_arg)
                })
            } else {
                // CLI flag with explicit path
                Some(std::path::PathBuf::from(image_arg))
            }
        } else {
            // Config image=true, use config image_path if set
            config.image_path.as_ref().map(std::path::PathBuf::from)
        };

        // Draw image layout (imagerender handles all the logic)
        visuals::imagerender::draw_image_layout(&sections, image_path.as_deref(), matches!(config.art_position, configloader::ArtPosition::Right));
    } else {
        // Standard ASCII art mode
        // Check for custom art first (overrides everything else)
        let (wide, narrow, small) = if let Some(ref custom_path) = config.custom_art {
            if let Some(custom_art) = modules::ascii::get_custom_art_lines(custom_path) {
                (custom_art.clone(), custom_art, None)
            } else {
                // Custom art file not found, fall back to default
                (wide_logo.clone(), narrow_logo.clone(), None)
            }
        } else {
            // Determine OS art setting: CLI args override config
            let os_art_setting = if let Some(ref os_override) = args.os_art {
                if os_override.is_empty() {
                    OsArtSetting::Auto
                } else {
                    OsArtSetting::Specific(os_override.clone())
                }
            } else {
                config.os_art.clone()
            };

            // Apply OS art setting
            match os_art_setting {
                OsArtSetting::Disabled => (wide_logo, narrow_logo, None),
                OsArtSetting::Auto => {
                    if let Some(os_logo) = modules::ascii::get_os_logo_lines(&os_name) {
                        let small_logo = modules::ascii::get_os_logo_lines_small(&os_name);
                        (os_logo.clone(), os_logo, small_logo)
                    } else {
                        (wide_logo, narrow_logo, None)
                    }
                }
                OsArtSetting::Specific(ref specific_os) => {
                    if let Some(os_logo) = modules::ascii::get_os_logo_lines(specific_os) {
                        let small_logo = modules::ascii::get_os_logo_lines_small(specific_os);
                        (os_logo.clone(), os_logo, small_logo)
                    } else {
                        (wide_logo, narrow_logo, None)
                    }
                }
            }
        };

        print!(
            "{}",
            visuals::renderer::draw_layout(&wide, &narrow, &sections, small.as_deref(), matches!(config.art_position, configloader::ArtPosition::Right))
        );
    }
}
