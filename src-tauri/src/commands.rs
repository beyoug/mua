use crate::aria2_client::{self, Aria2Task};
use crate::utils;
use std::collections::HashMap;
use tauri::{AppHandle, Manager};

#[tauri::command]
#[allow(clippy::too_many_arguments)]
pub async fn add_download_task(
    urls: Vec<String>,
    save_path: Option<String>,
    filename: Option<String>,
    user_agent: Option<String>,
    referer: Option<String>,
    headers: Option<String>,
    proxy: Option<String>,
    max_download_limit: Option<String>
) -> Result<String, String> {
    log::info!("add_download_task called with urls: {:?}", urls);
    let mut options = HashMap::new();
    
    if let Some(dir) = save_path {
        options.insert("dir".to_string(), utils::resolve_path(&dir));
    }
    if let Some(out) = filename {
        if !out.is_empty() {
             options.insert("out".to_string(), out);
        }
    }
    
    // 构建 header 选项
    let mut header_list = Vec::new();
    if let Some(ua) = user_agent {
        if !ua.is_empty() {
            options.insert("user-agent".to_string(), ua);
        }
    }
    if let Some(ref_url) = referer {
        if !ref_url.is_empty() {
             options.insert("referer".to_string(), ref_url);
        }
    }
    // 处理自定义 headers
    if let Some(h_str) = headers {
        for h in h_str.split(';') {
             if !h.trim().is_empty() {
                 header_list.push(h.trim().to_string());
             }
        }
    }

    if let Some(p) = proxy {
         if !p.is_empty() {
             options.insert("all-proxy".to_string(), p);
         }
    }
    
    if let Some(limit) = max_download_limit {
         if !limit.is_empty() {
             options.insert("max-download-limit".to_string(), limit);
         }
    }

    aria2_client::add_uri(urls, Some(options)).await
}

#[tauri::command]
pub async fn pause_task(gid: String) -> Result<String, String> {
    aria2_client::pause(gid).await
}

#[tauri::command]
pub async fn resume_task(gid: String) -> Result<String, String> {
    aria2_client::resume(gid).await
}

#[tauri::command]
pub async fn cancel_task(gid: String) -> Result<String, String> {
    aria2_client::remove(gid).await
}

#[tauri::command]
pub async fn remove_task_record(gid: String, delete_file: bool, filepath: Option<String>) -> Result<String, String> {
    // 1. 从 Aria2 内存中移除
    let _ = aria2_client::purge(gid.clone()).await; 
    
    // 2. 如果请求删除文件，则执行删除
    if delete_file {
        if let Some(path) = filepath {
            let resolved_path = utils::resolve_path(&path);
            
            if std::path::Path::new(&resolved_path).exists() {
                 match std::fs::remove_file(&resolved_path) {
                    Ok(_) => log::info!("Deleted file: {}", resolved_path),
                    Err(e) => log::error!("Failed to delete file {}: {}", resolved_path, e),
                 }
            } else {
                log::warn!("File not found for deletion: {}", resolved_path);
            }
        } else {
            log::warn!("Delete file requested but no filepath provided for gid: {}", gid);
        }
    }
    Ok("OK".to_string())
}

#[tauri::command]
pub async fn get_tasks() -> Result<Vec<Aria2Task>, String> {
    let tasks = aria2_client::get_all_tasks().await;
    if let Ok(ref t) = tasks {
        log::debug!("get_tasks returning {} tasks", t.len());
    }
    tasks
}

#[tauri::command]
pub async fn get_aria2_config_path(app: AppHandle) -> Result<String, String> {
    let config_dir = app.path().app_config_dir().map_err(|e| e.to_string())?;
    let path = config_dir.join("aria2.conf");
    Ok(path.to_string_lossy().to_string())
}

#[tauri::command]
pub async fn read_aria2_config(app: AppHandle) -> Result<String, String> {
    let config_dir = app.path().app_config_dir().map_err(|e| e.to_string())?;
    let path = config_dir.join("aria2.conf");
    
    if path.exists() {
        std::fs::read_to_string(path).map_err(|e| e.to_string())
    } else {
        Ok("".to_string())
    }
}

#[tauri::command]
pub async fn import_aria2_config(app: AppHandle, path: String) -> Result<String, String> {
    let config_dir = app.path().app_config_dir().map_err(|e| e.to_string())?;
    
    if !config_dir.exists() {
        std::fs::create_dir_all(&config_dir).map_err(|e| e.to_string())?;
    }
    
    let dest_path = config_dir.join("aria2.conf");
    std::fs::copy(&path, &dest_path).map_err(|e| e.to_string())?;
    
    Ok("Imported".to_string())
}

#[tauri::command]
pub async fn get_app_config(app: AppHandle) -> Result<crate::config::AppConfig, String> {
    Ok(crate::config::load_config(&app))
}

#[tauri::command]
pub async fn save_app_config(app: AppHandle, config: crate::config::AppConfig) -> Result<(), String> {
    // 1. 保存到磁盘
    crate::config::save_config(&app, &config)?;
    
    // 2. 我们可能想要重启 aria2 或者只是让用户重启应用。
    // 动态重启 sidecar 比较复杂。提示重启更好。
    Ok(())
}
