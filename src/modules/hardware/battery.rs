use std::fs;

use crate::helpers::{create_bar, get_cached_is_nerd_font, is_laptop, read_first_line};

// Get battery status if device is a laptop (chassis check)
pub fn laptop_battery() -> String {
    if !is_laptop() {
        return "unknown".to_string();
    }

    // Find first available battery (usually BAT0 or BAT1)
    let power_supply = std::path::Path::new("/sys/class/power_supply");
    if let Ok(entries) = fs::read_dir(power_supply) {
        for entry in entries.flatten() {
            let name = entry.file_name();
            let name_str = name.to_string_lossy();

            if name_str.starts_with("BAT") {
                let path = entry.path();

                // Get capacity
                let capacity = read_first_line(path.join("capacity").to_str().unwrap_or(""))
                    .and_then(|c| c.parse::<u8>().ok())
                    .unwrap_or(0);

                // Get status
                let status = read_first_line(path.join("status").to_str().unwrap_or(""))
                    .unwrap_or_else(|| "Unknown".to_string());

                let nerd = get_cached_is_nerd_font();
                let status_icon: &str = match status.as_str() {
                    "Charging" => if nerd { "󰂄" } else { "(+)" },
                    "Discharging" => if nerd { "󱟤" } else { "(-)" },
                    "Full" => if nerd { "󰁹" } else { "(=)" },
                    "Not charging" | "Not Charging" => if nerd { "" } else { "(=)" },
                    _ => &status,
                };

                let bar = create_bar(capacity as f64);

                return format!("{} {}% {}", bar, capacity, status_icon);
            }
        }
    }

    "unknown".to_string()
}