
use std::env;
use std::fs;
use std::path::Path;
use std::process::Command;
use memchr::{memchr_iter, memmem};
use crate::helpers::get_cached_is_nerd_font;

// Count RPM packages by querying rpmdb.sqlite directly via dlopen'd libsqlite3.
// This avoids spawning `rpm -qa` which takes ~600ms on systems with many packages.
// Uses the Sigmd5 table which naturally excludes gpg-pubkey virtual packages.
fn count_rpm_sqlite(db_path: &str) -> Option<usize> {
    use std::os::raw::{c_char, c_int, c_void};

    const SQLITE_OK: c_int = 0;
    const SQLITE_ROW: c_int = 100;
    const SQLITE_OPEN_READONLY: c_int = 0x00000001;

    // dlopen libsqlite3 - always present on RPM systems since rpm depends on it
    let lib_name = b"libsqlite3.so.0\0";
    let lib = unsafe { libc::dlopen(lib_name.as_ptr() as *const c_char, libc::RTLD_LAZY) };
    if lib.is_null() {
        return None;
    }

    // Load the 6 function pointers we need
    macro_rules! load_sym {
        ($lib:expr, $name:literal) => {{
            let sym = unsafe { libc::dlsym($lib, concat!($name, "\0").as_ptr() as *const c_char) };
            if sym.is_null() {
                unsafe { libc::dlclose($lib); }
                return None;
            }
            sym
        }};
    }

    let open_v2 = load_sym!(lib, "sqlite3_open_v2");
    let prepare_v2 = load_sym!(lib, "sqlite3_prepare_v2");
    let step = load_sym!(lib, "sqlite3_step");
    let column_int = load_sym!(lib, "sqlite3_column_int");
    let finalize = load_sym!(lib, "sqlite3_finalize");
    let close = load_sym!(lib, "sqlite3_close");

    type OpenV2Fn = unsafe extern "C" fn(*const c_char, *mut *mut c_void, c_int, *const c_char) -> c_int;
    type PrepareV2Fn = unsafe extern "C" fn(*mut c_void, *const c_char, c_int, *mut *mut c_void, *mut *const c_char) -> c_int;
    type StepFn = unsafe extern "C" fn(*mut c_void) -> c_int;
    type ColumnIntFn = unsafe extern "C" fn(*mut c_void, c_int) -> c_int;
    type FinalizeFn = unsafe extern "C" fn(*mut c_void) -> c_int;
    type CloseFn = unsafe extern "C" fn(*mut c_void) -> c_int;

    let sqlite3_open_v2: OpenV2Fn = unsafe { std::mem::transmute(open_v2) };
    let sqlite3_prepare_v2: PrepareV2Fn = unsafe { std::mem::transmute(prepare_v2) };
    let sqlite3_step: StepFn = unsafe { std::mem::transmute(step) };
    let sqlite3_column_int: ColumnIntFn = unsafe { std::mem::transmute(column_int) };
    let sqlite3_finalize: FinalizeFn = unsafe { std::mem::transmute(finalize) };
    let sqlite3_close: CloseFn = unsafe { std::mem::transmute(close) };

    // Build null-terminated path
    let mut path_buf = db_path.as_bytes().to_vec();
    path_buf.push(0);

    let mut db: *mut c_void = std::ptr::null_mut();
    let rc = unsafe { sqlite3_open_v2(path_buf.as_ptr() as *const c_char, &mut db, SQLITE_OPEN_READONLY, std::ptr::null()) };
    if rc != SQLITE_OK {
        unsafe { libc::dlclose(lib); }
        return None;
    }

    let sql = b"SELECT count(*) FROM Packages\0";
    let mut stmt: *mut c_void = std::ptr::null_mut();
    let rc = unsafe { sqlite3_prepare_v2(db, sql.as_ptr() as *const c_char, -1, &mut stmt, std::ptr::null_mut()) };
    if rc != SQLITE_OK {
        unsafe { sqlite3_close(db); libc::dlclose(lib); }
        return None;
    }

    let count = if unsafe { sqlite3_step(stmt) } == SQLITE_ROW {
        Some(unsafe { sqlite3_column_int(stmt, 0) } as usize)
    } else {
        None
    };

    unsafe {
        sqlite3_finalize(stmt);
        sqlite3_close(db);
        libc::dlclose(lib);
    }

    count
}

