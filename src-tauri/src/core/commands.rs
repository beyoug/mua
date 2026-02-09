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

    // 验证
    for url in &urls {
        if !utils::is_valid_url(url) {
            return Err(format!("无效的 URL: {}", url));
        }
    }

    // 1. 构建选项（使用 utils 中的辅助函数）
    let deduced_name = utils::deduce_filename(filename.clone(), &urls);

    // 智能文件名解析
    // 在本地解析保存路径以检查是否存在
    let resolved_save_path = if let Some(ref path) = save_path {
        utils::resolve_path(path)
    } else {
        // 默认下载目录？目前如果没有设置，假设为当前目录，或者由 aria2 处理。
        // 但是为了检查是否存在，我们需要一个路径。
        // 如果 save_path 为 None，Aria2 会使用其默认路径。在不知道 Aria2 默认路径的情况下，我们可能很难检查冲突。
        // 然而，Mua 可能会有一个默认设置。
        // 为了安全起见，如果提供了 save_path，我们就进行检查。如果没有提供，我们可能会跳过或尽力而为。
        // 我们假设大多数用法都会提供 save_path（设置中）。
        ".".to_string()
    };

    // 获取活跃文件名以防止与正在运行的任务冲突
    let active_names = state.get_active_filenames();

    // 生成唯一名称
    let unique_filename =
        utils::get_unique_filename(&resolved_save_path, &deduced_name, &active_names);

    // 现在构建 aria2 选项，强制 'out' 为 unique_filename
    // 我们传递 'Some(unique_filename)'，如果发生冲突，重命名的文件名将覆盖用户发送的文件名
    let (options, final_save_path) = utils::build_aria2_options(
        save_path,
        Some(unique_filename.clone()),
        user_agent,
        referer,
        headers,
        proxy,
        max_download_limit,
    );

    // 调用 Aria2
    match aria2_client::add_uri(urls.clone(), Some(options)).await {
        Ok(gid) => {
            // 持久化到存储 (Store)
            let task = PersistedTask {
                gid: gid.clone(),
                filename: unique_filename, // 使用唯一名称
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
    // 首先调用 Aria2，仅在成功时更新存储
    let result = aria2_client::pause(gid.clone()).await?;
    state.update_task_state(&gid, "paused");
    Ok(result)
}

#[tauri::command]

pub async fn resume_task(
    state: tauri::State<'_, TaskStore>,
    gid: String,
) -> Result<String, String> {
    // 1. 首先检查 Store 状态
    let should_smart_resume = if let Some(task) = state.get_task(&gid) {
        task.state == "cancelled"
            || task.state == "error"
            || task.state == "completed"
            || task.state == "removed"
            || task.state == "missing"
    } else {
        // 如果不在存储中，我们无法执行任何智能操作，尝试通用恢复
        false
    };

    if should_smart_resume {
        return smart_resume_task(&state, gid).await;
    }

    // 2. 为活跃/已暂停的任务尝试标准恢复 (resume)
    match aria2_client::resume(gid.clone()).await {
        Ok(res) => {
            state.update_task_state(&gid, "waiting");
            Ok(res)
        }
        Err(e) => {
            // 检查 GID 是否丢失/未找到（可能状态显示为已暂停，但 aria2 丢失了它）
            if e.to_lowercase().contains("not found") || e.to_lowercase().contains("error 1") {
                smart_resume_task(&state, gid).await
            } else {
                Err(e)
            }
        }
    }
}

// 用于重新提交任务的辅助函数
async fn smart_resume_task(state: &TaskStore, gid: String) -> Result<String, String> {
    log::info!("正在为 GID {} 尝试智能恢复 (Smart Resume)", gid);

    if let Some(task) = state.get_task(&gid) {
        // 重新提交任务
        log::info!("正在重新提交任务: {}", task.filename);

        let mut options = serde_json::Map::new();
        if !task.save_path.is_empty() {
            let p = utils::resolve_path(&task.save_path);
            options.insert("dir".to_string(), serde_json::Value::String(p));
        }
        if !task.filename.is_empty() {
            // 确保如果文件名是推断出来的，我们不会进行双重嵌套
            options.insert(
                "out".to_string(),
                serde_json::Value::String(task.filename.clone()),
            );
        }

        // 0. 如果旧任务结果存在于 Aria2 中，则清除它，以防止 GID 冲突或陈旧数据
        let _ = aria2_client::purge(gid.clone()).await;

        // 重新添加 URI
        match aria2_client::add_uri(
            vec![task.url.clone()],
            Some(serde_json::Value::Object(options)),
        )
        .await
        {
            Ok(new_gid) => {
                log::info!("智能恢复成功。旧 GID: {}, 新 GID: {}", gid, new_gid);

                // 移除旧记录
                let removed = state.remove_task(&gid);
                log::info!("智能恢复：已移除旧任务 {}？{}", gid, removed);

                // 添加新记录
                // 我们是否为了历史连续性保留原始的 'added_at'？
                // 或者更新它？让我们把它更新为“现在”，这样它就会排在最前面。
                let new_task = PersistedTask {
                    gid: new_gid.clone(),
                    state: "waiting".to_string(),
                    added_at: Local::now().to_rfc3339(),
                    total_length: "0".to_string(), // 重置进度
                    completed_length: "0".to_string(),
                    download_speed: "0".to_string(),
                    ..task
                };
                state.add_task(new_task);

                Ok(new_gid)
            }
            Err(add_err) => {
                log::error!("智能恢复重新添加任务失败: {}", add_err);
                Err(format!("智能恢复失败: {}", add_err))
            }
        }
    } else {
        Err("存储中未找到任务，无法恢复".to_string())
    }
}

#[tauri::command]
pub async fn cancel_task(
    state: tauri::State<'_, TaskStore>,
    gid: String,
) -> Result<String, String> {
    // 首先调用 Aria2，仅在成功时更新存储
    let result = aria2_client::remove(gid.clone()).await?;
    state.update_task_state(&gid, "cancelled");
    Ok(result)
}

#[tauri::command]
pub async fn pause_all_tasks(state: tauri::State<'_, TaskStore>) -> Result<String, String> {
    // 首先调用 Aria2，仅在成功时更新存储
    let result = aria2_client::pause_all().await?;
    state.update_all_active_to_paused();
    Ok(result)
}

#[tauri::command]
pub async fn resume_all_tasks(state: tauri::State<'_, TaskStore>) -> Result<String, String> {
    // 首先调用 Aria2，仅在成功时更新存储
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
                .map(|t| utils::is_active_state(&t.state))
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
            let full_path = utils::get_full_path(&task.save_path, &task.filename);
            let resolved_path = utils::resolve_path(&full_path);
            if std::path::Path::new(&resolved_path).exists() {
                if let Err(e) = std::fs::remove_file(&resolved_path) {
                    log::error!("删除文件失败 {}: {}", resolved_path, e);
                }
            }
            // 清理 .aria2 控制文件
            let aria2_file_path = format!("{}.aria2", resolved_path);
            let _ = std::fs::remove_file(&aria2_file_path);
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

// 内部辅助函数以避免代码重复
async fn remove_task_inner(
    state: &TaskStore,
    gid: String,
    delete_file: bool,
) -> Result<String, String> {
    // 0. 在删除前获取任务信息以获取路径
    let task_opt = state.get_task(&gid);

    // 检查任务是否活跃
    let is_active = task_opt
        .as_ref()
        .map_or(false, |t| utils::is_active_state(&t.state));

    // 1. 首先从 Aria2 中移除
    if is_active {
        // 如果是活跃状态，我们必须先取消它。aria2 中的 "remove" 会触发停止。
        let _ = aria2_client::remove(gid.clone()).await;
        // 我们也尝试清理 (purge)，以防万一
        let _ = aria2_client::purge(gid.clone()).await;
    } else {
        // 如果已停止/已完成，我们清理结果
        let _ = aria2_client::purge(gid.clone()).await;
    }

    // 2. 仅在通知 Aria2 后从 Store 中删除
    state.remove_task(&gid);

    // 3. 如果请求，则删除文件
    if delete_file {
        if let Some(task) = task_opt {
            // 如果任务刚才还是活跃的，并且我们仅发送了 'remove' 信号，那么删除文件是不安全的。
            // Aria2 可能仍会在一瞬间持有锁。
            // 但我们会尝试执行，并在失败时记录警告。

            let full_path = utils::get_full_path(&task.save_path, &task.filename);
            let resolved_path = utils::resolve_path(&full_path);

            if std::path::Path::new(&resolved_path).exists() {
                match std::fs::remove_file(&resolved_path) {
                    Ok(_) => log::info!("已删除文件: {}", resolved_path),
                    Err(e) => log::error!("删除文件失败 {}: {}", resolved_path, e),
                }
            } else {
                log::warn!("未找到要删除的文件: {}", resolved_path);
            }

            // 4. 尝试清理 .aria2 控制文件
            let aria2_file_path = format!("{}.aria2", resolved_path);
            if std::path::Path::new(&aria2_file_path).exists() {
                let _ = std::fs::remove_file(&aria2_file_path);
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
        let full_path = utils::get_full_path(&task.save_path, &task.filename);
        utils::show_in_file_manager(&full_path);
        Ok(())
    } else {
        Err("未找到任务".to_string())
    }
}

// 从 sync 模块导入的前端 DTO
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

    // 2. 更新内存中的状态，确保事件处理器能立即读取到最新配置
    if let Some(state) = app.try_state::<crate::core::config::ConfigState>() {
        if let Ok(mut lock) = state.config.lock() {
            *lock = config.clone();
        }
    }

    // 3. 实时同步到正在运行的 Aria2 内核 (针对支持动态修改的选项)
    let mut options = serde_json::Map::new();
    options.insert(
        "max-concurrent-downloads".to_string(),
        serde_json::Value::String(config.max_concurrent_downloads.to_string()),
    );

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
    #[cfg(unix)]
    use std::os::unix::fs::PermissionsExt;
    use std::path::Path;

    // 1. 准备目标路径
    let config_dir = app.path().app_config_dir().map_err(|e| e.to_string())?;
    let bin_dir = config_dir.join("custom-bin");
    if !bin_dir.exists() {
        fs::create_dir_all(&bin_dir).map_err(|e| e.to_string())?;
    }

    let target_name = if cfg!(windows) {
        "aria2c.exe"
    } else {
        "aria2c"
    };
    let target_path = bin_dir.join(target_name);

    // 2. 检查约束
    let source_path = Path::new(&file_path);
    if !source_path.exists() {
        return Err("源文件不存在".to_string());
    }

    // 3. 复制文件
    fs::copy(source_path, &target_path).map_err(|e| format!("复制失败: {}", e))?;

    // 4. 设置权限 (Unix)
    #[cfg(unix)]
    {
        let mut perms = fs::metadata(&target_path)
            .map_err(|e| e.to_string())?
            .permissions();
        perms.set_mode(0o755);
        fs::set_permissions(&target_path, perms).map_err(|e| e.to_string())?;
    }

    // 5. 验证 (空运行)
    let output = std::process::Command::new(&target_path)
        .arg("--version")
        .output()
        .map_err(|e| format!("验证失败 (执行错误): {}", e))?;

    if !output.status.success() {
        // 清理错误文件
        let _ = fs::remove_file(&target_path);
        return Err("验证失败：完整性检查返回非零退出代码".to_string());
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    if !stdout.contains("aria2 version") {
        let _ = fs::remove_file(&target_path);
        return Err("验证失败：不是有效的 aria2 二进制文件".to_string());
    }

    // 解析版本（简单处理）
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
    let target_name = if cfg!(windows) {
        "aria2c.exe"
    } else {
        "aria2c"
    };
    let custom_path = config_dir.join("custom-bin").join(target_name);

    // 检查自定义二进制文件是否存在并获取其版本
    let (has_custom, custom_ver) = if custom_path.exists() {
        match std::process::Command::new(&custom_path)
            .arg("--version")
            .output()
        {
            Ok(output) if output.status.success() => {
                let stdout = String::from_utf8_lossy(&output.stdout);
                let ver = stdout.lines().next().unwrap_or("Unknown").to_string();
                (true, Some(ver))
            }
            _ => (false, None),
        }
    } else {
        (false, None)
    };

    let config = crate::core::config::load_config(&app);

    // 确定活跃内核的信息
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
