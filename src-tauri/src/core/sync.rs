use crate::aria2::client::{self as aria2_client, Aria2Task};
use crate::core::error::{AppError, AppResult};
use crate::core::store::TaskStore;
use crate::core::types::TaskState;
use std::collections::HashMap;
use std::sync::atomic::Ordering;
use tauri::AppHandle;

// 连接日志的状态跟踪
// true = 已连接, false = 已断开
static LAST_CONNECTION_STATUS: std::sync::atomic::AtomicBool =
    std::sync::atomic::AtomicBool::new(true);

/// 计算剩余秒数（纯数值，不含格式化）
fn calculate_remaining_secs(raw_speed: u64, total: u64, completed: u64) -> u64 {
    if raw_speed == 0 || total <= completed {
        return 0;
    }
    (total - completed) / raw_speed
}

// 前端 DTO - 只传输原始数值，格式化由前端负责
#[derive(Debug, Clone, serde::Serialize)]
pub struct FrontendTask {
    pub id: String,
    pub filename: String,
    pub url: String,
    pub progress: f64,
    pub speed: u64,
    pub completed: u64,
    pub total: u64,
    #[serde(rename = "remainingSecs")]
    pub remaining_secs: u64,
    pub state: String,
    #[serde(rename = "addedAt")]
    pub added_at: String,
    #[serde(rename = "savePath")]
    pub save_path: String,
    #[serde(rename = "errorMessage")]
    pub error_message: String,
    #[serde(rename = "userAgent")]
    pub user_agent: String,
    pub referer: String,
    pub proxy: String,
    pub headers: Vec<String>,
    #[serde(rename = "maxDownloadLimit")]
    pub max_download_limit: String,
    #[serde(rename = "completedAt")]
    pub completed_at: Option<String>,
}

