//! 任务相关命令
//! 包含下载任务的 CRUD 操作

use crate::aria2::client as aria2_client;
use crate::core::error::{AppError, AppResult};
use crate::core::types::{DownloadConfig, TaskState};
use crate::utils;
use futures::future::join_all;
use tauri::AppHandle;

use crate::core::store::{PersistedTask, TaskStore};
use crate::core::sync::FrontendTask;
use chrono::Local;

#[tauri::command]
pub async fn add_download_tasks(
    state: tauri::State<'_, TaskStore>,
    configs: Vec<DownloadConfig>,
) -> AppResult<Vec<Option<String>>> {
    let futures = configs
        .into_iter()
        .map(|cfg| add_download_task_inner(&state, cfg));

    // 并发执行添加
    let results = join_all(futures).await;

    let mut gids = Vec::new();
    let mut errors = Vec::new();

    for res in results {
        match res {
            Ok(gid) => gids.push(Some(gid)),
            Err(e) => {
                errors.push(e.to_string());
                gids.push(None);
            }
        }
    }

    if !errors.is_empty() {
        log::warn!("批量添加遇到 {} 个错误: {:?}", errors.len(), errors);
    }

    Ok(gids)
}

use base64::Engine as _;

async fn add_download_task_inner(state: &TaskStore, cfg: DownloadConfig) -> AppResult<String> {
    log::info!("Adding download task: {:?}", cfg.urls);

    if let Some(ref torrent_cfg) = cfg.torrent_config {
        return add_torrent_task_inner(
            state,
            torrent_cfg.path.clone(),
            torrent_cfg.select_file.clone(),
            &cfg,
        )
        .await;
    }

    for url in &cfg.urls {
        if !utils::is_valid_url(url) {
            return Err(AppError::validation(format!("无效的 URL: {}", url)));
        }
    }

    let deduced_name = utils::deduce_filename(cfg.filename.clone(), &cfg.urls);

    let resolved_save_path = if let Some(ref path) = cfg.save_path {
        utils::resolve_path(path)
    } else {
        ".".to_string()
    };

    let active_names = state.get_active_filenames();
    let unique_filename =
        utils::get_unique_filename(&resolved_save_path, &deduced_name, &active_names);

    let (options, final_save_path) = utils::build_aria2_options(
        cfg.save_path.clone(),
        Some(unique_filename.clone()),
        cfg.user_agent.clone(),
        cfg.referer.clone(),
        cfg.headers.clone(),
        cfg.proxy.clone(),
        cfg.max_download_limit.clone(),
    );

    match aria2_client::add_uri(cfg.urls.clone(), Some(options)).await {
        Ok(gid) => {
            let task = create_persisted_task(
                gid.clone(),
                unique_filename,
                cfg.urls.get(0).cloned().unwrap_or_default(),
                final_save_path,
                &cfg,
            );
            state.add_task(task);
            Ok(gid)
        }
        Err(e) => Err(e),
    }
}

fn create_persisted_task(
    gid: String,
    filename: String,
    url: String,
    save_path: String,
    cfg: &DownloadConfig,
) -> PersistedTask {
    PersistedTask {
        gid,
        filename,
        url,
        save_path,
        added_at: Local::now().to_rfc3339(),
        state: TaskState::Waiting,
        total_length: "0".to_string(),
        completed_length: "0".to_string(),
        download_speed: "0 B/s".to_string(),
        completed_at: None,
        error_message: "".to_string(),
        user_agent: cfg.user_agent.clone().unwrap_or_default(),
        referer: cfg.referer.clone().unwrap_or_default(),
        proxy: cfg.proxy.clone().unwrap_or_default(),
        headers: cfg
            .headers
            .clone()
            .unwrap_or_default()
            .split(|c| c == ';' || c == '\n')
            .filter(|s| !s.trim().is_empty())
            .map(|s| s.trim().to_string())
            .collect(),
        max_download_limit: cfg.max_download_limit.clone().unwrap_or_default(),
    }
}

#[tauri::command]
pub async fn parse_torrent(path: String) -> AppResult<crate::core::torrent::TorrentInfo> {
    crate::core::torrent::parse_torrent_file(&path)
}

