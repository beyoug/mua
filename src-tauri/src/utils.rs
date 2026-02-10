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

pub fn format_size(bytes: u64) -> String {
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

pub fn format_speed(bytes_per_sec: u64) -> String {
    if bytes_per_sec == 0 {
        return "0.00|B/s".to_string();
    }
    let k = 1024.0;
    let sizes = ["B", "KB", "MB", "GB", "TB", "PB", "EB", "ZB", "YB"];
    let i = (bytes_per_sec as f64).log(k).floor() as usize;
    let unit = if i < sizes.len() {
        format!("{}/s", sizes[i])
    } else {
        "B/s".to_string()
    };
    format!("{:.2}|{}", (bytes_per_sec as f64) / k.powf(i as f64), unit)
}

pub fn format_duration(seconds: u64) -> String {
    if seconds == 0 {
        return "".to_string();
    }
    // Easter Egg: If > 30 days, just say "Thinking of you" (A long time)
    if seconds > 2592000 {
        return "很久很久".to_string();
    }
    if seconds >= 86400 {
        let days = seconds / 86400;
        let hours = (seconds % 86400) / 3600;
        if hours > 0 {
            format!("{}天{}小时", days, hours)
        } else {
            format!("{}天", days)
        }
    } else if seconds >= 3600 {
        let hours = seconds / 3600;
        let mins = (seconds % 3600) / 60;
        if mins > 0 {
            format!("{}小时{}分钟", hours, mins)
        } else {
            format!("{}小时", hours)
        }
    } else if seconds >= 60 {
        let mins = seconds / 60;
        let secs = seconds % 60;
        if secs > 0 {
            format!("{}分钟{}秒", mins, secs)
        } else {
            format!("{}分钟", mins)
        }
    } else {
        format!("{}秒", seconds)
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

pub fn is_valid_url(url: &str) -> bool {
    // Simple basic check for http/https/ftp
    let lower = url.to_lowercase();
    lower.starts_with("http://")
        || lower.starts_with("https://")
        || lower.starts_with("ftp://")
        || lower.starts_with("ftps://")
}
use crate::core::types::TaskState;

/// 将 Aria2 状态映射为应用状态枚举
pub fn map_status(aria2_status: &str) -> TaskState {
    TaskState::from_aria2_status(aria2_status)
}

/// 获取状态排序分数（用于任务列表排序）
pub fn get_state_score(state: TaskState) -> i32 {
    state.score()
}

/// 判断是否为活跃任务状态（下载中、等待中、已暂停）
pub fn is_active_state(state: TaskState) -> bool {
    state.is_active()
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
            log::error!("Failed to create directory {:?}: {}", parent, e);
            return Err(e);
        }
    }

    // Write content
    if let Err(e) = std::fs::write(&tmp_path, content) {
        log::error!("Failed to write temp file {:?}: {}", tmp_path, e);
        return Err(e);
    }

    // 2. Rename (Atomic replace)
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let _ = std::fs::set_permissions(&tmp_path, std::fs::Permissions::from_mode(0o600));
    }

    if let Err(e) = std::fs::rename(&tmp_path, path) {
        log::error!("Failed to rename {:?} to {:?}: {}", tmp_path, path, e);
        return Err(e);
    }

    Ok(())
}
