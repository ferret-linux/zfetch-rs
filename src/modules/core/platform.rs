use std::fs;

/// Get the system architecture.
/// Reads from /proc/version at runtime, falls back to compile-time std::env::consts::ARCH.
pub fn platform() -> String {
    // Try to read architecture from /proc/version at runtime
    // Format: "Linux version 6.9.3-arch1 (linux@archlinux) (gcc ...) #1 SMP ..."
    // We want the machine field from uname, which isn't in /proc/version.
    // /proc/sys/kernel/arch is the cleanest single-value source.
    if let Ok(arch) = fs::read_to_string("/proc/sys/kernel/arch") {
        let arch = arch.trim();
        if !arch.is_empty() {
            return arch.to_string();
        }
    }

    // Fallback: parse /proc/cpuinfo for the "Hardware" or check uname via /proc/version
    // as a last runtime attempt before compile-time constant
    if let Ok(content) = fs::read_to_string("/proc/cpuinfo") {
        for line in content.lines() {
            if line.starts_with("Architecture") || line.starts_with("architecture") {
                if let Some(val) = line.splitn(2, ':').nth(1) {
                    let val = val.trim();
                    if !val.is_empty() {
                        return val.to_string();
                    }
                }
            }
        }
    }

    // Compile-time fallback
    std::env::consts::ARCH.to_string()
}