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
    #[serde(rename = "autoResume")]
    pub auto_resume: bool,
    #[serde(rename = "rpcSecret")]
    pub rpc_secret: Option<String>,
    #[serde(
        rename = "aria2SaveSessionInterval",
        default = "default_session_interval"
    )]
    pub save_session_interval: u64,
    #[serde(rename = "useCustomAria2", default = "default_result_false")]
    pub use_custom_aria2: bool,
    #[serde(rename = "autoStart", default = "default_result_false")]
    pub auto_start: bool,
}

fn default_session_interval() -> u64 {
    30
}

fn default_result_false() -> bool {
    false
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            rpc_port: 6800,
            close_to_tray: true,
            auto_resume: false,
            rpc_secret: None,
            save_session_interval: 30,
            use_custom_aria2: false,
            auto_start: false,
        }
    }
}

pub struct ConfigState {
    pub config: Mutex<AppConfig>,
}

pub fn get_config_path(app: &AppHandle) -> Option<PathBuf> {
    app.path()
        .app_config_dir()
        .ok()
        .map(|p| p.join("mua_config.json"))
}

pub fn load_config(app: &AppHandle) -> AppConfig {
    if let Some(path) = get_config_path(app) {
        if path.exists() {
            if let Ok(content) = fs::read_to_string(&path) {
                if let Ok(mut config) = serde_json::from_str::<AppConfig>(&content) {
                    // Ensure secret exists
                    if config.rpc_secret.is_none() {
                        let secret = uuid::Uuid::new_v4().to_string();
                        config.rpc_secret = Some(secret);
                        // Save back
                        let _ = save_config(app, &config);
                    }
                    return config;
                }
            }
        }
    }

    // Default with new secret
    let mut config = AppConfig::default();
    config.rpc_secret = Some(uuid::Uuid::new_v4().to_string());
    // Try save immediately to persist the generated secret
    let _ = save_config(app, &config);
    config
}

pub fn save_config(app: &AppHandle, config: &AppConfig) -> Result<(), String> {
    if let Some(path) = get_config_path(app) {
        let json = serde_json::to_string_pretty(config).map_err(|e| e.to_string())?;
        crate::utils::atomic_write(&path, &json).map_err(|e| e.to_string())?;
        Ok(())
    } else {
        Err("Could not resolve config path".to_string())
    }
}
