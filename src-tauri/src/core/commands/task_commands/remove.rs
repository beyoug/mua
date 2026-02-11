use crate::aria2::client as aria2_client;
use crate::core::error::{AppError, AppResult};
use crate::core::store::TaskStore;
use crate::utils;
use futures::future::join_all;

#[tauri::command]
pub async fn remove_tasks(
    state: tauri::State<'_, TaskStore>,
    gids: Vec<String>,
    delete_file: bool,
) -> AppResult<String> {
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
