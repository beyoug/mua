use crate::aria2::client as aria2_client;
use crate::core::error::{AppError, AppResult};
use crate::core::store::{PersistedTask, TaskStore};
use crate::core::types::TaskState;
use crate::utils;
use chrono::Local;
use futures::future::join_all;
use serde_json::json;

#[tauri::command]
pub async fn pause_task(state: tauri::State<'_, TaskStore>, gid: String) -> AppResult<()> {
    aria2_client::pause(gid.clone()).await?;
    state.update_task_state(&gid, TaskState::Paused);
    Ok(())
}

#[tauri::command]
pub async fn resume_task(state: tauri::State<'_, TaskStore>, gid: String) -> AppResult<String> {
    let should_smart_resume = if let Some(task) = state.get_task(&gid) {
        TaskState::from(task.state.as_str()).is_terminal()
    } else {
        false
    };

    if should_smart_resume {
        return smart_resume_task(&state, gid).await;
    }

    match aria2_client::resume(gid.clone()).await {
        Ok(res) => {
            state.update_task_state(&gid, TaskState::Waiting);
            Ok(res)
        }
        Err(e) => {
            if e.is_aria2_not_found() {
                smart_resume_task(&state, gid).await
            } else {
                Err(e)
            }
        }
    }
}

#[tauri::command]
pub async fn cancel_task(state: tauri::State<'_, TaskStore>, gid: String) -> AppResult<()> {
    aria2_client::remove(gid.clone()).await?;
    state.update_task_state(&gid, TaskState::Removed);
    Ok(())
}

#[tauri::command]
pub async fn pause_all_tasks(state: tauri::State<'_, TaskStore>) -> AppResult<()> {
    aria2_client::pause_all().await?;
    state.update_all_active_to_paused();
    Ok(())
}

#[tauri::command]
pub async fn resume_all_tasks(state: tauri::State<'_, TaskStore>) -> AppResult<()> {
    aria2_client::unpause_all().await?;
    state.update_all_paused_to_waiting();
    Ok(())
}

#[tauri::command]
pub async fn cancel_tasks(
    state: tauri::State<'_, TaskStore>,
    gids: Vec<String>,
) -> AppResult<()> {
    state.update_batch_state(&gids, TaskState::Removed);
    state.save();

    let futures: Vec<_> = gids
        .iter()
        .map(|gid| aria2_client::remove(gid.clone()))
        .collect();
    let results = join_all(futures).await;

    let mut failed: Vec<String> = Vec::new();
    for (idx, result) in results.into_iter().enumerate() {
        if let Err(error) = result {
            let gid = gids.get(idx).cloned().unwrap_or_else(|| "unknown".to_string());
            failed.push(gid.clone());
            crate::app_warn!(
                "Core::TaskControl",
                "cancel_batch_item_failed",
                json!({ "gid": gid, "error": error.to_string() })
            );
        }
    }

    if !failed.is_empty() {
        crate::app_warn!(
            "Core::TaskControl",
            "cancel_batch_partial_failure",
            json!({ "failed_gids": failed })
        );
    }

    Ok(())
}

async fn smart_resume_task(state: &TaskStore, gid: String) -> AppResult<String> {
    crate::app_info!(
        "Core::TaskControl",
        "smart_resume_started",
        json!({ "gid": gid.clone() })
    );

    if let Some(task) = state.get_task(&gid) {
        crate::app_info!(
            "Core::TaskControl",
            "smart_resume_readd_attempt",
            json!({ "gid": gid.clone(), "filename": task.filename })
        );

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

        let (options, _) = utils::build_aria2_options(
            save_path_opt.clone(),
            filename_opt.clone(),
            ua_opt.clone(),
            referer_opt.clone(),
            headers_str.clone(),
            proxy_opt.clone(),
            limit_opt.clone(),
        );

        if let Err(error) = aria2_client::purge(gid.clone()).await {
            crate::app_warn!(
                "Core::TaskControl",
                "smart_resume_purge_failed",
                json!({ "gid": gid.clone(), "error": error.to_string() })
            );
        }

        // 区分重下逻辑：URL (HTTP/Magnet) vs Local Torrent File
        let result = if task.url.starts_with("file://") {
            let path = task.url.trim_start_matches("file://").to_string();
            // 构建基础 Config 传递通用设置
            let base_cfg = crate::core::types::DownloadConfig {
                urls: vec![],
                save_path: save_path_opt,
                filename: filename_opt,
                user_agent: ua_opt,
                referer: referer_opt,
                headers: if task.headers.is_empty() { None } else { Some(task.headers.join("; ")) },
                proxy: proxy_opt,
                max_download_limit: limit_opt,
                torrent_config: None, // 内部调用不嵌套
            };
            super::add::add_torrent_task_inner(
                state,
                path,
                task.select_file.clone(),
                task.trackers.clone(),
                &base_cfg
            ).await
        } else {
            aria2_client::add_uri(vec![task.url.clone()], Some(options)).await
        };

        match result {
            Ok(new_gid) => {
                crate::app_info!(
                    "Core::TaskControl",
                    "smart_resume_readd_succeeded",
                    json!({ "old_gid": gid.clone(), "new_gid": new_gid.clone() })
                );

                let removed = state.remove_task(&gid);
                crate::app_info!(
                    "Core::TaskControl",
                    "smart_resume_old_task_removed",
                    json!({ "gid": gid.clone(), "removed": removed })
                );

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
                crate::app_error!(
                    "Core::TaskControl",
                    "smart_resume_readd_failed",
                    json!({ "gid": gid.clone(), "error": add_err.to_string() })
                );
                Err(AppError::aria2(format!("智能恢复失败: {}", add_err)))
            }
        }
    } else {
        Err(AppError::task_not_found(gid))
    }
}
