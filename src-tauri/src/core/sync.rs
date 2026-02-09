use crate::aria2::client::{self as aria2_client, Aria2Task};
use crate::core::store::TaskStore;
use std::collections::HashMap;
use std::sync::atomic::Ordering;
use tauri::AppHandle;

// 连接日志的状态跟踪
// true = 已连接, false = 已断开
static LAST_CONNECTION_STATUS: std::sync::atomic::AtomicBool =
    std::sync::atomic::AtomicBool::new(true);

// EMA & 频率跟踪器: GID -> (EMA_速度, 上次更新时间戳_秒, 上次剩余时间字符串)
static SPEED_HISTORY: std::sync::OnceLock<std::sync::Mutex<HashMap<String, (f64, u64, String)>>> =
    std::sync::OnceLock::new();

/// 计算剩余时间（带 EMA 平滑）
/// 返回格式化后的剩余时间字符串
fn calculate_eta(
    gid: &str,
    state: &str,
    raw_speed: u64,
    total: u64,
    completed: u64,
) -> String {
    let history_map = SPEED_HISTORY.get_or_init(|| std::sync::Mutex::new(HashMap::new()));
    let mut history = history_map.lock().unwrap();
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs();

    let (ema_speed, last_update, last_remaining) =
        history
            .entry(gid.to_string())
            .or_insert((raw_speed as f64, 0, String::new()));

    // 更新 EMA 速度
    if state == "downloading" {
        *ema_speed = (raw_speed as f64 * 0.2) + (*ema_speed * 0.8);
    } else {
        *ema_speed = 0.0;
    }

    // 计算剩余时间
    if state == "downloading" && *ema_speed > 0.1 && total > completed {
        if now > *last_update {
            let seconds = ((total - completed) as f64 / *ema_speed) as u64;
            let new_remaining = crate::utils::format_duration(seconds);
            *last_update = now;
            *last_remaining = new_remaining.clone();
            new_remaining
        } else {
            last_remaining.clone()
        }
    } else {
        *last_update = 0;
        *last_remaining = String::new();
        String::new()
    }
}


// 前端 DTO (视图模型) - 从 commands.rs 移至此处
#[derive(Debug, Clone, serde::Serialize)]
pub struct FrontendTask {
    pub id: String,
    pub filename: String,
    pub url: String,
    pub progress: f64,
    pub speed: String,
    pub speed_u64: u64, // 用于统计的原始值
    pub downloaded: String,
    pub downloaded_u64: u64, // 原始值
    pub total: String,
    pub total_u64: u64, // 原始值
    pub remaining: String,
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
}

