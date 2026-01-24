

pub fn resolve_path(path_str: &str) -> String {
    if path_str.starts_with("~") {
        if let Some(home) = dirs::home_dir() {
            return path_str.replacen("~", home.to_str().unwrap_or(""), 1);
        }
    }
    path_str.to_string()
}
