//! 配置相关命令
//! 包含应用配置的读写操作

use crate::core::error::AppResult;
use tauri::{AppHandle, Manager};

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

    let _ = crate::aria2::client::change_global_option(serde_json::Value::Object(options)).await;

    Ok(())
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
                let _ = tauri::Emitter::emit(&app, "aria2-stdout", log);
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
