use std::fs;

use memchr::memchr_iter;
use memchr::memmem;

use crate::helpers::create_bar;

// Get storage usage for all physical disks using statvfs syscall.
// Reads /proc/mounts and uses statvfs for each real filesystem - much faster than spawning df
pub fn storage() -> String {
    let mut total_bytes: u64 = 0;
    let mut used_bytes: u64 = 0;
    let mut seen_devices = std::collections::HashSet::new();

    // Read /proc/mounts as bytes for SIMD-accelerated parsing
    if let Ok(content) = fs::read("/proc/mounts") {
        let mut start = 0;
        for end in memchr_iter(b'\n', &content) {
            let line = &content[start..end];
            start = end + 1;

            // Find first space (device ends here)
            let Some(space1) = memchr::memchr(b' ', line) else {
                continue;
            };
            let device = &line[..space1];

            // Find second space (mount point ends here)
            let rest = &line[space1 + 1..];
            let Some(space2) = memchr::memchr(b' ', rest) else {
                continue;
            };
            let mount_point_bytes = &rest[..space2];

            // Filter for real disks: starts with /dev/ and not loop devices
            if device.len() < 5
                || &device[..5] != b"/dev/"
                || memmem::find(device, b"/loop").is_some()
            {
                continue;
            }

            let Ok(device_str) = std::str::from_utf8(device) else {
                continue;
            };
            let Ok(mount_point) = std::str::from_utf8(mount_point_bytes) else {
                continue;
            };

            // Avoid double counting if device mounted multiple times
            if !seen_devices.insert(device_str.to_string()) {
                continue;
            }

            // Use statvfs syscall to get filesystem stats
            if let Some((total, used)) = get_fs_stats(mount_point) {
                total_bytes += total;
                used_bytes += used;
            }
        }
    }

    if total_bytes > 0 {
        let usage_percent = (used_bytes as f64 / total_bytes as f64) * 100.0;
        let bar = create_bar(usage_percent);

        // Convert to GB (decimal: 1 GB = 1,000,000,000 bytes)
        let used_gb = used_bytes as f64 / 1_000_000_000.0;
        let total_gb = total_bytes as f64 / 1_000_000_000.0;

        // Use TB for total if >= 1000GB, frees up horizontal line space
        if total_gb >= 1000.0 {
            let total_tb = total_gb / 1000.0;
            // Trim .00 if it's a whole number (e.g., 1.00TB -> 1TB)
            let total_str = if (total_tb - total_tb.floor()).abs() < 0.05 {
                format!("{}TB", total_tb.floor() as u64)
            } else {
                format!("{:.1}TB", total_tb)
            };
            let used_str = if used_gb >= 900.0 {
                let used_tb = used_gb / 1000.0;
                if (used_tb - used_tb.floor()).abs() < 0.05 {
                    format!("{}TB", used_tb.floor() as u64)
                } else {
                    format!("{:.1}TB", used_tb)
                }
            } else {
                format!("{:.0}GB", used_gb)
            };
            return format!("{} {}/{}", bar, used_str, total_str);
        }

        let used_str = if used_gb >= 900.0 {
            let used_tb = used_gb / 1000.0;
            if (used_tb - used_tb.floor()).abs() < 0.05 {
                format!("{}TB", used_tb.floor() as u64)
            } else {
                format!("{:.1}TB", used_tb)
            }
        } else {
            format!("{:.0}GB", used_gb)
        };
        return format!("{} {}/{:.0}GB", bar, used_str, total_gb);
    }
    "unknown".to_string()
}

// Get filesystem stats using statvfs syscall
// Returns (total_bytes, used_bytes) or None on failure
fn get_fs_stats(path: &str) -> Option<(u64, u64)> {
    use std::ffi::CString;
    use std::mem::MaybeUninit;

    let c_path = CString::new(path).ok()?;
    let mut stat: MaybeUninit<libc::statvfs> = MaybeUninit::uninit();

    // SAFETY: statvfs is a standard POSIX syscall, c_path is valid null-terminated string
    let result = unsafe { libc::statvfs(c_path.as_ptr(), stat.as_mut_ptr()) };

    if result != 0 {
        return None;
    }

    // SAFETY: statvfs succeeded, stat is now initialized
    let stat = unsafe { stat.assume_init() };

    let block_size = stat.f_frsize as u64;
    let total_blocks = stat.f_blocks as u64;
    let free_blocks = stat.f_bfree as u64;

    let total = total_blocks * block_size;
    let used = (total_blocks - free_blocks) * block_size;

    Some((total, used))
}