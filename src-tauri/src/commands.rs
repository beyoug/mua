use crate::aria2_client::{self, Aria2Task};
use crate::utils;
use std::collections::HashMap;
use tauri::{AppHandle, Manager};

use crate::store::{PersistedTask, TaskStore};
use chrono::Local;

#[tauri::command]
#[allow(clippy::too_many_arguments)]
pub async fn add_download_task(
    state: tauri::State<'_, TaskStore>,
    urls: Vec<String>,
    save_path: Option<String>,
    filename: Option<String>,
    user_agent: Option<String>,
    referer: Option<String>,
    headers: Option<String>,
    proxy: Option<String>,
    max_download_limit: Option<String>,
) -> Result<String, String> {
    log::info!("add_download_task called with urls: {:?}", urls);
    let mut options = HashMap::new();

    let save_path_str = if let Some(dir) = &save_path {
        let p = utils::resolve_path(dir);
        options.insert("dir".to_string(), p.clone());
        p
    } else {
        "".to_string()
    };

    let filename_str = if let Some(out) = &filename {
        if !out.is_empty() {
            options.insert("out".to_string(), out.clone());
        }
        out.clone()
    } else {
        "".to_string()
    };

    // 构建 header 选项
    let mut header_list = Vec::new();
    if let Some(ua) = &user_agent {
        if !ua.is_empty() {
            options.insert("user-agent".to_string(), ua.clone());
        }
    }
    if let Some(ref_url) = &referer {
        if !ref_url.is_empty() {
            options.insert("referer".to_string(), ref_url.clone());
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

    // Call Aria2
    match aria2_client::add_uri(urls.clone(), Some(options)).await {
        Ok(gid) => {
            // Persist to Store
            let task = PersistedTask {
                gid: gid.clone(),
                filename: filename_str,
                url: urls.get(0).cloned().unwrap_or_default(),
                save_path: save_path_str,
                added_at: Local::now().to_rfc3339(),
                state: "waiting".to_string(), // Initial state
                total_length: "0".to_string(),
                completed_length: "0".to_string(),
                download_speed: "0".to_string(),
            };
            state.add_task(task);
            Ok(gid)
        }
        Err(e) => Err(e),
    }
}

#[tauri::command]
pub async fn pause_task(state: tauri::State<'_, TaskStore>, gid: String) -> Result<String, String> {
    state.update_task_state(&gid, "paused");
    aria2_client::pause(gid).await
}

#[tauri::command]
pub async fn resume_task(
    state: tauri::State<'_, TaskStore>,
    gid: String,
) -> Result<String, String> {
    state.update_task_state(&gid, "waiting");
    aria2_client::resume(gid).await
}

#[tauri::command]
pub async fn cancel_task(
    state: tauri::State<'_, TaskStore>,
    gid: String,
) -> Result<String, String> {
    state.update_task_state(&gid, "cancelled");
    aria2_client::remove(gid).await
}

#[tauri::command]
pub async fn remove_task_record(
    state: tauri::State<'_, TaskStore>,
    gid: String,
    delete_file: bool,
    filepath: Option<String>,
) -> Result<String, String> {
    // 0. Remove from Store
    state.remove_task(&gid);

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

            // 3. 尝试清理 .aria2 控制文件 (Aria2 遗留文件)
            let aria2_file_path = format!("{}.aria2", resolved_path);
            if std::path::Path::new(&aria2_file_path).exists() {
                match std::fs::remove_file(&aria2_file_path) {
                    Ok(_) => log::info!("Deleted aria2 control file: {}", aria2_file_path),
                    // 这是一个“尽力而为”的操作，如果失败通常不严重，但记录警告
                    Err(e) => log::warn!("Failed to delete aria2 file {}: {}", aria2_file_path, e),
                }
            }
        } else {
            log::warn!(
                "Delete file requested but no filepath provided for gid: {}",
                gid
            );
        }
    }
    Ok("OK".to_string())
}

