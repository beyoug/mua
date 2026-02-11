use crate::aria2::client as aria2_client;
use crate::core::error::{AppError, AppResult};
use crate::core::store::{PersistedTask, TaskStore};
use crate::core::types::TaskState;
use crate::utils;
use chrono::Local;
use futures::future::join_all;

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

async fn smart_resume_task(state: &TaskStore, gid: String) -> AppResult<String> {
    log::info!("正在为 GID {} 尝试智能恢复 (Smart Resume)", gid);

    if let Some(task) = state.get_task(&gid) {
        log::info!("正在重新提交任务: {}", task.filename);

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
                log::info!("智能恢复成功。旧 GID: {}, 新 GID: {}", gid, new_gid);

                let removed = state.remove_task(&gid);
                log::info!("智能恢复：已移除旧任务 {}？{}", gid, removed);

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
