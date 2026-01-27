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

pub fn format_bytes(bytes: u64) -> String {
    if bytes == 0 {
        return "0 B".to_string();
    }
    let k = 1024.0;
    let sizes = ["B", "KB", "MB", "GB", "TB", "PB", "EB", "ZB", "YB"];
    let i = (bytes as f64).log(k).floor() as usize;
    if i >= sizes.len() {
        return format!("{} {}", bytes, sizes[0]);
    }
    format!("{:.2} {}", (bytes as f64) / k.powf(i as f64), sizes[i])
}

pub fn format_duration(seconds: u64) -> String {
    if seconds == 0 {
        return "".to_string();
    }
    if seconds > 3600 {
        format!("{:.1}小时", seconds as f64 / 3600.0)
    } else if seconds > 60 {
        format!("{:.0}分钟", seconds as f64 / 60.0)
    } else {
        format!("{}秒", seconds)
    }
}

pub fn is_valid_url(url: &str) -> bool {
    // Simple basic check for http/https/ftp
    let lower = url.to_lowercase();
    lower.starts_with("http://")
        || lower.starts_with("https://")
        || lower.starts_with("ftp://")
        || lower.starts_with("ftps://")
}

pub fn map_status(aria2_status: &str) -> String {
    match aria2_status {
        "active" => "downloading".to_string(),
        "waiting" => "waiting".to_string(),
        "paused" => "paused".to_string(),
        "complete" => "completed".to_string(),
        "error" => "error".to_string(),
        "removed" | "cancelled" => "cancelled".to_string(),
        _ => "waiting".to_string(),
    }
}

pub fn get_state_score(state: &str) -> i32 {
    match state {
        "downloading" | "waiting" => 2,
        "paused" => 1,
        _ => 0,
    }
}

pub fn deduce_filename(filename: Option<String>, urls: &Vec<String>) -> String {
    if let Some(out) = filename {
        if !out.is_empty() {
            return out;
        }
    }

    // Try to extract from URL
    if let Some(first_url) = urls.get(0) {
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
            options.insert("out".to_string(), serde_json::Value::String(out.clone()));
            out
        } else {
            "".to_string()
        }
    } else {
        "".to_string()
    };

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
        for h in h_str.split(';') {
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
        std::fs::create_dir_all(parent)?;
    }

    // Write content
    std::fs::write(&tmp_path, content)?;

    // 2. Rename (Atomic replace)
    std::fs::rename(&tmp_path, path)?;

    Ok(())
}
