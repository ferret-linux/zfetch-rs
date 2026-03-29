// ASCII art module for zfetch
// Renders colorized ASCII art using placeholder-based colorization

use crate::visuals::colorcontrol::get_art_colors;
use crate::visuals::asciiengine::AsciiArtColorizer;
use std::fs;

// The ASCII art for the zfetch logo Wide version.
const ASCII_ART_WIDE: &str = include_str!("../assets/default/wide.txt");

// The ASCII art for the zfetch logo narrow version.
const ASCII_ART_NARROW: &str = include_str!("../assets/default/narrow.txt");

// OS-specific ASCII art
const ASCII_ART_ARCH: &str = include_str!("../assets/distros/full/arch.txt");
const ASCII_ART_CACHYOS: &str = include_str!("../assets/distros/full/cachy.txt");
const ASCII_ART_FEDORA: &str = include_str!("../assets/distros/full/fedora.txt");
const ASCII_ART_UBUNTU: &str = include_str!("../assets/distros/full/ubuntu.txt");
const ASCII_ART_NIX: &str = include_str!("../assets/distros/full/nix.txt");
const ASCII_ART_GENTOO: &str = include_str!("../assets/distros/full/gentoo.txt");
const ASCII_ART_VOID: &str = include_str!("../assets/distros/full/void.txt");
const ASCII_ART_PIKA: &str = include_str!("../assets/distros/full/pika.txt");
const ASCII_ART_DEBIAN: &str = include_str!("../assets/distros/full/debian.txt");
const ASCII_ART_BAZZITE: &str = include_str!("../assets/distros/full/bazzite.txt");
const ASCII_ART_AURORA: &str = include_str!("../assets/distros/full/aurora.txt");
const ASCII_ART_OMARCHY: &str = include_str!("../assets/distros/full/omarchy.txt");
const ASCII_ART_MINT: &str = include_str!("../assets/distros/full/mint.txt");
const ASCII_ART_NOBARA: &str = include_str!("../assets/distros/full/nobara.txt");
const ASCII_ART_ZODIUM: &str = include_str!("../assets/distros/full/zodium.txt");
const ASCII_ART_ALMA: &str = include_str!("../assets/distros/full/alma.txt");
const ASCII_ART_RHEL: &str = include_str!("../assets/distros/full/rhel.txt");
const ASCII_ART_OPENSUSE: &str = include_str!("../assets/distros/full/opensuse.txt");

// Meme versions
const ASCII_ART_ARCHMEME: &str = include_str!("../assets/distros/meme/arch-meme.txt");
const ASCII_ART_NIXMEME: &str = include_str!("../assets/distros/meme/nix-meme.txt");

// Small versions of OS-specific ASCII art
const ASCII_ART_ARCH_SMALL: &str = include_str!("../assets/distros/small/arch-small.txt");
const ASCII_ART_CACHYOS_SMALL: &str = include_str!("../assets/distros/small/cachy-small.txt");
const ASCII_ART_FEDORA_SMALL: &str = include_str!("../assets/distros/small/fedora-small.txt");
const ASCII_ART_UBUNTU_SMALL: &str = include_str!("../assets/distros/small/ubuntu-small.txt");
const ASCII_ART_NIX_SMALL: &str = include_str!("../assets/distros/small/nix-small.txt");
const ASCII_ART_GENTOO_SMALL: &str = include_str!("../assets/distros/small/gentoo-small.txt");
const ASCII_ART_VOID_SMALL: &str = include_str!("../assets/distros/small/void-small.txt");
const ASCII_ART_PIKA_SMALL: &str = include_str!("../assets/distros/small/pika-small.txt");
const ASCII_ART_DEBIAN_SMALL: &str = include_str!("../assets/distros/small/debian-small.txt");
const ASCII_ART_BAZZITE_SMALL: &str = include_str!("../assets/distros/small/bazzite-small.txt");
const ASCII_ART_AURORA_SMALL: &str = include_str!("../assets/distros/small/aurora-small.txt");
const ASCII_ART_OMARCHY_SMALL: &str = include_str!("../assets/distros/small/omarchy-small.txt");
const ASCII_ART_MINT_SMALL: &str = include_str!("../assets/distros/small/mint-small.txt");
const ASCII_ART_NOBARA_SMALL: &str = include_str!("../assets/distros/small/nobara-small.txt");
const ASCII_ART_ZODIUM_SMALL: &str = include_str!("../assets/distros/small/zodium-small.txt");
const ASCII_ART_ALMA_SMALL: &str = include_str!("../assets/distros/small/alma-small.txt");
const ASCII_ART_RHEL_SMALL: &str = include_str!("../assets/distros/small/rhel-small.txt");
const ASCII_ART_OPENSUSE_SMALL: &str = include_str!("../assets/distros/small/opensuse-small.txt");

// Render the wide ASCII art logo and return lines as a Vec
pub fn get_wide_logo_lines() -> Vec<String> {
    let colors = get_art_colors();
    AsciiArtColorizer::with_colors(ASCII_ART_WIDE, &colors, true).collect()
}

// Render the narrow ASCII art logo and return lines as a Vec
pub fn get_narrow_logo_lines() -> Vec<String> {
    let colors = get_art_colors();
    AsciiArtColorizer::with_colors(ASCII_ART_NARROW, &colors, true).collect()
}