pub async fn sync_tasks(
    state: &TaskStore,
    app_handle: &AppHandle,
) -> Result<Vec<FrontendTask>, String> {
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

            return Err(format!("无法连接到 Aria2: {}", e));
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
        
        let raw_status = if let Some(at) = aria_task {
            at.status.clone()
        } else {
            task.state.clone()
        };

        // 确定映射后的状态
        let mut mapped_state = crate::utils::map_status(&raw_status);

        // 终态时的文件存在性检查
        // "missing" 只适用于已完成但文件被删除的情况
        // "error" 状态保持不变（下载失败本来就没有文件）
        if mapped_state == "completed" {
            let full_path = std::path::Path::new(&task.save_path).join(&task.filename);
            if !full_path.exists() {
                mapped_state = "missing".to_string();
            }
        }

        // 如果状态发生变化，同步回 Store
        if task.state != mapped_state {
            task.state = mapped_state.clone();
            dirty = true;
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
        } else if mapped_state != "missing"
            && mapped_state != "error"
            && mapped_state != "completed"
            && mapped_state != "cancelled"
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
                    task.state = aria_task.status.clone();
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
                    let lower_msg = e.to_lowercase();
                    if lower_msg.contains("not found")
                        || lower_msg.contains("error 1")
                        || lower_msg.contains("http error")
                    {
                        task.state = "error".to_string();
                        task.download_speed = "0".to_string();
                        dirty = true;
                    }
                }
            }
        }

        let total = task.total_length.parse::<u64>().unwrap_or(0);
        let completed = task.completed_length.parse::<u64>().unwrap_or(0);
        let raw_speed = task.download_speed.parse::<u64>().unwrap_or(0);

        // 使用辅助函数计算剩余时间
        let current_remaining = calculate_eta(&task.gid, &task.state, raw_speed, total, completed);

        let progress = if total > 0 {
            (completed as f64 / total as f64) * 100.0
        } else {
            0.0
        };

        if task.state == "downloading" {
            total_dl += raw_speed;
        }

        result.push(FrontendTask {
            id: task.gid.clone(),
            filename: task.filename.clone(),
            url: task.url.clone(),
            progress,
            speed: crate::utils::format_speed(raw_speed),
            speed_u64: raw_speed,
            downloaded: crate::utils::format_size(completed),
            downloaded_u64: completed,
            total: crate::utils::format_size(total),
            total_u64: total,
            remaining: current_remaining,
            state: task.state.clone(), // 使用持久化状态！
            added_at: task.added_at.clone(),
            save_path: task.save_path.clone(),
            error_message: task.error_message.clone(),
            user_agent: task.user_agent.clone(),
            referer: task.referer.clone(),
            proxy: task.proxy.clone(),
            headers: task.headers.clone(),
            max_download_limit: task.max_download_limit.clone(),
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

    // 结果排序：分数降序 -> 添加时间降序
    result.sort_by(|a, b| {
        let score_a = crate::utils::get_state_score(&a.state);
        let score_b = crate::utils::get_state_score(&b.state);

        if score_a != score_b {
            return score_b.cmp(&score_a); // 分数高的排在前面
        }

        // 如果分数相等，按时间排序（最新的排在前面）
        b.added_at.cmp(&a.added_at)
    });

    // 更新任务栏图标
    let _ =
        crate::ui::tray::update_tray_icon_with_speed(app_handle.clone(), total_dl, total_ul).await;

    Ok(result)
}

pub fn start_background_sync(app_handle: AppHandle) {
    use tauri::Emitter;
    use tauri::Manager;

    // 通知状态跟踪器: GID -> State
    let mut previous_states: std::collections::HashMap<String, String> =
        std::collections::HashMap::new();

    tauri::async_runtime::spawn(async move {
        // 启动宽限期：等待 Sidecar 完全初始化（绑定端口）
        tokio::time::sleep(std::time::Duration::from_secs(1)).await;

        loop {
            // 获取状态（线程安全获取）
            let state = app_handle.state::<crate::core::store::TaskStore>();
            let mut has_active_tasks = false;

            // 运行同步
            match sync_tasks(&state, &app_handle).await {
                Ok(tasks) => {
                    // 检查完成情况和活跃状态
                    for task in &tasks {
                        // 自适应轮询检查
                        if task.state == "active"
                            || task.state == "waiting"
                            || task.state == "downloading"
                        {
                            has_active_tasks = true;
                        }

                        let prev = previous_states
                            .get(&task.id)
                            .cloned()
                            .unwrap_or("unknown".to_string());

                        if task.state == "completed" && prev != "completed" && prev != "unknown" {
                            // 检测到状态转变！
                            log::info!("任务已完成: {}", task.filename);
                            if let Err(e) = app_handle.emit("task-completed", task.clone()) {
                                log::warn!("发送 task-completed 事件失败: {}", e);
                            }
                        }

                        previous_states.insert(task.id.clone(), task.state.clone());
                    }

                    // 发送事件
                    if let Err(e) = app_handle.emit("tasks-update", tasks) {
                        log::warn!("发送 tasks-update 事件失败: {}", e);
                    }
                }
                Err(e) => {
                    // 同步模块会处理静默模式日志，所以我们这里不需要记录 ERROR
                    // 除非需要调试信息。
                    log::debug!("后台同步失败: {}", e);
                    // 发生错误时，假设没有活跃任务以退避，或者保持标准频率？
                    // 让我们假设在错误时退避。
                    has_active_tasks = false;
                }
            }

            // 自适应休眠
            let sleep_duration = if has_active_tasks {
                std::time::Duration::from_millis(200) // 超快模式（实时）
            } else {
                std::time::Duration::from_secs(2) // 空闲模式
            };

            tokio::time::sleep(sleep_duration).await;
        }
    });
}