async fn add_torrent_task_inner(
    state: &TaskStore,
    path: String,
    select_file: Option<String>,
    base_cfg: &DownloadConfig,
) -> AppResult<String> {
    // Read file and convert to base64
    let content = std::fs::read(&path).map_err(|e| AppError::Fs(e.to_string()))?;
    let torrent_b64 = base64::engine::general_purpose::STANDARD.encode(&content);

    // Build options
    let (mut options_val, final_save_path) = utils::build_aria2_options(
        base_cfg.save_path.clone(),
        None, // Torrent usually has its own name structure, avoid forcing filename unless single file?
        base_cfg.user_agent.clone(),
        base_cfg.referer.clone(),
        base_cfg.headers.clone(),
        base_cfg.proxy.clone(),
        base_cfg.max_download_limit.clone(),
    );

    if let Some(opts) = options_val.as_object_mut() {
        if let Some(sf) = select_file {
            opts.insert("select-file".to_string(), serde_json::Value::String(sf));
        }
    }

    match aria2_client::add_torrent(torrent_b64, Some(options_val)).await {
        Ok(gid) => {
            // Torrent task might not have a URL, but use path as identifier
            let task = create_persisted_task(
                gid.clone(),
                // Use a placeholder or let Aria2 update it later (store will sync)
                // But we can try to guess name from path temporarily
                std::path::Path::new(&path)
                    .file_name()
                    .unwrap_or_default()
                    .to_string_lossy()
                    .to_string(),
                format!("file://{}", path),
                final_save_path,
                base_cfg,
            );
            state.add_task(task);
            Ok(gid)
        }
        Err(e) => Err(e),
    }
}

#[tauri::command]
pub async fn pause_task(state: tauri::State<'_, TaskStore>, gid: String) -> AppResult<String> {
    let result = aria2_client::pause(gid.clone()).await?;
    state.update_task_state(&gid, TaskState::Paused);
    Ok(result)
}

#[tauri::command]
pub async fn resume_task(state: tauri::State<'_, TaskStore>, gid: String) -> AppResult<String> {
    // 1. 首先检查 Store 状态
    let should_smart_resume = if let Some(task) = state.get_task(&gid) {
        TaskState::from(task.state.as_str()).is_terminal()
    } else {
        false
    };

    if should_smart_resume {
        return smart_resume_task(&state, gid).await;
    }

    // 2. 为活跃/已暂停的任务尝试标准恢复 (resume)
    match aria2_client::resume(gid.clone()).await {
        Ok(res) => {
            state.update_task_state(&gid, TaskState::Waiting);
            Ok(res)
        }
        Err(e) => {
            let msg = e.to_string().to_lowercase();
            if msg.contains("not found") || msg.contains("error 1") {
                smart_resume_task(&state, gid).await
            } else {
                Err(e)
            }
        }
    }
}

// 用于重新提交任务的辅助函数
async fn smart_resume_task(state: &TaskStore, gid: String) -> AppResult<String> {
    log::info!("正在为 GID {} 尝试智能恢复 (Smart Resume)", gid);

    if let Some(task) = state.get_task(&gid) {
        log::info!("正在重新提交任务: {}", task.filename);

        // 1. 从存储中还原所有高级参数
        let save_path_opt = if task.save_path.is_empty() {
            None
        } else {
            Some(task.save_path.clone())
        };
        let filename_opt = if task.filename.is_empty() {
            None
        } else {
            Some(task.filename.clone())
        };
        let ua_opt = if task.user_agent.is_empty() {
            None
        } else {
            Some(task.user_agent.clone())
        };
        let referer_opt = if task.referer.is_empty() {
            None
        } else {
            Some(task.referer.clone())
        };
        let proxy_opt = if task.proxy.is_empty() {
            None
        } else {
            Some(task.proxy.clone())
        };
        let limit_opt = if task.max_download_limit.is_empty() {
            None
        } else {
            Some(task.max_download_limit.clone())
        };

        let headers_str = if task.headers.is_empty() {
            None
        } else {
            Some(task.headers.join("; "))
        };

        // 2. 利用 utils 统一构建选项
        let (options, _) = utils::build_aria2_options(
            save_path_opt,
            filename_opt,
            ua_opt,
            referer_opt,
            headers_str,
            proxy_opt,
            limit_opt,
        );

        // 3. 如果旧任务结果存在于 Aria2 中，则清除它
        let _ = aria2_client::purge(gid.clone()).await;

        // 4. 重新添加 URI
        match aria2_client::add_uri(vec![task.url.clone()], Some(options)).await {
            Ok(new_gid) => {
                log::info!("智能恢复成功。旧 GID: {}, 新 GID: {}", gid, new_gid);

                // 移除旧记录
                let removed = state.remove_task(&gid);
                log::info!("智能恢复：已移除旧任务 {}？{}", gid, removed);

                // 添加新记录
                let new_task = PersistedTask {
                    gid: new_gid.clone(),
                    state: TaskState::Waiting,
                    added_at: Local::now().to_rfc3339(),
                    total_length: "0".to_string(),
                    completed_length: "0".to_string(),
                    download_speed: "0".to_string(),
                    completed_at: None,
                    ..task
                };
                state.add_task(new_task);

                Ok(new_gid)
            }
            Err(add_err) => {
                log::error!("智能恢复重新添加任务失败: {}", add_err);
                Err(AppError::aria2(format!("智能恢复失败: {}", add_err)))
            }
        }
    } else {
        Err(AppError::task_not_found(gid))
    }
}

