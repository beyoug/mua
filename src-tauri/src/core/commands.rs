use crate::aria2::client as aria2_client;
use crate::utils;
use futures::future::join_all;
use tauri::{AppHandle, Emitter, Manager};

use crate::core::store::{PersistedTask, TaskStore};
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

    // Validation
    for url in &urls {
        if !utils::is_valid_url(url) {
            return Err(format!("Invalid URL: {}", url));
        }
    }

    // 1. Build Options (using helper from utils)
    let deduced_name = utils::deduce_filename(filename.clone(), &urls);

    // Smart Filename Resolution
    // Resolve save path locally to check for existence
    let resolved_save_path = if let Some(ref path) = save_path {
        utils::resolve_path(path)
    } else {
        // Default download dir? For now let's assume current dir if not set, or handled by aria2
        // But for existence check we need a path.
        // If save_path is None, Aria2 uses its default. We might not be able to check collision easily without knowing Aria2 default.
        // However, Mua might have a default setting.
        // For SAFETY, if save_path is provided, we check. If not, we might skip or best effort.
        // Let's assume most usages provide save_path (settings).
        ".".to_string()
    };

    // Get active filenames to prevent collision with running tasks
    let active_names = state.get_active_filenames();

    // Generate Unique Name
    let unique_filename =
        utils::get_unique_filename(&resolved_save_path, &deduced_name, &active_names);

    // Now build aria2 options, forcing 'out' to be unique_filename
    // We pass 'Some(unique_filename)' as filename overrides what user sent if it collided
    let (options, final_save_path) = utils::build_aria2_options(
        save_path,
        Some(unique_filename.clone()),
        user_agent,
        referer,
        headers,
        proxy,
        max_download_limit,
    );

    // Call Aria2
    match aria2_client::add_uri(urls.clone(), Some(options)).await {
        Ok(gid) => {
            // Persist to Store
            let task = PersistedTask {
                gid: gid.clone(),
                filename: unique_filename, // Use unique name
                url: urls.get(0).cloned().unwrap_or_default(),
                save_path: final_save_path,
                added_at: Local::now().to_rfc3339(),
                state: "waiting".to_string(),
                total_length: "0".to_string(),
                completed_length: "0".to_string(),
                download_speed: "0 B/s".to_string(),
                error_message: "".to_string(),
            };
            state.add_task(task);
            Ok(gid)
        }
        Err(e) => Err(e),
    }
}

#[tauri::command]
pub async fn pause_task(state: tauri::State<'_, TaskStore>, gid: String) -> Result<String, String> {
    // Call Aria2 first, only update store on success
    let result = aria2_client::pause(gid.clone()).await?;
    state.update_task_state(&gid, "paused");
    Ok(result)
}

#[tauri::command]

pub async fn resume_task(
    state: tauri::State<'_, TaskStore>,
    gid: String,
) -> Result<String, String> {
    // 1. Check Store state first
    let should_smart_resume = if let Some(task) = state.get_task(&gid) {
        task.state == "cancelled"
            || task.state == "error"
            || task.state == "completed"
            || task.state == "removed"
            || task.state == "missing"
    } else {
        // If not in store, we can't do anything smart, try generic resume
        false
    };

    if should_smart_resume {
        return smart_resume_task(&state, gid).await;
    }

    // 2. Try standard resume for active/paused tasks
    match aria2_client::resume(gid.clone()).await {
        Ok(res) => {
            state.update_task_state(&gid, "waiting");
            Ok(res)
        }
        Err(e) => {
            // Check if GID is lost/not found (maybe state said paused but aria2 lost it)
            if e.to_lowercase().contains("not found") || e.to_lowercase().contains("error 1") {
                smart_resume_task(&state, gid).await
            } else {
                Err(e)
            }
        }
    }
}

