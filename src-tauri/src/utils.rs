pub fn resolve_path(path_str: &str) -> String {
    if path_str.starts_with("~") {
        if let Some(home) = dirs::home_dir() {
            return path_str.replacen("~", home.to_str().unwrap_or(""), 1);
        }
    }
    path_str.to_string()
}

pub fn show_in_file_manager(path: &str) {
    let resolved = resolve_path(path);

    #[cfg(target_os = "macos")]
    {
        let _ = std::process::Command::new("open")
            .arg("-R")
            .arg(&resolved)
            .spawn();
    }

    #[cfg(target_os = "windows")]
    {
        let _ = std::process::Command::new("explorer")
            .arg("/select,")
            .arg(&resolved)
            .spawn();
    }

    #[cfg(target_os = "linux")]
    {
        // Linux is tricky, standardizing on xdg-open for parent dir
        if let Some(parent) = std::path::Path::new(&resolved).parent() {
            let _ = std::process::Command::new("xdg-open").arg(parent).spawn();
        }
    }
}

pub fn get_full_path(save_path: &str, filename: &str) -> String {
    if save_path.is_empty() {
        return resolve_path(filename);
    }
    let p = std::path::Path::new(save_path);
    let resolved = resolve_path(&p.join(filename).to_string_lossy());
    resolved
}

pub fn is_safe_filename(filename: &str) -> bool {
    let path = std::path::Path::new(filename);
    if path.is_absolute() {
        return false;
    }

    let mut has_normal_component = false;
    for comp in path.components() {
        match comp {
            std::path::Component::Normal(_) => has_normal_component = true,
            _ => return false,
        }
    }

    has_normal_component
}

pub fn canonicalize_abs(path_str: &str) -> std::io::Result<std::path::PathBuf> {
    let resolved = resolve_path(path_str);
    std::fs::canonicalize(resolved)
}

pub fn is_valid_url(url: &str) -> bool {
    // Simple basic check for http/https/ftp
    let lower = url.to_lowercase();
    lower.starts_with("http://")
        || lower.starts_with("https://")
        || lower.starts_with("ftp://")
        || lower.starts_with("ftps://")
        || lower.starts_with("magnet:")
}

pub fn normalize_bt_trackers(trackers: &str) -> String {
    let mut normalized: Vec<String> = Vec::new();

    for tracker in trackers
        .split(|c| c == '\n' || c == '\r' || c == ',' || c == ';')
        .map(|s| s.trim())
        .filter(|s| !s.is_empty())
    {
        let value = tracker.to_string();
        if !normalized.contains(&value) {
            normalized.push(value);
        }
    }

    normalized.join(",")
}

pub fn deduce_filename(filename: Option<String>, urls: &[String]) -> String {
    if let Some(out) = filename {
        if !out.is_empty() {
            return out;
        }
    }

    // Try to extract from URL
    if let Some(first_url) = urls.get(0) {
        if first_url.starts_with("magnet:") {
            // 解析 dn 参数
            if let Some(start) = first_url.find("dn=") {
                let rest = &first_url[start + 3..];
                let end = rest.find('&').unwrap_or(rest.len());
                let dn = &rest[..end];
                if let Ok(decoded) = urlencoding::decode(dn) {
                    return decoded.to_string();
                }
            }
            return "magnet_download".to_string();
        }

        // Simple heuristic: take last part of path
        if let Some(name) = first_url.split('/').last() {
            let clean_name = name.split('?').next().unwrap_or(name);
            if !clean_name.is_empty() {
                return clean_name.to_string();
            }
        }
    }

    "Unknown".to_string()
}

pub fn get_unique_filename(save_path: &str, filename: &str, reserved_names: &[String]) -> String {
    let path = std::path::Path::new(save_path);
    let mut name = filename.to_string();
    let mut counter = 1;

    // Split extension
    let (stem, ext) = if let Some(idx) = filename.rfind('.') {
        // Handle no extension
        if idx == 0 {
            (filename, "")
        } else {
            (&filename[..idx], &filename[idx..])
        }
    } else {
        (filename, "")
    };

    loop {
        let full_path = path.join(&name);

        // Check 1: File system existence
        let exists_on_disk = full_path.exists();

        // Check 2: Active tasks collision
        let exists_in_store = reserved_names.contains(&name);

        if !exists_on_disk && !exists_in_store {
            break;
        }

        // Collision! Try next number: "file (1).ext"
        name = format!("{} ({}){}", stem, counter, ext);
        counter += 1;

        // Loop safety break (unlikely to hit this)
        if counter > 10000 {
            break;
        }
    }

    name
}

