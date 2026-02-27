use serde::{Deserialize, Serialize};
use serde_json::json;
use std::fs;
use std::path::PathBuf;
use std::sync::Mutex;
use tauri::{AppHandle, Manager};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct AppConfig {
    /// 配置文件版本号，用于未来迁移
    #[serde(default = "default_config_version")]
    pub version: u32,
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
    #[serde(rename = "maxConcurrentDownloads", default = "default_max_downloads")]
    pub max_concurrent_downloads: u32,
    #[serde(rename = "uaHistory", default = "default_ua_history")]
    pub ua_history: Vec<String>,
    #[serde(rename = "defaultSavePath", default = "default_save_path")]
    pub default_save_path: String,
    #[serde(rename = "globalMaxDownloadLimit", default = "default_string_empty")]
    pub global_max_download_limit: String,
    #[serde(rename = "globalMaxUploadLimit", default = "default_string_empty")]
    pub global_max_upload_limit: String,
    #[serde(default = "default_theme")]
    pub theme: String,
    #[serde(rename = "colorMode", default = "default_color_mode")]
    pub color_mode: String,
    #[serde(rename = "particlesEnabled", default = "default_result_true")]
    pub particles_enabled: bool,
    #[serde(rename = "startMinimized", default = "default_result_false")]
    pub start_minimized: bool,
    #[serde(rename = "btTrackers", default = "default_string_empty")]
    pub bt_trackers: String,
    #[serde(rename = "enableDht", default = "default_result_true")]
    pub enable_dht: bool,
    #[serde(rename = "enablePeerExchange", default = "default_result_true")]
    pub enable_peer_exchange: bool,
    #[serde(rename = "enableSeeding", default = "default_result_true")]
    pub enable_seeding: bool,
    #[serde(rename = "seedRatio", default = "default_seed_ratio")]
    pub seed_ratio: f64,
    #[serde(rename = "dhtListenPort", default = "default_bt_port")]
    pub dht_listen_port: String,
    #[serde(rename = "listenPort", default = "default_bt_port")]
    pub listen_port: String,
}

/// 当前配置版本号
const CURRENT_CONFIG_VERSION: u32 = 1;

fn default_config_version() -> u32 {
    CURRENT_CONFIG_VERSION
}

fn default_seed_ratio() -> f64 {
    1.0
}

fn default_bt_port() -> String {
    "6881".to_string()
}

fn default_ua_history() -> Vec<String> {
    Vec::new()
}

fn default_max_downloads() -> u32 {
    3
}

fn default_session_interval() -> u64 {
    30
}

fn default_result_false() -> bool {
    false
}

fn default_result_true() -> bool {
    true
}

fn default_save_path() -> String {
    dirs::download_dir()
        .map(|p| p.to_string_lossy().to_string())
        .unwrap_or_else(|| "~/Downloads".to_string())
}

fn default_string_empty() -> String {
    String::new()
}

fn default_theme() -> String {
    "default".to_string()
}

fn default_color_mode() -> String {
    "dark".to_string()
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            version: CURRENT_CONFIG_VERSION,
            rpc_port: 6800,
            close_to_tray: true,
            auto_resume: false,
            rpc_secret: None,
            save_session_interval: 30,
            use_custom_aria2: false,
            auto_start: false,
            max_concurrent_downloads: 3,
            ua_history: Vec::new(),
            default_save_path: default_save_path(),
            global_max_download_limit: String::new(),
            global_max_upload_limit: String::new(),
            theme: default_theme(),
            color_mode: default_color_mode(),
            particles_enabled: true,
            start_minimized: false,
            bt_trackers: String::new(),
            enable_dht: true,
            enable_peer_exchange: true,
            enable_seeding: true,
            seed_ratio: 1.0,
            dht_listen_port: default_bt_port(),
            listen_port: default_bt_port(),
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
        crate::app_info!(
            "Core::Config",
            "config_path_resolved",
            json!({ "path": path.to_string_lossy() })
        );
        if path.exists() {
            if let Ok(content) = fs::read_to_string(&path) {
                if let Ok(mut config) = serde_json::from_str::<AppConfig>(&content) {
                    crate::app_info!(
                        "Core::Config",
                        "loaded_from_disk",
                        json!({ "start_minimized": config.start_minimized })
                    );
                    // Ensure secret exists
                    if config.rpc_secret.is_none() {
                        let secret = uuid::Uuid::new_v4().to_string();
                        config.rpc_secret = Some(secret);
                        // Save back
                        let _ = save_config(app, &config);
                    }
                    return config;
                } else {
                    crate::app_error!(
                        "Core::Config",
                        "deserialize_failed",
                        json!({ "raw": content })
                    );
                }
            } else {
                crate::app_error!(
                    "Core::Config",
                    "read_failed",
                    json!({ "path": path.to_string_lossy() })
                );
            }
        } else {
            crate::app_warn!(
                "Core::Config",
                "config_file_missing",
                json!({ "path": path.to_string_lossy() })
            );
        }
    } else {
        crate::app_error!("Core::Config", "config_path_unresolved");
    }

    crate::app_info!("Core::Config", "fallback_default_config");
    // Default with new secret
    let mut config = AppConfig::default();
    config.rpc_secret = Some(uuid::Uuid::new_v4().to_string());
    // Try save immediately to persist the generated secret
    let _ = save_config(app, &config);
    config
}

pub fn save_config(app: &AppHandle, config: &AppConfig) -> crate::core::error::AppResult<()> {
    if let Some(path) = get_config_path(app) {
        let json = serde_json::to_string_pretty(config)?;
        crate::utils::atomic_write(&path, &json)?;
        Ok(())
    } else {
        Err(crate::core::error::AppError::config(
            "无法解析配置文件路径".to_string(),
        ))
    }
}
