// Section loading logic for zfetch

use crate::configloader::Config;
use crate::modules;
use crate::visuals::renderer::Section;
use std::thread;

// Load all sections based on config toggles.
// Spawns threads for slow I/O operations (GPU, storage, packages, shell, font, screen)
// and runs fast operations synchronously (OS, kernel, uptime, CPU, memory, etc.).
// Returns a tuple of (core, hardware, userspace) sections.
pub fn load_sections(config: &Config) -> (Section, Section, Section) {
    // Spawn threads for slow I/O operations (subprocesses) if enabled.
    // These may run external commands like vulkaninfo, df, shell --version, etc.
    let gpu_handler = if config.hardware.gpu {
        Some(thread::spawn(modules::hardware::gpu))
    } else { None };
    let storage_handler = if config.hardware.storage {
        Some(thread::spawn(modules::hardware::storage))
    } else { None };
    let packages_handler = if config.userspace.packages {
        Some(thread::spawn(modules::userspace::packages))
    } else { None };
    let shell_handler = if config.userspace.shell {
        Some(thread::spawn(modules::userspace::shell))
    } else { None };
    let font_handler = if config.userspace.terminal_font {
        Some(thread::spawn(modules::font::find_font))
    } else { None };
    let screen_handler = if config.hardware.screen {
        Some(thread::spawn(modules::hardware::screen))
    } else { None };

    // Fast operations - just file reads or env var checks, no benefit from threading.
    let os = if config.core.os { Some(modules::core::os()) } else { None };
    let kernel = if config.core.kernel { Some(modules::core::kernel()) } else { None };
    let platform = if config.core.platform { Some(modules::core::platform()) } else { None };
    let uptime = if config.core.uptime { Some(modules::core::uptime()) } else { None };
    let init = if config.core.init { Some(modules::core::init()) } else { None };
    let os_age = if config.core.os_age { Some(modules::core::os_age()) } else { None };
    let cpu = if config.hardware.cpu { Some(modules::hardware::cpu()) } else { None };
    let memory = if config.hardware.memory { Some(modules::hardware::memory()) } else { None };
    let battery = if config.hardware.battery { Some(modules::hardware::laptop_battery()) } else { None };
    let terminal = if config.userspace.terminal { Some(modules::userspace::terminal()) } else { None };
    let wm = if config.userspace.wm { Some(modules::userspace::wm()) } else { None };
    let ui = if config.userspace.ui { Some(modules::userspace::ui()) } else { None };
    let editor = if config.userspace.editor { Some(modules::userspace::editor()) } else { None };

    // Build core section - OS info, kernel version, system uptime, init system, OS age.
    let mut core_lines = Vec::new();
    if let Some(v) = os { core_lines.push(("OS".to_string(), v)); }
    if let Some(v) = kernel { core_lines.push(("Kernel".to_string(), v)); }
    if let Some(v) = platform { core_lines.push(("Platform".to_string(), v)); }
    if let Some(v) = uptime { core_lines.push(("Uptime".to_string(), v)); }
    if let Some(v) = init { core_lines.push(("Init".to_string(), v)); }
    if let Some(v) = os_age {
        if v != "unknown" {
            core_lines.push(("OS Age".to_string(), v));
        }
    }
    let core = Section::new("Core", core_lines);

    // Build hardware section - CPU, GPU, memory, storage, battery, displays.
    let mut hardware_lines = Vec::new();
    if let Some(v) = cpu { hardware_lines.push(("CPU".to_string(), v)); }
    if let Some(h) = gpu_handler {
        // Join the GPU thread and get the result.
        let gpu_info = h.join().unwrap_or_else(|_| modules::hardware::GpuInfo::default());
        // Format GPU display based on configuration
        let gpu_entries = modules::hardware::format_gpu_display(&gpu_info, config.hardware.gpu_display);
        hardware_lines.extend(gpu_entries);
    }
    if let Some(v) = memory { hardware_lines.push(("Memory".to_string(), v)); }
    if let Some(h) = storage_handler {
        // Join the storage thread and get the result.
        hardware_lines.push(("Storage".to_string(), h.join().unwrap_or_else(|_| String::from("error"))));
    }
    if let Some(v) = battery {
        // Only show battery if it's actually detected (not "unknown").
        if v != "unknown" {
            hardware_lines.push(("Battery".to_string(), v));
        }
    }
    if let Some(h) = screen_handler {
        // Screen returns multiple entries (one per display).
        let screen_entries: Vec<(String, String)> = h.join().unwrap_or_else(|_| vec![]);
        hardware_lines.extend(screen_entries);
    }
    let hardware = Section::new("Hardware", hardware_lines);

    // Build userspace section - packages, terminal, shell, WM, UI, editor, font.
    let mut userspace_lines = Vec::new();
    if let Some(h) = packages_handler {
        userspace_lines.push(("Packages".to_string(), h.join().unwrap_or_else(|_| "error".into())));
    }
    if let Some(v) = terminal { userspace_lines.push(("Terminal".to_string(), v)); }
    if let Some(h) = shell_handler {
        userspace_lines.push(("Shell".to_string(), h.join().unwrap_or_else(|_| "error".into())));
    }
    if let Some(v) = wm { userspace_lines.push(("WM".to_string(), v)); }
    if let Some(v) = ui { userspace_lines.push(("UI".to_string(), v)); }
    if let Some(v) = editor {
        // Only show editor if one was detected.
        if !v.is_empty() {
            userspace_lines.push(("Editor".to_string(), v));
        }
    }
    if let Some(h) = font_handler {
        userspace_lines.push(("Terminal Font".to_string(), h.join().unwrap_or_else(|_| "error".into())));
    }
    let userspace = Section::new("Userspace", userspace_lines);

    (core, hardware, userspace)
}