pub fn build_aria2_options(
    save_path: Option<String>,
    filename: Option<String>,
    user_agent: Option<String>,
    referer: Option<String>,
    headers: Option<String>,
    proxy: Option<String>,
    max_download_limit: Option<String>,
) -> (serde_json::Value, String) {
    let mut options = serde_json::Map::new();

    let save_path_str = if let Some(dir) = save_path {
        let p = resolve_path(&dir);
        options.insert("dir".to_string(), serde_json::Value::String(p.clone()));
        p
    } else {
        "".to_string()
    };

    if let Some(out) = filename {
        if !out.is_empty() {
            options.insert("out".to_string(), serde_json::Value::String(out));
        }
    }

    // Construct headers
    let mut header_list = Vec::new();
    if let Some(ua) = user_agent {
        if !ua.is_empty() {
            options.insert("user-agent".to_string(), serde_json::Value::String(ua));
        }
    }
    if let Some(ref_url) = referer {
        if !ref_url.is_empty() {
            options.insert("referer".to_string(), serde_json::Value::String(ref_url));
        }
    }

    // Custom Headers and Cookie
    if let Some(h_str) = headers {
        // 支持分号或换行符分隔
        for h in h_str.split(|c| c == ';' || c == '\n') {
            let trim_h = h.trim();
            if !trim_h.is_empty() {
                header_list.push(serde_json::Value::String(trim_h.to_string()));
            }
        }
    }

    if !header_list.is_empty() {
        options.insert("header".to_string(), serde_json::Value::Array(header_list));
    }

    if let Some(p) = proxy {
        if !p.is_empty() {
            options.insert("all-proxy".to_string(), serde_json::Value::String(p));
        }
    }

    if let Some(limit) = max_download_limit {
        if !limit.is_empty() {
            options.insert(
                "max-download-limit".to_string(),
                serde_json::Value::String(limit),
            );
        }
    }

    (serde_json::Value::Object(options), save_path_str)
}

pub fn atomic_write(path: &std::path::Path, content: &str) -> std::io::Result<()> {
    // 1. Write to temp file
    // We use .tmp suffix
    let mut tmp_path = path.to_path_buf();
    if let Some(ext) = tmp_path.extension() {
        let mut new_ext = ext.to_os_string();
        new_ext.push(".tmp");
        tmp_path.set_extension(new_ext);
    } else {
        tmp_path.set_extension("tmp");
    }

    // Ensure parent dir exists
    if let Some(parent) = path.parent() {
        if let Err(e) = std::fs::create_dir_all(parent) {
            crate::app_error!(
                "Core::Utils",
                "atomic_write_create_dir_failed",
                serde_json::json!({
                    "parent": parent.to_string_lossy(),
                    "error": e.to_string()
                })
            );
            return Err(e);
        }
    }

    // Write content
    if let Err(e) = std::fs::write(&tmp_path, content) {
        crate::app_error!(
            "Core::Utils",
            "atomic_write_temp_write_failed",
            serde_json::json!({
                "temp_path": tmp_path.to_string_lossy(),
                "error": e.to_string()
            })
        );
        return Err(e);
    }

    // 2. Rename (Atomic replace)
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let _ = std::fs::set_permissions(&tmp_path, std::fs::Permissions::from_mode(0o600));
    }

    if let Err(e) = std::fs::rename(&tmp_path, path) {
        crate::app_error!(
            "Core::Utils",
            "atomic_write_rename_failed",
            serde_json::json!({
                "temp_path": tmp_path.to_string_lossy(),
                "target_path": path.to_string_lossy(),
                "error": e.to_string()
            })
        );
        return Err(e);
    }

    Ok(())
}
