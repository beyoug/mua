//! 配置相关命令
//! 包含应用配置的读写操作

use crate::core::error::AppResult;
use crate::core::events::EVENT_ARIA2_STDOUT;
use tauri::{AppHandle, Manager};

const INFINITE_SEED_TIME: &str = "999999999";

#[tauri::command]
pub async fn get_app_config(app: AppHandle) -> AppResult<crate::core::config::AppConfig> {
    Ok(crate::core::config::load_config(&app))
}

#[tauri::command]
pub async fn save_app_config(
    app: AppHandle,
    config: crate::core::config::AppConfig,
) -> AppResult<()> {
    // 1. 保存到磁盘
    crate::core::config::save_config(&app, &config)?;

    // 2. 更新内存中的状态
    if let Some(state) = app.try_state::<crate::core::config::ConfigState>() {
        if let Ok(mut lock) = state.config.lock() {
            *lock = config.clone();
        }
    }

    // 3. 实时同步到正在运行的 Aria2 内核
    let mut options = serde_json::Map::new();
    options.insert(
        "max-concurrent-downloads".to_string(),
        serde_json::Value::String(config.max_concurrent_downloads.to_string()),
    );

    if !config.global_max_download_limit.is_empty() {
        options.insert(
            "max-download-limit".to_string(),
            serde_json::Value::String(config.global_max_download_limit.clone()),
        );
    } else {
        options.insert(
            "max-download-limit".to_string(),
            serde_json::Value::String("0".to_string()),
        );
    }

    if !config.global_max_upload_limit.is_empty() {
        options.insert(
            "max-overall-upload-limit".to_string(),
            serde_json::Value::String(config.global_max_upload_limit.clone()),
        );
    } else {
        options.insert(
            "max-overall-upload-limit".to_string(),
            serde_json::Value::String("0".to_string()),
        );
    }

    // BT 设置
    if !config.bt_trackers.is_empty() {
        let normalized_trackers = crate::utils::normalize_bt_trackers(&config.bt_trackers);
        options.insert(
            "bt-tracker".to_string(),
            serde_json::Value::String(normalized_trackers),
        );
    }

    options.insert(
        "enable-dht".to_string(),
        serde_json::Value::String(config.enable_dht.to_string()),
    );

    options.insert(
        "enable-peer-exchange".to_string(),
        serde_json::Value::String(config.enable_peer_exchange.to_string()),
    );

    options.insert(
        "seed-ratio".to_string(),
        serde_json::Value::String(config.seed_ratio.to_string()),
    );

    if config.enable_seeding {
        options.insert(
            "seed-time".to_string(),
            serde_json::Value::String(INFINITE_SEED_TIME.to_string()),
        );
    } else {
        options.insert(
            "seed-time".to_string(),
            serde_json::Value::String("0".to_string()),
        );
    }

    options.insert(
        "dht-listen-port".to_string(),
        serde_json::Value::String(config.dht_listen_port.clone()),
    );

    options.insert(
        "listen-port".to_string(),
        serde_json::Value::String(config.listen_port.clone()),
    );

    if let Err(e) = crate::aria2::client::change_global_option(serde_json::Value::Object(options)).await {
        crate::app_warn!(
            "Core::Config",
            "runtime_apply_failed",
            serde_json::json!({ "error": e.to_string() })
        );
    }

    Ok(())
}

#[tauri::command]
pub async fn fetch_public_trackers() -> AppResult<Vec<String>> {
    let url = "https://trackerslist.com/best.txt";
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(10))
        .build()
        .map_err(|e| crate::core::error::AppError::Config(e.to_string()))?;

    let response = client
        .get(url)
        .send()
        .await
        .map_err(|e| crate::core::error::AppError::Config(e.to_string()))?;

    let text = response
        .text()
        .await
        .map_err(|e| crate::core::error::AppError::Config(e.to_string()))?;

    let trackers: Vec<String> = text
        .lines()
        .map(|line| line.trim())
        .filter(|line: &&str| !line.is_empty())
        .map(|line: &str| line.to_string())
        .collect();

    Ok(trackers)
}

#[tauri::command]
pub fn start_log_stream(app: AppHandle) {
    if let Some(state) = app.try_state::<crate::aria2::sidecar::LogStreamEnabled>() {
        state.0.store(true, std::sync::atomic::Ordering::Relaxed);
    }

    // 立即发送缓冲的日志
    if let Some(state) = app.try_state::<crate::aria2::sidecar::SidecarState>() {
        if let Ok(logs) = state.recent_logs.lock() {
            for log in logs.iter() {
                let _ = tauri::Emitter::emit(&app, EVENT_ARIA2_STDOUT, log);
            }
        }
    }
}

#[tauri::command]
pub fn stop_log_stream(app: AppHandle) {
    if let Some(state) = app.try_state::<crate::aria2::sidecar::LogStreamEnabled>() {
        state.0.store(false, std::sync::atomic::Ordering::Relaxed);
    }
}

#[derive(serde::Serialize)]
pub struct PlatformInfo {
    pub os: String,
    pub arch: String,
}

#[tauri::command]
pub fn get_platform_info() -> PlatformInfo {
    PlatformInfo {
        os: std::env::consts::OS.to_string(),
        arch: std::env::consts::ARCH.to_string(),
    }
}
