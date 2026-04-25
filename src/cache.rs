// Persistent cache for slow-to-fetch OS/GPU values.

use std::fs;
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};

// Global flag to force cache refresh
static FORCE_REFRESH: AtomicBool = AtomicBool::new(false);

pub fn set_force_refresh(value: bool) {
    FORCE_REFRESH.store(value, Ordering::Relaxed);
}

pub fn should_refresh() -> bool {
    FORCE_REFRESH.load(Ordering::Relaxed)
}

fn get_cache_dir() -> Option<PathBuf> {
    let home = std::env::var("HOME").ok()?;
    let cache_dir = PathBuf::from(home).join(".cache").join("zfetch");

    // Create cache directory if it doesn't exist
    if !cache_dir.exists() {
        fs::create_dir_all(&cache_dir).ok()?;
    }

    Some(cache_dir)
}

fn get_cache_path(key: &str) -> Option<PathBuf> {
    Some(get_cache_dir()?.join(key))
}

// Cache TTL: 7 days in seconds
const CACHE_TTL_SECS: u64 = 7 * 24 * 60 * 60;

// Read a cached value. Returns None if cache doesn't exist, is older than TTL, or refresh is forced.
pub fn read_cache(key: &str) -> Option<String> {
    if should_refresh() {
        return None;
    }

    let path = get_cache_path(key)?;

    // Check cache age — invalidate if older than TTL
    if let Ok(metadata) = fs::metadata(&path) {
        if let Ok(modified) = metadata.modified() {
            if let Ok(age) = modified.elapsed() {
                if age.as_secs() > CACHE_TTL_SECS {
                    let _ = fs::remove_file(&path);
                    return None;
                }
            }
        }
    }

    fs::read_to_string(path).ok()
}

// Write a value to cache. 10,000IQ
pub fn write_cache(key: &str, value: &str) -> Option<()> {
    let path = get_cache_path(key)?;
    fs::write(path, value).ok()
}

// Read cached GPU info in new format (igpu=/dgpu=)
// Returns None if cache doesn't exist, needs refresh, or is in old format (triggers re-detection)
pub fn get_cached_gpu_info() -> Option<crate::modules::hardware::GpuInfo> {
    use crate::modules::hardware::GpuInfo;

    if should_refresh() {
        return None;
    }

    let content = read_cache("gpu")?;

    // Check if this is the new format (contains "igpu=" or "dgpu=")
    if !content.contains("igpu=") && !content.contains("dgpu=") {
        // Old format - return None to trigger re-detection
        return None;
    }

    // Parse new format
    let mut info = GpuInfo::default();
    for line in content.lines() {
        if let Some(value) = line.strip_prefix("igpu=") {
            if !value.is_empty() {
                info.integrated = Some(value.to_string());
            }
        } else if let Some(value) = line.strip_prefix("dgpu=") {
            if !value.is_empty() {
                info.discrete = Some(value.to_string());
            }
        }
    }

    // Return None if no GPUs were parsed (malformed cache)
    if info.is_empty() {
        None
    } else {
        Some(info)
    }
}

// Cache GPU info in new format (igpu=/dgpu=)
pub fn cache_gpu_info(info: &crate::modules::hardware::GpuInfo) {
    let mut content = String::new();

    if let Some(ref igpu) = info.integrated {
        content.push_str("igpu=");
        content.push_str(igpu);
        content.push('\n');
    }

    if let Some(ref dgpu) = info.discrete {
        content.push_str("dgpu=");
        content.push_str(dgpu);
        content.push('\n');
    }

    let _ = write_cache("gpu", &content);
}

// Read cached OS value, or return None to trigger a fresh fetch.
pub fn get_cached_os() -> Option<String> {
    read_cache("os")
}

// Cache the OS value (arch btw)
pub fn cache_os(value: &str) {
    let _ = write_cache("os", value);
}

// Read cached CPU value, or return None to trigger a fresh fetch.
pub fn get_cached_cpu() -> Option<String> {
    read_cache("cpu")
}

// Cache the CPU value
pub fn cache_cpu(value: &str) {
    let _ = write_cache("cpu", value);
}

// Read cached init system value, or return None to trigger a fresh fetch.
pub fn get_cached_init() -> Option<String> {
    read_cache("init")
}

// Cache the init system value
pub fn cache_init(value: &str) {
    let _ = write_cache("init", value);
}

// Read cached OS birth timestamp (Unix epoch seconds), or return None to trigger a fresh fetch.
pub fn get_cached_os_birth() -> Option<u64> {
    read_cache("os_birth")?.parse().ok()
}

// Cache the OS birth timestamp (Unix epoch seconds)
pub fn cache_os_birth(timestamp: u64) {
    let _ = write_cache("os_birth", &timestamp.to_string());
}
