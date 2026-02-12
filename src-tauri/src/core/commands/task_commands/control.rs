use crate::aria2::client as aria2_client;
use crate::core::error::{AppError, AppResult};
use crate::core::store::{PersistedTask, TaskStore};
use crate::core::types::TaskState;
use crate::utils;
use chrono::Local;
use serde_json::json;

use super::BatchCommandResult;

fn is_not_found_error_text(message: &str) -> bool {
    let lower = message.to_lowercase();
    lower.contains("not found") || lower.contains("error 1")
}

#[tauri::command]
pub async fn pause_task(state: tauri::State<'_, TaskStore>, gid: String) -> AppResult<String> {
    let result = aria2_client::pause(gid.clone()).await?;
    state.update_task_state(&gid, TaskState::Paused);
    Ok(result)
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
            let msg = e.to_string().to_lowercase();
            if msg.contains("not found") || msg.contains("error 1") {
                smart_resume_task(&state, gid).await
            } else {
                Err(e)
            }
        }
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
) -> AppResult<BatchCommandResult> {
    let mut succeeded_gids: Vec<String> = Vec::new();
    let mut failed_gids: Vec<String> = Vec::new();

    for gid in &gids {
        match aria2_client::remove(gid.clone()).await {
            Ok(_) => succeeded_gids.push(gid.clone()),
            Err(e) => {
                if is_not_found_error_text(&e.to_string()) {
                    succeeded_gids.push(gid.clone());
                } else {
                    failed_gids.push(gid.clone());
                    crate::app_warn!(
                        "Core::TaskControl",
                        "batch_cancel_rpc_failed",
                        json!({ "gid": gid, "error": e.to_string() })
                    );
                }
            }
        }
    }

    if !succeeded_gids.is_empty() {
        state.update_batch_state(&succeeded_gids, TaskState::Removed);
        state.save();
    }

    Ok(BatchCommandResult {
        requested: gids.len(),
        partial: !failed_gids.is_empty(),
        succeeded_gids,
        failed_gids,
    })
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
            save_path_opt,
            filename_opt,
            ua_opt,
            referer_opt,
            headers_str,
            proxy_opt,
            limit_opt,
        );

        let _ = aria2_client::purge(gid.clone()).await;

        match aria2_client::add_uri(vec![task.url.clone()], Some(options)).await {
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
