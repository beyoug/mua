use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use std::sync::Mutex;
use tauri::{AppHandle, Manager};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct AppConfig {
    #[serde(rename = "rpcPort")]
    pub rpc_port: u16,
    #[serde(rename = "closeToTray")]
    pub close_to_tray: bool,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self { 
            rpc_port: 6800,
            close_to_tray: true,
        }
    }
}

pub struct ConfigState {
    pub config: Mutex<AppConfig>,
}

pub fn get_config_path(app: &AppHandle) -> Option<PathBuf> {
    app.path().app_config_dir().ok().map(|p| p.join("mua_config.json"))
}

pub fn load_config(app: &AppHandle) -> AppConfig {
    if let Some(path) = get_config_path(app) {
        if path.exists() {
            if let Ok(content) = fs::read_to_string(path) {
                if let Ok(config) = serde_json::from_str(&content) {
                    return config;
                }
            }
        }
    }
    AppConfig::default()
}

pub fn save_config(app: &AppHandle, config: &AppConfig) -> Result<(), String> {
    if let Some(path) = get_config_path(app) {
        if let Some(parent) = path.parent() {
            let _ = fs::create_dir_all(parent);
        }
        let json = serde_json::to_string_pretty(config).map_err(|e| e.to_string())?;
        fs::write(path, json).map_err(|e| e.to_string())?;
        Ok(())
    } else {
        Err("Could not resolve config path".to_string())
    }
}
