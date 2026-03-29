use crate::helpers::read_first_line;

const UNKNOWN: &str = "unknown";

// Get the kernel version
pub fn kernel() -> String {
    #[cfg(target_os = "linux")]
    {
        read_first_line("/proc/sys/kernel/osrelease")
            .map(|s| {
                // Strip trailing architecture suffix (e.g. .x86_64, .aarch64, .i686, .armv7l, .ppc64le)
                let s = s.rsplit_once('.')
                    .filter(|(_, arch)| matches!(*arch,
                        "x86_64" | "i686" | "i386" | "aarch64" | "armv7l" | "armv7hl" |
                        "armv6l" | "ppc64le" | "ppc64" | "s390x" | "riscv64"
                    ))
                    .map(|(rest, _)| rest)
                    .unwrap_or(&s);
                // Strip distro/build tag (e.g. .fc43, .el9, .mga9)
                let s = s.rsplit_once('.')
                    .filter(|(_, tag)| {
                        let bytes = tag.as_bytes();
                        bytes.len() >= 2
                            && bytes[..2].iter().all(|b| b.is_ascii_alphabetic())
                            && bytes[2..].iter().all(|b| b.is_ascii_digit())
                    })
                    .map(|(rest, _)| rest)
                    .unwrap_or(s);
                // Strip numeric-only build suffix (e.g. -200 in 6.14.6-200) but preserve named ones (e.g. -cachyos)
                let s = s.rsplit_once('-')
                    .filter(|(_, suffix)| suffix.bytes().all(|b| b.is_ascii_digit()))
                    .map(|(rest, _)| rest)
                    .unwrap_or(s);
                s.to_string()
            })
            .unwrap_or_else(|| UNKNOWN.to_string())
    }

    #[cfg(any(target_os = "macos", target_os = "freebsd", target_os = "openbsd", target_os = "netbsd", target_os = "dragonfly"))]
    {
        use std::ffi::CStr;
        use std::mem::MaybeUninit;
        let mut info: MaybeUninit<libc::utsname> = MaybeUninit::uninit();
        if unsafe { libc::uname(info.as_mut_ptr()) } == 0 {
            let info = unsafe { info.assume_init() };
            let release = unsafe { CStr::from_ptr(info.release.as_ptr()) };
            if let Ok(s) = release.to_str() {
                return s.to_string();
            }
        }
        UNKNOWN.to_string()
    }

    #[cfg(not(any(target_os = "macos", target_os = "linux", target_os = "freebsd", target_os = "openbsd", target_os = "netbsd", target_os = "dragonfly")))]
    {
        "unsupported platform".to_string()
    }
}