// Get OS-specific art if available, returns None if no match
pub fn get_os_logo_lines(os_name: &str) -> Option<Vec<String>> {
    let os_lower = os_name.to_lowercase();
    // Check for meme versions first (explicit override only)
    let art_str = if os_lower == "archbtw" {
        Some(ASCII_ART_ARCHMEME)
    } else if os_lower == "nixbtw" {
        Some(ASCII_ART_NIXMEME)
    // Regular OS matching
    } else if os_lower.contains("cachyos") || os_lower.contains("cachy") {
        Some(ASCII_ART_CACHYOS)
    } else if os_lower.contains("omarchy") {
        Some(ASCII_ART_OMARCHY)
    } else if os_lower.contains("arch") {
        Some(ASCII_ART_ARCH)
    } else if os_lower.contains("fedora") {
        Some(ASCII_ART_FEDORA)
    } else if os_lower.contains("ubuntu") {
        Some(ASCII_ART_UBUNTU)
    } else if os_lower.contains("nixos") || os_lower.contains("nix") {
        Some(ASCII_ART_NIX)
    } else if os_lower.contains("gentoo") {
        Some(ASCII_ART_GENTOO)
    } else if os_lower.contains("void") {
        Some(ASCII_ART_VOID)
    } else if os_lower.contains("pika") {
        Some(ASCII_ART_PIKA)
    } else if os_lower.contains("debian") {
        Some(ASCII_ART_DEBIAN)
    } else if os_lower.contains("bazzite") {
        Some(ASCII_ART_BAZZITE)
    } else if os_lower.contains("aurora") {
        Some(ASCII_ART_AURORA)
    } else if os_lower.contains("mint") {
        Some(ASCII_ART_MINT)
    } else if os_lower.contains("nobara") {
        Some(ASCII_ART_NOBARA)
    } else if os_lower.contains("zodium") {
        Some(ASCII_ART_ZODIUM)
    } else if os_lower.contains("alma") {
        Some(ASCII_ART_ALMA)
    } else if os_lower.contains("rhel") {
        Some(ASCII_ART_RHEL)
    } else if os_lower.contains("opensuse") {
        Some(ASCII_ART_OPENSUSE)
    } else {
        None
    };

    art_str.map(|s| {
        let colors = get_art_colors();
        AsciiArtColorizer::with_colors(s, &colors, true).collect()
    })
}

// Get small OS-specific art if available, returns None if no match
pub fn get_os_logo_lines_small(os_name: &str) -> Option<Vec<String>> {
    let os_lower = os_name.to_lowercase();
    // Check for special versions first (explicit override only)
    let art_str = if os_lower == "archbtw" {
        None // No small version for meme
    } else if os_lower == "nixbtw" {
        None // No small version for meme
    // Regular OS matching
    } else if os_lower.contains("cachyos") || os_lower.contains("cachy") {
        Some(ASCII_ART_CACHYOS_SMALL)
    } else if os_lower.contains("omarchy") {
        Some(ASCII_ART_OMARCHY_SMALL)
    } else if os_lower.contains("arch") {
        Some(ASCII_ART_ARCH_SMALL)
    } else if os_lower.contains("fedora") {
        Some(ASCII_ART_FEDORA_SMALL)
    } else if os_lower.contains("ubuntu") {
        Some(ASCII_ART_UBUNTU_SMALL)
    } else if os_lower.contains("nixos") || os_lower.contains("nix") {
        Some(ASCII_ART_NIX_SMALL)
    } else if os_lower.contains("gentoo") {
        Some(ASCII_ART_GENTOO_SMALL)
    } else if os_lower.contains("void") {
        Some(ASCII_ART_VOID_SMALL)
    } else if os_lower.contains("pika") {
        Some(ASCII_ART_PIKA_SMALL)
    } else if os_lower.contains("debian") {
        Some(ASCII_ART_DEBIAN_SMALL)
    } else if os_lower.contains("bazzite") {
        Some(ASCII_ART_BAZZITE_SMALL)
    } else if os_lower.contains("aurora") {
        Some(ASCII_ART_AURORA_SMALL)
    } else if os_lower.contains("mint") {
        Some(ASCII_ART_MINT_SMALL)
    } else if os_lower.contains("nobara") {
        Some(ASCII_ART_NOBARA_SMALL)
    } else if os_lower.contains("zodium") {
        Some(ASCII_ART_ZODIUM_SMALL)
    } else if os_lower.contains("alma") {
        Some(ASCII_ART_ALMA_SMALL)
    } else if os_lower.contains("rhel") {
        Some(ASCII_ART_RHEL_SMALL)
    } else if os_lower.contains("opensuse") {
        Some(ASCII_ART_OPENSUSE_SMALL)
    } else {
        None
    };

    art_str.map(|s| {
        let colors = get_art_colors();
        AsciiArtColorizer::with_colors(s, &colors, true).collect()
    })
}

// Load custom ASCII art from a file path
// Returns None if file doesn't exist or can't be read
pub fn get_custom_art_lines(path: &str) -> Option<Vec<String>> {
    let content = fs::read_to_string(path).ok()?;
    let colors = get_art_colors();
    Some(AsciiArtColorizer::with_colors(&content, &colors, true).collect())
}