// Helper for re-submitting a task
async fn smart_resume_task(state: &TaskStore, gid: String) -> Result<String, String> {
    log::info!("Attempting Smart Resume for GID: {}", gid);

    if let Some(task) = state.get_task(&gid) {
        // Re-submit task
        log::info!("Resubmitting task: {}", task.filename);

        let mut options = serde_json::Map::new();
        if !task.save_path.is_empty() {
            let p = utils::resolve_path(&task.save_path);
            options.insert("dir".to_string(), serde_json::Value::String(p));
        }
        if !task.filename.is_empty() {
            // Ensure we don't double-nest filename if it was inferred
            options.insert(
                "out".to_string(),
                serde_json::Value::String(task.filename.clone()),
            );
        }

        // 0. Purge old task result if it exists in Aria2 to prevent GID conflict or stale data
        let _ = aria2_client::purge(gid.clone()).await;

        // Re-add URI
        match aria2_client::add_uri(
            vec![task.url.clone()],
            Some(serde_json::Value::Object(options)),
        )
        .await
        {
            Ok(new_gid) => {
                log::info!(
                    "Smart Resume successful. Old GID: {}, New GID: {}",
                    gid,
                    new_gid
                );

                // Remove old record
                let removed = state.remove_task(&gid);
                log::info!("Smart Resume: Removed old task {}? {}", gid, removed);

                // Add new record
                // We keep the original 'added_at' for history continuity?
                // Or maybe update it? Let's update it to now so it bumps to top.
                let new_task = PersistedTask {
                    gid: new_gid.clone(),
                    state: "waiting".to_string(),
                    added_at: Local::now().to_rfc3339(),
                    total_length: "0".to_string(), // Reset progress
                    completed_length: "0".to_string(),
                    download_speed: "0".to_string(),
                    ..task
                };
                state.add_task(new_task);

                Ok(new_gid)
            }
            Err(add_err) => {
                log::error!("Smart Resume failed to re-add task: {}", add_err);
                Err(format!("Smart Resume Failed: {}", add_err))
            }
        }
    } else {
        Err("Task not found in store, cannot resume".to_string())
    }
}

#[tauri::command]
pub async fn cancel_task(
    state: tauri::State<'_, TaskStore>,
    gid: String,
) -> Result<String, String> {
    // Call Aria2 first, only update store on success
    let result = aria2_client::remove(gid.clone()).await?;
    state.update_task_state(&gid, "cancelled");
    Ok(result)
}

#[tauri::command]
pub async fn pause_all_tasks(state: tauri::State<'_, TaskStore>) -> Result<String, String> {
    // Call Aria2 first, only update store on success
    let result = aria2_client::pause_all().await?;
    state.update_all_active_to_paused();
    Ok(result)
}

#[tauri::command]
pub async fn resume_all_tasks(state: tauri::State<'_, TaskStore>) -> Result<String, String> {
    // Call Aria2 first, only update store on success
    let result = aria2_client::unpause_all().await?;
    state.update_all_paused_to_waiting();
    Ok(result)
}

#[tauri::command]
pub async fn cancel_tasks(
    state: tauri::State<'_, TaskStore>,
    gids: Vec<String>,
) -> Result<String, String> {
    // 1. 批量更新状态（只触发一次 save）
    state.update_batch_state(&gids, "cancelled");
    state.save();

    // 2. 并发调用 Aria2 取消任务
    let futures: Vec<_> = gids
        .iter()
        .map(|gid| aria2_client::remove(gid.clone()))
        .collect();
    let _ = join_all(futures).await;

    Ok("OK".to_string())
}

#[tauri::command]
pub async fn remove_tasks(
    state: tauri::State<'_, TaskStore>,
    gids: Vec<String>,
    delete_file: bool,
) -> Result<String, String> {
    // 0. 获取所有任务信息用于文件删除和状态判断
    let tasks_info: Vec<_> = gids.iter().filter_map(|gid| state.get_task(gid)).collect();

    // 1. 分类：活跃任务和非活跃任务
    let (active_gids, _inactive_gids): (Vec<_>, Vec<_>) =
        tasks_info.iter().map(|t| t.gid.clone()).partition(|gid| {
            tasks_info
                .iter()
                .find(|t| &t.gid == gid)
                .map(|t| t.state == "downloading" || t.state == "waiting" || t.state == "paused")
                .unwrap_or(false)
        });

    // 1. 并发处理 Aria2
    // 活跃任务：remove (停止)
    let active_futures: Vec<_> = active_gids
        .iter()
        .map(|gid| aria2_client::remove(gid.clone()))
        .collect();
    let _ = join_all(active_futures).await;

    // 所有任务：purge (清理结果)
    let purge_futures: Vec<_> = gids
        .iter()
        .map(|gid| aria2_client::purge(gid.clone()))
        .collect();
    let _ = join_all(purge_futures).await;

    // 2. 批量从 Store 删除 (Aria2 处理后再删记录，防止通知失败导致孤儿任务)
    state.remove_tasks_batch(&gids);
    state.save();

    // 4. 删除文件（如果需要）
    if delete_file {
        for task in &tasks_info {
            let filepath = if !task.save_path.is_empty() && !task.filename.is_empty() {
                if task.save_path.ends_with('/') {
                    Some(format!("{}{}", task.save_path, task.filename))
                } else {
                    Some(format!("{}/{}", task.save_path, task.filename))
                }
            } else {
                None
            };

            if let Some(path) = filepath {
                let resolved_path = utils::resolve_path(&path);
                if std::path::Path::new(&resolved_path).exists() {
                    if let Err(e) = std::fs::remove_file(&resolved_path) {
                        log::error!("Failed to delete file {}: {}", resolved_path, e);
                    }
                }
                // 清理 .aria2 控制文件
                let aria2_file_path = format!("{}.aria2", resolved_path);
                let _ = std::fs::remove_file(&aria2_file_path);
            }
        }
    }

    Ok("OK".to_string())
}

