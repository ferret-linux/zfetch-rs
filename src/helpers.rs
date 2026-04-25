// Helper functions

use std::collections::HashMap;
use std::fs::{self, File};
use std::io::{BufRead, BufReader};
use std::sync::OnceLock;
use std::sync::atomic::{AtomicU8, Ordering};

use memchr::memchr_iter;

use crate::modules::font::{find_font, is_nerd_font};
use crate::modules::userspace::terminal;

// Cache for font detection - only computed once
static CACHED_FONT: OnceLock<String> = OnceLock::new();
static CACHED_IS_NERD: OnceLock<bool> = OnceLock::new();
static CACHED_TERMINAL: OnceLock<String> = OnceLock::new();

// Global nerd font override: 0 = auto, 1 = force on, 2 = force off
static NERD_FONT_OVERRIDE: AtomicU8 = AtomicU8::new(0);

/// Set the nerd font override from config
/// 0 = auto (detect from font), 1 = force on, 2 = force off
pub fn set_nerd_font_override(mode: u8) {
    NERD_FONT_OVERRIDE.store(mode, Ordering::Relaxed);
}

pub fn get_cached_is_nerd_font() -> bool {
    match NERD_FONT_OVERRIDE.load(Ordering::Relaxed) {
        1 => true,  // Force on
        2 => false, // Force off
        _ => {
            // Check if terminal is Ghostty or WezTerm (they support nerd fonts natively)
            let term = CACHED_TERMINAL.get_or_init(terminal);
            if term.contains("Ghostty") || term.contains("WezTerm") {
                return true;
            }

            // For other terminals, detect from font
            *CACHED_IS_NERD.get_or_init(|| {
                let font = CACHED_FONT.get_or_init(find_font);
                is_nerd_font(font)
            })
        }
    }
}

// Parsed PCI database: vendor_id -> (vendor_name, device_id -> device_name)
pub type PciDatabase = HashMap<String, (String, HashMap<String, String>)>;
static PCI_DB: OnceLock<Option<PciDatabase>> = OnceLock::new();

pub fn get_pci_database() -> &'static Option<PciDatabase> {
    PCI_DB.get_or_init(|| {
        let content = fs::read("/usr/share/hwdata/pci.ids")
            .or_else(|_| fs::read("/usr/share/misc/pci.ids"))
            .ok()?;

        let mut db: PciDatabase = HashMap::new();
        let mut current_vendor_id: Option<String> = None;

        // Use memchr for SIMD-accelerated newline finding
        let mut start = 0;
        for end in memchr_iter(b'\n', &content) {
            let line = &content[start..end];
            start = end + 1;

            // Skip empty lines and comments
            if line.is_empty() || line[0] == b'#' {
                continue;
            }

            // Vendor line: starts with hex digit, no leading tab
            if line[0] != b'\t' && line.len() >= 4 {
                if line[..4].iter().all(|b| b.is_ascii_hexdigit()) {
                    let vendor_id = std::str::from_utf8(&line[..4])
                        .ok()?
                        .to_ascii_lowercase();
                    let vendor_name = std::str::from_utf8(&line[4..])
                        .ok()
                        .map(|s| s.trim().to_string())
                        .unwrap_or_default();
                    db.insert(vendor_id.clone(), (vendor_name, HashMap::new()));
                    current_vendor_id = Some(vendor_id);
                }
            }
            // Device line: starts with single tab (not double tab for subsystem)
            else if line[0] == b'\t' && line.get(1) != Some(&b'\t') && line.len() >= 5 {
                if let Some(ref vendor_id) = current_vendor_id {
                    let trimmed = &line[1..]; // Skip the tab
                    if trimmed[..4].iter().all(|b| b.is_ascii_hexdigit()) {
                        let device_id = std::str::from_utf8(&trimmed[..4])
                            .ok()?
                            .to_ascii_lowercase();
                        let device_name = std::str::from_utf8(&trimmed[4..])
                            .ok()
                            .map(|s| s.trim().to_string())
                            .unwrap_or_default();
                        if let Some((_, devices)) = db.get_mut(vendor_id) {
                            devices.insert(device_id, device_name);
                        }
                    }
                }
            }
        }

        Some(db)
    })
}

// Helper to read the first line of a file using buffered I/O
// Only reads until first newline instead of entire file
pub fn read_first_line(path: impl AsRef<std::path::Path>) -> Option<String> {
    let file = File::open(path).ok()?;
    let mut reader = BufReader::new(file);
    let mut line = String::new();
    reader.read_line(&mut line).ok()?;
    // Trim trailing newline
    if line.ends_with('\n') {
        line.pop();
        if line.ends_with('\r') {
            line.pop();
        }
    }
    Some(line)
}

// Check if the device is a laptop based on chassis type
// 8: Portable, 9: Laptop, 10: Notebook, 11: Hand Held, 12: Docking Station,
// 14: Sub Notebook, 30: Tablet, 31: Convertible, 32: Detachable
pub fn is_laptop() -> bool {
    read_first_line("/sys/class/dmi/id/chassis_type")
        .and_then(|t| t.trim().parse::<u32>().ok())
        .map(|t| matches!(t, 8 | 9 | 10 | 11 | 12 | 14 | 30 | 31 | 32))
        .unwrap_or(false)
}

// Helper to capitalize the first letter of a string.
// No im not importing a crate for this.
pub fn capitalize(s: &str) -> String {
    let mut c = s.chars();
    match c.next() {
        None => String::new(),
        Some(f) => f.to_uppercase().collect::<String>() + c.as_str(),
    }
}

// Draw the bar with nerd font icons
pub fn create_bar_pretty(usage_percent: f64) -> String {
    // Calculate filled blocks, 10 blocks = 100%
    let filled_blocks = ((usage_percent / 10.0).round() as usize).min(10);

    if filled_blocks == 0 {
        // Empty bar = Start empty + 9 empty middle + End
        format!("{}", "".repeat(9))
    } else {
        // Filled/Semi-filled = Start filled + (N-1) filled middle + remaining empty + End
        let filled_middle = filled_blocks - 1;
        let empty_middle = 10 - filled_blocks;
        format!(
            "{}{}",
            "".repeat(filled_middle),
            "".repeat(empty_middle)
        )
    }
}

// Draw the bar with regular characters
pub fn create_bar_ascii(usage_percent: f64) -> String {
    // Calculate filled blocks, 10 blocks = 100%
    let filled_blocks = ((usage_percent / 10.0).round() as usize).min(10);
    let empty_blocks = 10 - filled_blocks;

    format!("{}{}", "🬋".repeat(filled_blocks), "═".repeat(empty_blocks))
}

// Draw the bar, auto-selecting style based on font (cached)
pub fn create_bar(usage_percent: f64) -> String {
    if get_cached_is_nerd_font() {
        create_bar_pretty(usage_percent)
    } else {
        create_bar_ascii(usage_percent)
    }
}