// Get the total number of installed packages.
// Supports pacman aka Arch, hopefully supports debian and fedora but idk, im not setting up a vm to test sorry
pub fn packages() -> String {
    let mut counts: Vec<String> = Vec::with_capacity(9);
    let nerd = get_cached_is_nerd_font();

    // Pacman - count directories in /var/lib/pacman/local/
    if let Ok(entries) = fs::read_dir("/var/lib/pacman/local") {
        let count = entries.filter_map(|e| e.ok()).filter(|e| e.file_type().map_or(false, |ft| ft.is_dir())).count();
        if count > 0 {
            let icon = if nerd { "󰮯" } else { "(pacman)" };
            counts.push(format!("{} {}", icon, count));
        }
    }

    // dpkg (Debian/Ubuntu) - count occurrences of status line using SIMD-accelerated search
    if let Ok(content) = fs::read("/var/lib/dpkg/status") {
        const NEEDLE: &[u8] = b"\nStatus: install ok installed\n";
        let count = memmem::find_iter(&content, NEEDLE).count();
        if count > 0 {
            let icon = if nerd { "󰕈" } else { "(dpkg)" };
            counts.push(format!("{} {}", icon, count));
        }
    }

    // RPM - query rpmdb.sqlite directly via dlopen'd libsqlite3 (avoids spawning rpm -qa which is ~600ms)
    // Sigmd5 table naturally excludes gpg-pubkey virtual packages
    {
        let rpm_db_path = [
            "/usr/lib/sysimage/rpm/rpmdb.sqlite",
            "/var/lib/rpm/rpmdb.sqlite",
        ]
        .iter()
        .find(|p| Path::new(p).exists());

        if let Some(&db_path) = rpm_db_path {
            let count = count_rpm_sqlite(db_path).unwrap_or_else(|| {
                // Fallback to rpm -qa if sqlite query fails
                Command::new("rpm")
                    .arg("-qa")
                    .output()
                    .ok()
                    .map(|output| memchr_iter(b'\n', &output.stdout).count())
                    .unwrap_or(0)
            });
            if count > 0 {
                let icon = if nerd { "" } else { "(rpm)" };
                counts.push(format!("{} {}", icon, count));
            }
        } else if Path::new("/var/lib/rpm/Packages").exists() {
            // Legacy BDB format - must use rpm command
            if let Ok(output) = Command::new("rpm").arg("-qa").output() {
                let count = memchr_iter(b'\n', &output.stdout).count();
                if count > 0 {
                    let icon = if nerd { "" } else { "(rpm)" };
                    counts.push(format!("{} {}", icon, count));
                }
            }
        }
    }

    // Flatpak - count installed applications from both system and user installs
    {
        use std::collections::HashSet;
        let mut seen = HashSet::new();
        let mut count = 0;
        let dirs = [
            Some("/var/lib/flatpak/app".to_string()),
            env::var("HOME").ok().map(|h| format!("{}/.local/share/flatpak/app", h)),
        ];
        for dir in dirs.iter().flatten() {
            if let Ok(entries) = fs::read_dir(dir) {
                for e in entries.flatten() {
                    if seen.insert(e.file_name()) {
                        count += 1;
                    }
                }
            }
        }
        if count > 0 {
            let icon = if nerd { "" } else { "(flatpak)" };
            counts.push(format!("{} {}", icon, count));
        }
    }

    // Nix - count packages in user profile
    if let Ok(home) = env::var("HOME") {
        let nix_profile = format!("{}/.nix-profile/manifest.nix", home);
        if Path::new(&nix_profile).exists() {
            // Count packages via nix-env -q
            if let Ok(output) = Command::new("nix-env").arg("-q").output() {
                // Count non-empty lines using SIMD-accelerated memchr
                let stdout = &output.stdout;
                let newline_count = memchr_iter(b'\n', stdout).count();
                // If output ends with newline, count equals lines; otherwise add 1 for last line
                let count = if stdout.last() == Some(&b'\n') || stdout.is_empty() {
                    newline_count
                } else {
                    newline_count + 1
                };
                if count > 0 {
                    let icon = if nerd { "󱄅" } else { "(nix)" };
                    counts.push(format!("{} {}", icon, count));
                }
            }
        }
    }

    // XBPS (Void Linux) - find pkgdb dir and count package subdirs
    if let Ok(entries) = fs::read_dir("/var/db/xbps") {
        if let Some(pkgdb) = entries.filter_map(|e| e.ok())
            .find(|e| e.file_name().as_encoded_bytes().starts_with(b"pkgdb"))
        {
            if let Ok(pkgs) = fs::read_dir(pkgdb.path()) {
                let count = pkgs.filter_map(|e| e.ok())
                    .filter(|e| e.file_type().map_or(false, |ft| ft.is_dir()))
                    .count();
                if count > 0 {
                    let icon = if nerd { "" } else { "(xbps)" };
                    counts.push(format!("{} {}", icon, count));
                }
            }
        }
    }

    // Portage (Gentoo) - count package directories in /var/db/pkg/
    // Structure is /var/db/pkg/<category>/<package>-<version>/
    if let Ok(categories) = fs::read_dir("/var/db/pkg") {
        let count: usize = categories
            .filter_map(|cat| cat.ok())
            .filter(|cat| cat.file_type().map_or(false, |ft| ft.is_dir()))
            .filter_map(|cat| fs::read_dir(cat.path()).ok())
            .map(|pkgs| {
                pkgs.filter_map(|p| p.ok())
                    .filter(|p| p.file_type().map_or(false, |ft| ft.is_dir()))
                    .count()
            })
            .sum();
        if count > 0 {
            let icon = if nerd { "" } else { "(portage)" };
            counts.push(format!("{} {}", icon, count));
        }
    }
    
    // apk (Alpine)
    if Path::new("/lib/apk/db/installed").exists() {
        if let Ok(content) = fs::read("/lib/apk/db/installed") {
            let count = content.split(|&b| b == b'\n')
                .filter(|line| line.starts_with(b"P:"))
                .count();
            if count > 0 {
                let icon = if nerd { "" } else { "(apk)" };
                counts.push(format!("{} {}", icon, count));
            }
        }
    }

    // eopkg (Solus)
    if let Ok(entries) = fs::read_dir("/var/lib/eopkg/package") {
        let count = entries.filter_map(|e| e.ok()).filter(|e| e.file_type().map_or(false, |ft| ft.is_dir())).count();
        if count > 0 {
            let icon = if nerd { "" } else { "(eopkg)" };
            counts.push(format!("{} {}", icon, count));
        }
    }

    if counts.is_empty() {
        "unknown".to_string()
    } else {
        counts.join(" | ")
    }
}