#[tauri::command]
pub async fn remove_task_record(
    state: tauri::State<'_, TaskStore>,
    gid: String,
    delete_file: bool,
) -> Result<String, String> {
    remove_task_inner(&state, gid, delete_file).await
}

// Inner helper to avoid code duplication
async fn remove_task_inner(
    state: &TaskStore,
    gid: String,
    delete_file: bool,
) -> Result<String, String> {
    // 0. Get task info BEFORE removal to know path
    let task_opt = state.get_task(&gid);

    // Check if task is active
    let is_active = if let Some(ref t) = task_opt {
        t.state == "downloading" || t.state == "waiting" || t.state == "paused"
    } else {
        false
    };

    // 1. Remove from Aria2 First
    if is_active {
        // If active, we must cancel it first. "remove" in aria2 triggers stop.
        let _ = aria2_client::remove(gid.clone()).await;
        // We try to purge too, just in case
        let _ = aria2_client::purge(gid.clone()).await;
    } else {
        // If stopped/completed, we purge the result
        let _ = aria2_client::purge(gid.clone()).await;
    }

    // 2. Remove from Store Only after Aria2 was notified
    state.remove_task(&gid);

    // 3. Delete File if requested
    if delete_file {
        if let Some(task) = task_opt {
            // Unsafe to delete file if it was just active and we only signaled 'remove'.
            // Aria2 might still hold the lock for a split second.
            // But we will try anyway, logging warning on failure.

            let filepath = if !task.save_path.is_empty() && !task.filename.is_empty() {
                if task.save_path.ends_with('/') {
                    Some(format!("{}{}", task.save_path, task.filename))
                } else {
                    Some(format!("{}/{}", task.save_path, task.filename))
                }
            } else {
                None
            };

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

                // 4. Try to clean .aria2 control file
                let aria2_file_path = format!("{}.aria2", resolved_path);
                if std::path::Path::new(&aria2_file_path).exists() {
                    let _ = std::fs::remove_file(&aria2_file_path);
                }
            }
        }
    }
    Ok("OK".to_string())
}

#[tauri::command]
pub async fn show_task_in_folder(
    state: tauri::State<'_, TaskStore>,
    gid: String,
) -> Result<(), String> {
    if let Some(task) = state.get_task(&gid) {
        let full_path = if !task.save_path.is_empty() {
            if task.save_path.ends_with('/') {
                format!("{}{}", task.save_path, task.filename)
            } else {
                format!("{}/{}", task.save_path, task.filename)
            }
        } else {
            task.filename.clone()
        };

        utils::show_in_file_manager(&full_path);
        Ok(())
    } else {
        Err("Task not found".to_string())
    }
}

// Frontend DTO imported from sync module
use crate::core::sync::FrontendTask;

