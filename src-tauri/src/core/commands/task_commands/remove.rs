use crate::aria2::client as aria2_client;
use crate::core::error::{AppError, AppResult};
use crate::core::store::TaskStore;
use crate::utils;
use futures::future::join_all;
use serde_json::json;
use tauri::AppHandle;

fn log_batch_errors(scope: &str, gids: &[String], results: Vec<Result<String, AppError>>) {
    let mut failed: Vec<String> = Vec::new();

    for (idx, result) in results.into_iter().enumerate() {
        if let Err(error) = result {
            let gid = gids.get(idx).cloned().unwrap_or_else(|| "unknown".to_string());
            failed.push(gid.clone());
            crate::app_warn!(
                "Core::TaskRemove",
                scope,
                json!({ "gid": gid, "error": error.to_string() })
            );
        }
    }

    if !failed.is_empty() {
        crate::app_warn!(
            "Core::TaskRemove",
            "batch_operation_partial_failure",
            json!({ "scope": scope, "failed_gids": failed })
        );
    }
}

fn is_not_found(err: &AppError) -> bool {
    err.is_aria2_not_found()
}

fn delete_task_files(save_path: &str, filename: &str) -> AppResult<()> {
    if !utils::is_safe_filename(filename) {
        return Err(AppError::validation(format!("非法文件名: {filename}")));
    }

    let resolved_save_path = utils::resolve_path(save_path);
    let save_dir = std::path::PathBuf::from(resolved_save_path);

    if !save_dir.exists() {
        return Err(AppError::validation(format!(
            "保存目录不存在: {}",
            save_dir.to_string_lossy()
        )));
    }

    let save_dir_abs = std::fs::canonicalize(&save_dir).map_err(|e| AppError::Fs(e.to_string()))?;
    let target = save_dir_abs.join(filename);
    if !target.starts_with(&save_dir_abs) {
        return Err(AppError::validation(format!(
            "非法目标路径: {}",
            target.to_string_lossy()
        )));
    }

    if target.exists() {
        std::fs::remove_file(&target).map_err(|e| AppError::Fs(e.to_string()))?;
    }

    let aria2_file_path = std::path::PathBuf::from(format!("{}.aria2", target.to_string_lossy()));
    if aria2_file_path.exists() {
        let _ = std::fs::remove_file(aria2_file_path);
    }

    Ok(())
}

#[tauri::command]
pub async fn remove_tasks(
    _app: AppHandle,
    state: tauri::State<'_, TaskStore>,
    gids: Vec<String>,
    delete_file: bool,
) -> AppResult<()> {
    let tasks_info: Vec<_> = gids.iter().filter_map(|gid| state.get_task(gid)).collect();

    let active_gids: Vec<String> = tasks_info
        .iter()
        .filter(|task| task.state.is_active())
        .map(|task| task.gid.clone())
        .collect();

    let active_futures: Vec<_> = active_gids
        .iter()
        .map(|gid| aria2_client::remove(gid.clone()))
        .collect();
    let active_results = join_all(active_futures).await;
    log_batch_errors("remove_active_failed", &active_gids, active_results.clone());

    let purge_futures: Vec<_> = gids
        .iter()
        .map(|gid| aria2_client::purge(gid.clone()))
        .collect();
    let purge_results = join_all(purge_futures).await;
    log_batch_errors("purge_failed", &gids, purge_results.clone());

    let mut failed = Vec::new();
    for (idx, result) in active_results.into_iter().enumerate() {
        if let Err(error) = result {
            if !is_not_found(&error) {
                let gid = active_gids.get(idx).cloned().unwrap_or_else(|| "unknown".to_string());
                failed.push(gid);
            }
        }
    }

    for (idx, result) in purge_results.into_iter().enumerate() {
        if let Err(error) = result {
            if !is_not_found(&error) {
                let gid = gids.get(idx).cloned().unwrap_or_else(|| "unknown".to_string());
                failed.push(gid);
            }
        }
    }

    if !failed.is_empty() {
        failed.sort();
        failed.dedup();
        return Err(AppError::aria2(format!(
            "批量删除失败，未完成任务: {}",
            failed.join(",")
        )));
    }

    state.remove_tasks_batch(&gids);
    state.save();

    if delete_file {
        for task in &tasks_info {
            delete_task_files(&task.save_path, &task.filename)?;
        }
    }

    Ok(())
}

#[tauri::command]
pub async fn remove_task_record(
    _app: AppHandle,
    state: tauri::State<'_, TaskStore>,
    gid: String,
    delete_file: bool,
) -> AppResult<()> {
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

async fn remove_task_inner(state: &TaskStore, gid: String, delete_file: bool) -> AppResult<()> {
    let task_opt = state.get_task(&gid);

    let is_active = task_opt.as_ref().is_some_and(|t| t.state.is_active());

    if is_active {
        if let Err(error) = aria2_client::remove(gid.clone()).await {
            if !is_not_found(&error) {
                return Err(error);
            }
        }
        if let Err(error) = aria2_client::purge(gid.clone()).await {
            if !is_not_found(&error) {
                return Err(error);
            }
        }
    } else {
        if let Err(error) = aria2_client::purge(gid.clone()).await {
            if !is_not_found(&error) {
                return Err(error);
            }
        }
    }

    state.remove_task(&gid);

    if delete_file {
        if let Some(task) = task_opt {
            delete_task_files(&task.save_path, &task.filename)?;
        }
    }

    Ok(())
}