pub async fn sync_tasks(state: &TaskStore, app_handle: &AppHandle) -> AppResult<Vec<FrontendTask>> {
    // 1. 从 Store 获取所有任务
    let mut store_tasks = state.get_all();

    // 2. 从 Aria2 获取（失败时快速返回，避免冗余日志）
    let all_tasks = match aria2_client::get_all_tasks().await {
        Ok(t) => {
            // 检查是否刚从故障中恢复
            if !LAST_CONNECTION_STATUS.load(Ordering::Relaxed) {
                log::info!("Aria2 连接已恢复。");
                LAST_CONNECTION_STATUS.store(true, Ordering::Relaxed);
            }
            t
        }
        Err(e) => {
            // 检查这是否是新的故障
            if LAST_CONNECTION_STATUS.load(Ordering::Relaxed) {
                log::warn!("Aria2 连接丢失: {}。正在进入静默模式。", e);
                LAST_CONNECTION_STATUS.store(false, Ordering::Relaxed);
            }
            // 如果已经是 false，则保持沉默（静默模式）

            return Err(AppError::aria2(format!("无法连接到 Aria2: {}", e)));
        }
    };

    // 3. 创建 GID -> Aria2Task 的映射以方便查找
    let mut aria2_map: HashMap<String, Aria2Task> = HashMap::new();
    for t in all_tasks {
        aria2_map.insert(t.gid.clone(), t);
    }

    let mut result: Vec<FrontendTask> = Vec::new();
    let mut total_dl = 0u64;
    let total_ul = 0u64;

    // 跟踪是否需要将更改保存到磁盘
    let mut dirty = false;

    // 4. 同步逻辑 & 构造视图模型
    for task in store_tasks.iter_mut() {
        let aria_task = aria2_map.get(&task.gid);

        let mut mapped_state = if let Some(at) = aria_task {
            TaskState::from_aria2_status(&at.status)
        } else {
            task.state
        };

        // 终态时的文件存在性检查
        // "missing" 只适用于已完成但文件被删除的情况
        // "error" 状态保持不变（下载失败本来就没有文件）
        if mapped_state == TaskState::Complete {
            let full_path = std::path::Path::new(&task.save_path).join(&task.filename);
            if !full_path.exists() {
                mapped_state = TaskState::Missing;
            }
        }

        // 如果状态发生变化，同步回 Store
        if task.state != mapped_state {
            let old_state = task.state;
            task.state = mapped_state;
            dirty = true;

            // 记录进入终态或暂停的时间
            if task.state.is_terminal() || task.state == TaskState::Paused {
                // 如果是第一次进入该状态（针对 Pause/Error 等可重复进入的状态），或者是 Complete 状态
                if task.completed_at.is_none() || old_state != TaskState::Complete {
                    task.completed_at = Some(chrono::Local::now().to_rfc3339());
                }
            } else if task.state == TaskState::Active || task.state == TaskState::Waiting {
                // 恢复活跃状态时，清除结束时间
                task.completed_at = None;
            }
        }

        // 如果存在 aria2 数据，则同步其他字段
        if let Some(at) = aria_task {
            if task.completed_length != at.completed_length {
                task.completed_length = at.completed_length.clone();
                dirty = true;
            }
            task.total_length = at.total_length.clone();
            task.download_speed = at.download_speed.clone();

            // 同步特定字段
            if let Some(msg) = &at.error_message {
                if task.error_message != *msg {
                    task.error_message = msg.clone();
                    dirty = true;
                }
            }

            // 始终从 Aria2 同步文件名以处理自动重命名（例如 file.1.mp4）
            if let Some(file) = at.files.get(0) {
                if !file.path.is_empty() {
                    let path = std::path::Path::new(&file.path);
                    if let Some(name) = path.file_name() {
                        if let Some(name_str) = name.to_str() {
                            if task.filename != name_str {
                                task.filename = name_str.to_string();
                                dirty = true;
                            }
                        }
                    }
                }
            }
        } else if mapped_state != TaskState::Missing
            && mapped_state != TaskState::Error
            && mapped_state != TaskState::Complete
            && mapped_state != TaskState::Removed
        {
            match aria2_client::tell_status(
                task.gid.clone(),
                vec![
                    "gid",
                    "status",
                    "totalLength",
                    "completedLength",
                    "downloadSpeed",
                    "uploadLength",
                    "uploadSpeed",
                    "files",
                    "errorMessage",
                ],
            )
            .await
            {
                Ok(aria_task) => {
                    task.state = TaskState::from_aria2_status(&aria_task.status);
                    task.total_length = aria_task.total_length.clone();
                    task.completed_length = aria_task.completed_length.clone();
                    task.download_speed = aria_task.download_speed.clone();
                    if let Some(msg) = &aria_task.error_message {
                        task.error_message = msg.clone();
                    }
                    dirty = true;

                    if let Some(file) = aria_task.files.get(0) {
                        if !file.path.is_empty() {
                            let path = std::path::Path::new(&file.path);
                            if let Some(name) = path.file_name() {
                                if let Some(name_str) = name.to_str() {
                                    if task.filename != name_str {
                                        task.filename = name_str.to_string();
                                    }
                                }
                            }
                        }
                    }
                }
                Err(e) => {
                    let _lower_msg = e.to_string().to_lowercase();
                    {
                        task.state = TaskState::Error;
                        task.download_speed = "0".to_string();
                        dirty = true;
                    }
                }
            }
        }

        let total = task.total_length.parse::<u64>().unwrap_or(0);
        let completed = task.completed_length.parse::<u64>().unwrap_or(0);
        let raw_speed = task.download_speed.parse::<u64>().unwrap_or(0);

        let remaining_secs = calculate_remaining_secs(raw_speed, total, completed);

        let progress = if total > 0 {
            (completed as f64 / total as f64) * 100.0
        } else {
            0.0
        };

        if task.state == TaskState::Active {
            total_dl += raw_speed;
        }

        result.push(FrontendTask {
            id: task.gid.clone(),
            filename: task.filename.clone(),
            url: task.url.clone(),
            progress,
            speed: raw_speed,
            completed,
            total,
            remaining_secs,
            state: task.state.to_string(),
            added_at: task.added_at.clone(),
            save_path: task.save_path.clone(),
            error_message: task.error_message.clone(),
            user_agent: task.user_agent.clone(),
            referer: task.referer.clone(),
            proxy: task.proxy.clone(),
            headers: task.headers.clone(),
            max_download_limit: task.max_download_limit.clone(),
            completed_at: task.completed_at.clone(),
        });
    }

    // 5. 清理孤儿任务（Aria2 中存在但 Store 中不存在的任务）
    // 创建一个 Store GID 的集合以方便查找
    let store_gids: std::collections::HashSet<String> =
        store_tasks.iter().map(|t| t.gid.clone()).collect();

    for (gid, _) in aria2_map.iter() {
        if !store_gids.contains(gid) {
            log::warn!("同步：在 Aria2 中发现孤儿任务: {}。正在清理...", gid);
            // 生成清理任务，不阻塞同步过程
            let gid_clone = gid.clone();
            tauri::async_runtime::spawn(async move {
                let _ = aria2_client::remove(gid_clone.clone()).await;
                let _ = aria2_client::purge(gid_clone).await;
            });
        }
    }

    // 批量提交更改
    if dirty {
        state.update_all(store_tasks);
    }

    // 更新任务栏图标
    let _ =
        crate::ui::tray::update_tray_icon_with_speed(app_handle.clone(), total_dl, total_ul).await;

    Ok(result)
}

pub fn start_background_sync(app_handle: AppHandle) {
    use tauri::Emitter;
    use tauri::Manager;

    tauri::async_runtime::spawn(async move {
        // 启动宽限期：等待 Sidecar 完全初始化（绑定端口）
        tokio::time::sleep(std::time::Duration::from_secs(1)).await;

        loop {
            let state = app_handle.state::<crate::core::store::TaskStore>();
            let mut has_active_tasks = false;

            match sync_tasks(&state, &app_handle).await {
                Ok(tasks) => {
                    // 检查是否有活跃任务（用于自适应轮询）
                    for task in &tasks {
                        let task_state = TaskState::from(task.state.as_str());
                        if task_state.is_active() {
                            has_active_tasks = true;
                            break;
                        }
                    }

                    if let Err(e) = app_handle.emit("tasks-update", tasks) {
                        log::warn!("发送 tasks-update 事件失败: {}", e);
                    }
                }
                Err(e) => {
                    log::debug!("后台同步失败: {}", e);
                    has_active_tasks = false;
                }
            }

            // 自适应休眠
            let sleep_duration = if has_active_tasks {
                std::time::Duration::from_millis(200)
            } else {
                std::time::Duration::from_secs(2)
            };

            tokio::time::sleep(sleep_duration).await;
        }
    });
}