#[tauri::command]
pub async fn get_tasks(
    state: tauri::State<'_, TaskStore>,
    app_handle: AppHandle,
) -> Result<Vec<FrontendTask>, String> {
    crate::core::sync::sync_tasks(&state, &app_handle).await
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
pub async fn get_app_config(app: AppHandle) -> Result<crate::core::config::AppConfig, String> {
    Ok(crate::core::config::load_config(&app))
}

#[tauri::command]
pub async fn save_app_config(
    app: AppHandle,
    config: crate::core::config::AppConfig,
) -> Result<(), String> {
    // 1. 保存到磁盘
    crate::core::config::save_config(&app, &config)?;

    // 2. 我们可能想要重启 aria2 或者只是让用户重启应用。
    // 动态重启 sidecar 比较复杂。提示重启更好。
    Ok(())
}

#[tauri::command]
pub fn start_log_stream(app: AppHandle) {
    if let Some(state) = app.try_state::<crate::aria2::sidecar::LogStreamEnabled>() {
        state.0.store(true, std::sync::atomic::Ordering::Relaxed);
    }
    
    // Send buffered logs immediately
    if let Some(state) = app.try_state::<crate::aria2::sidecar::SidecarState>() {
        if let Ok(logs) = state.recent_logs.lock() {
            for log in logs.iter() {
                let _ = app.emit("aria2-stdout", log);
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

#[tauri::command]
pub async fn import_custom_binary(app: AppHandle, file_path: String) -> Result<String, String> {
    use std::fs;
    use std::path::Path;
    #[cfg(unix)]
    use std::os::unix::fs::PermissionsExt;

    // 1. Prepare target path
    let config_dir = app.path().app_config_dir().map_err(|e| e.to_string())?;
    let bin_dir = config_dir.join("custom-bin");
    if !bin_dir.exists() {
        fs::create_dir_all(&bin_dir).map_err(|e| e.to_string())?;
    }
    
    let target_name = if cfg!(windows) { "aria2c.exe" } else { "aria2c" };
    let target_path = bin_dir.join(target_name);

    // 2. Scan constraints
    let source_path = Path::new(&file_path);
    if !source_path.exists() {
        return Err("Source file does not exist".to_string());
    }

    // 3. Copy file
    fs::copy(source_path, &target_path).map_err(|e| format!("Copy failed: {}", e))?;

    // 4. Set permissions (Unix)
    #[cfg(unix)]
    {
        let mut perms = fs::metadata(&target_path).map_err(|e| e.to_string())?.permissions();
        perms.set_mode(0o755);
        fs::set_permissions(&target_path, perms).map_err(|e| e.to_string())?;
    }

    // 5. Validation (Dry Run)
    let output = std::process::Command::new(&target_path)
        .arg("--version")
        .output()
        .map_err(|e| format!("Validation failed (exec error): {}", e))?;

    if !output.status.success() {
        // Cleanup bad file
        let _ = fs::remove_file(&target_path);
        return Err("Validation failed: integrity check returned non-zero exit code".to_string());
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    if !stdout.contains("aria2 version") {
        let _ = fs::remove_file(&target_path);
        return Err("Validation failed: Not a valid aria2 binary".to_string());
    }
    
    // Parse version (simplistic)
    // "aria2 version 1.36.0"
    let version_line = stdout.lines().next().unwrap_or("Unknown");
    
    Ok(version_line.to_string())
}

#[derive(serde::Serialize)]
pub struct Aria2VersionInfo {
    pub version: String,
    pub is_custom: bool,
    pub path: String,
    pub custom_binary_exists: bool,
    pub custom_binary_version: Option<String>,
}

#[tauri::command]
pub async fn get_aria2_version_info(app: AppHandle) -> Result<Aria2VersionInfo, String> {
    let config_dir = app.path().app_config_dir().map_err(|e| e.to_string())?;
    let target_name = if cfg!(windows) { "aria2c.exe" } else { "aria2c" };
    let custom_path = config_dir.join("custom-bin").join(target_name);
    
    // Check if custom binary exists and get its version
    let (has_custom, custom_ver) = if custom_path.exists() {
        match std::process::Command::new(&custom_path).arg("--version").output() {
            Ok(output) if output.status.success() => {
               let stdout = String::from_utf8_lossy(&output.stdout);
               let ver = stdout.lines().next().unwrap_or("Unknown").to_string();
               (true, Some(ver))
            },
            _ => (false, None)
        }
    } else {
        (false, None)
    };
    
    let config = crate::core::config::load_config(&app);
    
    // Determine info about the *Active* kernel
    if config.use_custom_aria2 && has_custom {
        Ok(Aria2VersionInfo {
            version: custom_ver.clone().unwrap_or_default(),
            is_custom: true,
            path: custom_path.to_string_lossy().to_string(),
            custom_binary_exists: true,
            custom_binary_version: custom_ver,
        })
    } else {
        Ok(Aria2VersionInfo {
            version: "Built-in (1.36.0)".to_string(), 
            is_custom: false,
            path: "Embedded Sidecar".to_string(),
            custom_binary_exists: has_custom,
            custom_binary_version: custom_ver,
        })
    }
}
