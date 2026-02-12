use crate::aria2::client as aria2_client;
use crate::core::error::{AppError, AppResult};
use crate::core::store::TaskStore;
use crate::core::types::TaskState;
use crate::utils;
use serde_json::json;
use std::collections::{HashMap, HashSet};

use super::BatchCommandResult;

fn is_not_found_error(error: &AppError) -> bool {
    let message = error.to_string().to_lowercase();
    message.contains("not found") || message.contains("error 1")
}

#[tauri::command]
pub async fn remove_tasks(
    state: tauri::State<'_, TaskStore>,
    gids: Vec<String>,
    delete_file: bool,
) -> AppResult<BatchCommandResult> {
    let tasks_info: HashMap<String, _> = gids
        .iter()
        .filter_map(|gid| state.get_task(gid).map(|task| (gid.clone(), task)))
        .collect();

    let active_gids: Vec<String> = tasks_info
        .values()
        .filter(|task| task.state == TaskState::Active || task.state == TaskState::Waiting)
        .map(|task| task.gid.clone())
        .collect();

    let mut failed_gids = HashSet::new();

    for gid in &active_gids {
        if let Err(e) = aria2_client::remove(gid.clone()).await {
            if !is_not_found_error(&e) {
                failed_gids.insert(gid.clone());
                crate::app_warn!(
                    "Core::TaskRemove",
                    "batch_remove_rpc_failed",
                    json!({ "gid": gid, "error": e.to_string() })
                );
            }
        }
    }

    for gid in &gids {
        if let Err(e) = aria2_client::purge(gid.clone()).await {
            if !is_not_found_error(&e) {
                failed_gids.insert(gid.clone());
                crate::app_warn!(
                    "Core::TaskRemove",
                    "batch_purge_rpc_failed",
                    json!({ "gid": gid, "error": e.to_string() })
                );
            }
        }
    }

    let succeeded_gids: Vec<String> = gids
        .iter()
        .filter(|gid| !failed_gids.contains(*gid))
        .cloned()
        .collect();

    if !succeeded_gids.is_empty() {
        state.remove_tasks_batch(&succeeded_gids);
        state.save();
    }

    if delete_file {
        for gid in &succeeded_gids {
            let Some(task) = tasks_info.get(gid) else {
                continue;
            };

            let full_path = utils::get_full_path(&task.save_path, &task.filename);
            let resolved_path = utils::resolve_path(&full_path);
            if std::path::Path::new(&resolved_path).exists() {
                if let Err(e) = std::fs::remove_file(&resolved_path) {
                    crate::app_error!(
                        "Core::TaskRemove",
                        "file_delete_failed",
                        json!({ "path": resolved_path, "error": e.to_string() })
                    );
                }
            }
            let aria2_file_path = format!("{}.aria2", resolved_path);
            let _ = std::fs::remove_file(&aria2_file_path);
        }
    }

    let failed_gids: Vec<String> = gids
        .iter()
        .filter(|gid| failed_gids.contains(*gid))
        .cloned()
        .collect();

    Ok(BatchCommandResult {
        requested: gids.len(),
        partial: !failed_gids.is_empty(),
        succeeded_gids,
        failed_gids,
    })
}

#[tauri::command]
pub async fn remove_task_record(
    state: tauri::State<'_, TaskStore>,
    gid: String,
    delete_file: bool,
) -> AppResult<String> {
    remove_task_inner(&state, gid, delete_file).await
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

async fn remove_task_inner(state: &TaskStore, gid: String, delete_file: bool) -> AppResult<String> {
    let task_opt = state.get_task(&gid);

    let is_active = task_opt.as_ref().is_some_and(|t| t.state.is_active());

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
                    Ok(_) => crate::app_info!(
                        "Core::TaskRemove",
                        "file_deleted",
                        json!({ "path": resolved_path })
                    ),
                    Err(e) => crate::app_error!(
                        "Core::TaskRemove",
                        "file_delete_failed",
                        json!({ "path": resolved_path, "error": e.to_string() })
                    ),
                }
            } else {
                crate::app_warn!(
                    "Core::TaskRemove",
                    "file_not_found_on_delete",
                    json!({ "path": resolved_path })
                );
            }

            let aria2_file_path = format!("{}.aria2", resolved_path);
            if std::path::Path::new(&aria2_file_path).exists() {
                let _ = std::fs::remove_file(&aria2_file_path);
            }
        }
    }

    Ok("OK".to_string())
}
