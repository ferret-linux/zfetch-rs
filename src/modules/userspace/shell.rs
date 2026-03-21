use std::env;
use std::process::Command;
use memchr;
use crate::helpers::capitalize;

/// Get the active shell with version.
pub fn shell() -> String {
    let shell_path = match env::var("SHELL") {
        Ok(p) => p,
        Err(_) => return "unknown".to_string(),
    };

    let shell_name = match shell_path.rsplit('/').next() {
        Some(name) if !name.is_empty() => name,
        _ => return "unknown".to_string(),
    };

    // Try to get version by running shell --version
    let version = Command::new(&shell_path)
        .arg("--version")
        .output()
        .ok()
        .and_then(|output| {
            // Find first line directly in bytes using memchr
            let stdout = &output.stdout;
            let first_line_end = memchr::memchr(b'\n', stdout).unwrap_or(stdout.len());
            let first_line = std::str::from_utf8(&stdout[..first_line_end]).ok()?;

            // Extract version number (e.g., "5.2.26" from "bash 5.2.26(1)-release")
            first_line
                .split_ascii_whitespace()
                .find(|word| word.as_bytes().first().map_or(false, |b| b.is_ascii_digit()))
                .map(|v| {
                    // Clean up version string - find first ( or - using memchr
                    let v_bytes = v.as_bytes();
                    let paren_pos = memchr::memchr(b'(', v_bytes);
                    let dash_pos = memchr::memchr(b'-', v_bytes);
                    let end = match (paren_pos, dash_pos) {
                        (Some(p), Some(d)) => p.min(d),
                        (Some(p), None) => p,
                        (None, Some(d)) => d,
                        (None, None) => v.len(),
                    };
                    v[..end].to_string()
                })
        });

    match version {
        Some(v) => format!("{} {}", capitalize(shell_name), v),
        None => capitalize(shell_name),
    }
}