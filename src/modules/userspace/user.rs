use std::env;
use std::fs;

// Get the current user's full name (from /etc/passwd GECOS field) or username fallback.
pub fn user() -> String {
    let username = env::var("USER")
        .or_else(|_| env::var("LOGNAME"))
        .unwrap_or_else(|_| whoami_from_proc());

    if username.is_empty() || username == "unknown" {
        return "unknown".to_string();
    }

    if let Some(full_name) = get_full_name_from_passwd(&username) {
        return full_name;
    }

    username
}

fn get_full_name_from_passwd(username: &str) -> Option<String> {
    let passwd = fs::read_to_string("/etc/passwd").ok()?;

    for line in passwd.lines() {
        let mut fields = line.splitn(7, ':');
        let name = fields.next()?;
        if name != username { continue; }
        fields.next(); fields.next(); fields.next(); // skip password, uid, gid

        let gecos = fields.next().unwrap_or("").trim();
        if gecos.is_empty() { return None; }

        let full_name = gecos.split(',').next().unwrap_or("").trim();
        if full_name.is_empty() { return None; }

        return Some(full_name.to_string());
    }

    None
}

fn whoami_from_proc() -> String {
    if let Ok(status) = fs::read_to_string("/proc/self/status") {
        for line in status.lines() {
            if line.starts_with("Uid:") {
                let uid_str = line.split_whitespace().nth(1).unwrap_or("");
                if let Ok(uid) = uid_str.parse::<u32>() {
                    if let Ok(passwd) = fs::read_to_string("/etc/passwd") {
                        for entry in passwd.lines() {
                            let mut parts = entry.splitn(7, ':');
                            let uname = parts.next().unwrap_or("");
                            parts.next();
                            let uid_field = parts.next().unwrap_or("");
                            if uid_field.parse::<u32>().ok() == Some(uid) {
                                return uname.to_string();
                            }
                        }
                    }
                }
                break;
            }
        }
    }
    "unknown".to_string()
}