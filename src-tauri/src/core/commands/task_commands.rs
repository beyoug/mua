//! 任务相关命令
//! 包含下载任务的 CRUD 操作

use crate::aria2::client as aria2_client;
use crate::utils;
use futures::future::join_all;
use tauri::AppHandle;

use crate::core::store::{PersistedTask, TaskStore};
use crate::core::sync::FrontendTask;
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
    let resolved_save_path = if let Some(ref path) = save_path {
        utils::resolve_path(path)
    } else {
        ".".to_string()
    };

    // 获取活跃文件名以防止与正在运行的任务冲突
    let active_names = state.get_active_filenames();

    // 生成唯一名称
    let unique_filename =
        utils::get_unique_filename(&resolved_save_path, &deduced_name, &active_names);

    // 现在构建 aria2 选项，强制 'out' 为 unique_filename
    let (options, final_save_path) = utils::build_aria2_options(
        save_path,
        Some(unique_filename.clone()),
        user_agent.clone(),
        referer.clone(),
        headers.clone(),
        proxy.clone(),
        max_download_limit.clone(),
    );

    // 调用 Aria2
    match aria2_client::add_uri(urls.clone(), Some(options)).await {
        Ok(gid) => {
            // 持久化到存储 (Store)
            let task = PersistedTask {
                gid: gid.clone(),
                filename: unique_filename,
                url: urls.get(0).cloned().unwrap_or_default(),
                save_path: final_save_path,
                added_at: Local::now().to_rfc3339(),
                state: "waiting".to_string(),
                total_length: "0".to_string(),
                completed_length: "0".to_string(),
                download_speed: "0 B/s".to_string(),
                error_message: "".to_string(),
                user_agent: user_agent.unwrap_or_default(),
                referer: referer.unwrap_or_default(),
                proxy: proxy.unwrap_or_default(),
                headers: headers
                    .unwrap_or_default()
                    .split(';')
                    .filter(|s| !s.trim().is_empty())
                    .map(|s| s.trim().to_string())
                    .collect(),
                max_download_limit: max_download_limit.unwrap_or_default(),
            };
            state.add_task(task);
            Ok(gid)
        }
        Err(e) => Err(e),
    }
}

#[tauri::command]
pub async fn pause_task(state: tauri::State<'_, TaskStore>, gid: String) -> Result<String, String> {
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
        log::info!("正在重新提交任务: {}", task.filename);

        // 1. 从存储中还原所有高级参数
        let save_path_opt = if task.save_path.is_empty() { None } else { Some(task.save_path.clone()) };
        let filename_opt = if task.filename.is_empty() { None } else { Some(task.filename.clone()) };
        let ua_opt = if task.user_agent.is_empty() { None } else { Some(task.user_agent.clone()) };
        let referer_opt = if task.referer.is_empty() { None } else { Some(task.referer.clone()) };
        let proxy_opt = if task.proxy.is_empty() { None } else { Some(task.proxy.clone()) };
        let limit_opt = if task.max_download_limit.is_empty() { None } else { Some(task.max_download_limit.clone()) };
        
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
                    state: "waiting".to_string(),
                    added_at: Local::now().to_rfc3339(),
                    total_length: "0".to_string(),
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
    let result = aria2_client::remove(gid.clone()).await?;
    state.update_task_state(&gid, "cancelled");
    Ok(result)
}

#[tauri::command]
pub async fn pause_all_tasks(state: tauri::State<'_, TaskStore>) -> Result<String, String> {
    let result = aria2_client::pause_all().await?;
    state.update_all_active_to_paused();
    Ok(result)
}

#[tauri::command]
pub async fn resume_all_tasks(state: tauri::State<'_, TaskStore>) -> Result<String, String> {
    let result = aria2_client::unpause_all().await?;
    state.update_all_paused_to_waiting();
    Ok(result)
}

#[tauri::command]
pub async fn cancel_tasks(
    state: tauri::State<'_, TaskStore>,
    gids: Vec<String>,
) -> Result<String, String> {
    state.update_batch_state(&gids, "cancelled");
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
) -> Result<String, String> {
    let tasks_info: Vec<_> = gids.iter().filter_map(|gid| state.get_task(gid)).collect();

    let (active_gids, _inactive_gids): (Vec<_>, Vec<_>) =
        tasks_info.iter().map(|t| t.gid.clone()).partition(|gid| {
            tasks_info
                .iter()
                .find(|t| &t.gid == gid)
                .map(|t| utils::is_active_state(&t.state))
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
) -> Result<String, String> {
    remove_task_inner(&state, gid, delete_file).await
}

async fn remove_task_inner(
    state: &TaskStore,
    gid: String,
    delete_file: bool,
) -> Result<String, String> {
    let task_opt = state.get_task(&gid);

    let is_active = task_opt
        .as_ref()
        .map_or(false, |t| utils::is_active_state(&t.state));

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

#[tauri::command]
pub async fn get_tasks(
    state: tauri::State<'_, TaskStore>,
    app_handle: AppHandle,
) -> Result<Vec<FrontendTask>, String> {
    crate::core::sync::sync_tasks(&state, &app_handle).await
}