#[tauri::command]
pub async fn get_tasks(
    state: tauri::State<'_, TaskStore>,
    _app_handle: AppHandle,
) -> Result<Vec<Aria2Task>, String> {
    // We still return Aria2Task to frontend for compat
    // 1. Get all tasks from Store
    let mut store_tasks = state.get_all();

    // 2. Fetch from Aria2
    // 2. Fetch from Aria2
    let all_tasks = aria2_client::get_all_tasks().await.unwrap_or_default();

    // 3. Create a Map of GID -> Aria2Task for easy lookup
    let mut aria2_map: HashMap<String, Aria2Task> = HashMap::new();
    for t in all_tasks {
        aria2_map.insert(t.gid.clone(), t);
    }

    // 4. Sync Logic
    // Iterate over STORE tasks (Truth) and update them with Aria2 data

    // Also handle "Auto Resume" logic if this is first run?
    // Ideally syncing happens just by updating state.

    for task in store_tasks.iter_mut() {
        if let Some(aria_task) = aria2_map.get(&task.gid) {
            // Task exists in Aria2 -> Update Store
            task.state = aria_task.status.clone();
            task.total_length = aria_task.total_length.clone();
            task.completed_length = aria_task.completed_length.clone();
            task.download_speed = aria_task.download_speed.clone();

            // Update persistent store
            state.update_from_aria2(
                &task.gid,
                &aria_task.status,
                &aria_task.completed_length,
                &aria_task.download_speed,
                &aria_task.total_length,
            );
        } else {
            // Task missing from Aria2 lists (active/waiting/stopped)
            // It might be transitioning or legitimately lost.

            // Skip verification if we already know it's in a terminal state (Error/Removed)
            // or if we already marked it as completed but it's gone from Aria2 (which is fine).
            if task.state == "error" || task.state == "removed" || task.state == "completed" {
                continue;
            }

            // Verify explicitly.
            match aria2_client::tell_status(
                task.gid.clone(),
                vec![
                    "gid",
                    "status",
                    "totalLength",
                    "completedLength",
                    "downloadSpeed",
                    "dir",
                    "files",
                ],
            )
            .await
            {
                Ok(aria_task) => {
                    // Task found! Sync it.
                    // This handles the race condition where task is transitioning (e.g. adding -> waiting)
                    // and hasn't appeared in the list snapshot yet but exists.
                    task.state = aria_task.status.clone();
                    task.total_length = aria_task.total_length.clone();
                    task.completed_length = aria_task.completed_length.clone();
                    task.download_speed = aria_task.download_speed.clone();

                    state.update_from_aria2(
                        &task.gid,
                        &aria_task.status,
                        &aria_task.completed_length,
                        &aria_task.download_speed,
                        &aria_task.total_length,
                    );
                }
                Err(e) => {
                    // Task truly lost (404) or Aria2 error
                    log::warn!(
                        "Task {} verification failed: {}. Marking as error.",
                        task.gid,
                        e
                    );

                    // Mark as error to stop infinite verification loop
                    task.state = "error".to_string();
                    task.download_speed = "0".to_string();
                    state.update_task_state(&task.gid, "error");
                }
            }
        }
    }

    // 5. Convert PersistedTask back to Aria2Task for Frontend Compatibility
    let mut result: Vec<Aria2Task> = Vec::new();
    for t in store_tasks {
        // Find saved file path for 'files'
        let mut files = Vec::new();
        // Construct pseudo-file struct
        // Frontend expects: files: [{ path: string, uris: [{uri: string}] }]

        // This is a bit hacky to mock Aria2Task structure from PersistedTask
        // But keeps frontend changes minimal.
        let file_path = if !t.save_path.is_empty() && !t.filename.is_empty() {
            if t.save_path.ends_with('/') {
                format!("{}{}", t.save_path, t.filename)
            } else {
                format!("{}/{}", t.save_path, t.filename)
            }
        } else {
            t.filename.clone()
        };

        files.push(crate::aria2_client::Aria2File {
            index: "1".to_string(),
            path: file_path,
            length: t.total_length.clone(),
            completed_length: t.completed_length.clone(),
            selected: "true".to_string(),
            uris: vec![crate::aria2_client::Aria2Uri {
                uri: t.url.clone(),
                status: "used".to_string(),
            }],
        });

        result.push(Aria2Task {
            gid: t.gid,
            status: t.state,
            total_length: t.total_length,
            completed_length: t.completed_length,
            download_speed: t.download_speed,
            upload_length: "0".to_string(), // Default
            upload_speed: "0".to_string(),  // Default
            error_code: None,
            error_message: None,
            dir: t.save_path,
            files: files,
        });
    }

    Ok(result)
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
pub async fn save_app_config(
    app: AppHandle,
    config: crate::config::AppConfig,
) -> Result<(), String> {
    // 1. 保存到磁盘
    crate::config::save_config(&app, &config)?;

    // 2. 我们可能想要重启 aria2 或者只是让用户重启应用。
    // 动态重启 sidecar 比较复杂。提示重启更好。
    Ok(())
}