#[tauri::command]
pub async fn cancel_task(state: tauri::State<'_, TaskStore>, gid: String) -> AppResult<String> {
    let result = aria2_client::remove(gid.clone()).await?;
    state.update_task_state(&gid, TaskState::Removed);
    Ok(result)
}

#[tauri::command]
pub async fn pause_all_tasks(state: tauri::State<'_, TaskStore>) -> AppResult<String> {
    let result = aria2_client::pause_all().await?;
    state.update_all_active_to_paused();
    Ok(result)
}

#[tauri::command]
pub async fn resume_all_tasks(state: tauri::State<'_, TaskStore>) -> AppResult<String> {
    let result = aria2_client::unpause_all().await?;
    state.update_all_paused_to_waiting();
    Ok(result)
}

#[tauri::command]
pub async fn cancel_tasks(
    state: tauri::State<'_, TaskStore>,
    gids: Vec<String>,
) -> AppResult<String> {
    state.update_batch_state(&gids, TaskState::Removed);
    state.save();

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
) -> AppResult<String> {
    let tasks_info: Vec<_> = gids.iter().filter_map(|gid| state.get_task(gid)).collect();

    let (active_gids, _inactive_gids): (Vec<_>, Vec<_>) =
        tasks_info.iter().map(|t| t.gid.clone()).partition(|gid| {
            tasks_info
                .iter()
                .find(|t| &t.gid == gid)
                .map(|t| t.state.is_active())
                .unwrap_or(false)
        });

    let active_futures: Vec<_> = active_gids
        .iter()
        .map(|gid| aria2_client::remove(gid.clone()))
        .collect();
    let _ = join_all(active_futures).await;

    let purge_futures: Vec<_> = gids
        .iter()
        .map(|gid| aria2_client::purge(gid.clone()))
        .collect();
    let _ = join_all(purge_futures).await;

    state.remove_tasks_batch(&gids);
    state.save();

    if delete_file {
        for task in &tasks_info {
            let full_path = utils::get_full_path(&task.save_path, &task.filename);
            let resolved_path = utils::resolve_path(&full_path);
            if std::path::Path::new(&resolved_path).exists() {
                if let Err(e) = std::fs::remove_file(&resolved_path) {
                    log::error!("删除文件失败 {}: {}", resolved_path, e);
                }
            }
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
) -> AppResult<String> {
    remove_task_inner(&state, gid, delete_file).await
}

async fn remove_task_inner(state: &TaskStore, gid: String, delete_file: bool) -> AppResult<String> {
    let task_opt = state.get_task(&gid);

    let is_active = task_opt
        .as_ref()
        .map_or(false, |t| t.state.is_active());

    if is_active {
        let _ = aria2_client::remove(gid.clone()).await;
        let _ = aria2_client::purge(gid.clone()).await;
    } else {
        let _ = aria2_client::purge(gid.clone()).await;
    }

    state.remove_task(&gid);

    if delete_file {
        if let Some(task) = task_opt {
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

            let aria2_file_path = format!("{}.aria2", resolved_path);
            if std::path::Path::new(&aria2_file_path).exists() {
                let _ = std::fs::remove_file(&aria2_file_path);
            }
        }
    }
    Ok("OK".to_string())
}

#[tauri::command]
pub async fn show_task_in_folder(state: tauri::State<'_, TaskStore>, gid: String) -> AppResult<()> {
    if let Some(task) = state.get_task(&gid) {
        let full_path = utils::get_full_path(&task.save_path, &task.filename);
        utils::show_in_file_manager(&full_path);
        Ok(())
    } else {
        Err(AppError::task_not_found(gid))
    }
}

#[tauri::command]
pub async fn get_tasks(
    state: tauri::State<'_, TaskStore>,
    app_handle: AppHandle,
) -> AppResult<Vec<FrontendTask>> {
    crate::core::sync::sync_tasks(&state, &app_handle).await
}
