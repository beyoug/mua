use crate::aria2::client::{self as aria2_client, Aria2Task};
use crate::core::store::TaskStore;
use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, Ordering};
use tauri::AppHandle;

// Status tracker for connection logging
// true = connected, false = disconnected
static LAST_CONNECTION_STATUS: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(true);

// EMA & Frequency Tracker: GID -> (EMA_Speed, Last_Update_Timestamp_Secs, Last_Remaining_String)
static SPEED_HISTORY: std::sync::OnceLock<std::sync::Mutex<HashMap<String, (f64, u64, String)>>> =
    std::sync::OnceLock::new();

// Frontend DTO (View Model) - Moved from commands.rs
#[derive(Debug, Clone, serde::Serialize)]
pub struct FrontendTask {
    pub id: String,
    pub filename: String,
    pub url: String,
    pub progress: f64,
    pub speed: String,
    pub speed_u64: u64, // Raw value for stats
    pub downloaded: String,
    pub downloaded_u64: u64, // Raw value
    pub total: String,
    pub total_u64: u64, // Raw value
    pub remaining: String,
    pub state: String,
    #[serde(rename = "addedAt")]
    pub added_at: String,
    #[serde(rename = "savePath")]
    pub save_path: String,
    #[serde(rename = "errorMessage")]
    pub error_message: String,
}

pub async fn sync_tasks(
    state: &TaskStore,
    app_handle: &AppHandle,
) -> Result<Vec<FrontendTask>, String> {
    // 1. Get all tasks from Store
    let mut store_tasks = state.get_all();

    // 2. Fetch from Aria2 (Fail Fast with Quiet Mode)
    let all_tasks = match aria2_client::get_all_tasks().await {
        Ok(t) => {
            // Check if we just recovered from a failure
            if !LAST_CONNECTION_STATUS.load(Ordering::Relaxed) {
                log::info!("Aria2 connection restored.");
                LAST_CONNECTION_STATUS.store(true, Ordering::Relaxed);
            }
            t
        }
        Err(e) => {
            // Check if this is a new failure
            if LAST_CONNECTION_STATUS.load(Ordering::Relaxed) {
                log::warn!("Aria2 connection lost: {}. Entering quiet mode.", e);
                LAST_CONNECTION_STATUS.store(false, Ordering::Relaxed);
            }
            // If already false, we stay silent (quiet mode)

            return Err(format!("Aria2 unreachable: {}", e));
        }
    };

    // 3. Create a Map of GID -> Aria2Task for easy lookup
    let mut aria2_map: HashMap<String, Aria2Task> = HashMap::new();
    for t in all_tasks {
        aria2_map.insert(t.gid.clone(), t);
    }

    let mut result: Vec<FrontendTask> = Vec::new();
    let mut total_dl = 0u64;
    let total_ul = 0u64;

    // Track if we need to save changes to disk
    let mut dirty = false;

    // 4. Sync Logic & View Model construction
    for task in store_tasks.iter_mut() {
        let raw_status = if let Some(aria_task) = aria2_map.get(&task.gid) {
            aria_task.status.clone()
        } else {
            task.state.clone()
        };

        // Determine mapped state
        let mut mapped_state = crate::utils::map_status(&raw_status);

        // Final File Presence Check for Terminal States
        if mapped_state == "completed" || mapped_state == "error" {
            let full_path = std::path::Path::new(&task.save_path).join(&task.filename);
            if !full_path.exists() {
                mapped_state = "missing".to_string();
            }
        }

        // Sync back to Store if changed
        if task.state != mapped_state {
            task.state = mapped_state.clone();
            dirty = true;
        }

        // Sync other fields if aria2 data exists
        if let Some(aria_task) = aria2_map.get(&task.gid) {
            if task.completed_length != aria_task.completed_length {
                task.completed_length = aria_task.completed_length.clone();
                dirty = true;
            }
            task.total_length = aria_task.total_length.clone();
            task.download_speed = aria_task.download_speed.clone();
            
            // Sync specific fields
            if let Some(msg) = &aria_task.error_message {
                if task.error_message != *msg {
                    task.error_message = msg.clone();
                    dirty = true;
                }
            }

            // Always sync filename from Aria2 to handle auto-renaming (e.g. file.1.mp4)
            if let Some(file) = aria_task.files.get(0) {
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
                    "dir",
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

        // EMA ... (existing EMA logic)
        let history_map = SPEED_HISTORY.get_or_init(|| std::sync::Mutex::new(HashMap::new()));
        let mut history = history_map.lock().unwrap();
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();

        let (ema_speed, last_update, last_remaining) = history
            .entry(task.gid.clone())
            .or_insert((raw_speed as f64, 0, "".to_string()));

        if task.state == "downloading" {
            *ema_speed = (raw_speed as f64 * 0.2) + (*ema_speed * 0.8);
        } else {
            *ema_speed = 0.0;
        }

        let current_remaining = if task.state == "downloading"
            && *ema_speed > 0.1
            && total > completed
        {
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
            *last_remaining = "".to_string();
            "".to_string()
        };

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
            state: task.state.clone(), // Use persisted state!
            added_at: task.added_at.clone(),
            save_path: task.save_path.clone(),
            error_message: task.error_message.clone(),
        });
    }

    // 5. Cleanup Orphans (Aria2 has it, but Store doesn't)
    // Create a set of Store GIDs for fast lookup
    let store_gids: std::collections::HashSet<String> =
        store_tasks.iter().map(|t| t.gid.clone()).collect();

    for (gid, _) in aria2_map.iter() {
        if !store_gids.contains(gid) {
            log::warn!("Sync: Found orphan task in Aria2: {}. Cleaning up...", gid);
            // Spawn cleanup to not block sync
            let gid_clone = gid.clone();
            tauri::async_runtime::spawn(async move {
                let _ = aria2_client::remove(gid_clone.clone()).await;
                let _ = aria2_client::purge(gid_clone).await;
            });
        }
    }

    // Batch Commit
    if dirty {
        state.update_all(store_tasks);
    }

    // Sort Result: Score Desc -> AddedAt Desc
    result.sort_by(|a, b| {
        let score_a = crate::utils::get_state_score(&a.state);
        let score_b = crate::utils::get_state_score(&b.state);

        if score_a != score_b {
            return score_b.cmp(&score_a); // Higher score first
        }

        // If scores are equal, sort by time (Newest first)
        b.added_at.cmp(&a.added_at)
    });

    // Update Tray
    let _ =
        crate::ui::tray::update_tray_icon_with_speed(app_handle.clone(), total_dl, total_ul).await;

    Ok(result)
}

pub fn start_background_sync(app_handle: AppHandle) {
    use tauri::Emitter;
    use tauri::Manager;

    // Notification State Tracker: GID -> State
    let mut previous_states: std::collections::HashMap<String, String> =
        std::collections::HashMap::new();

    tauri::async_runtime::spawn(async move {
        // Startup Grace Period: Wait for Sidecar to fully initialize (bind port)
        tokio::time::sleep(std::time::Duration::from_secs(1)).await;

        loop {
            // Get State (thread-safe retrieval)
            let state = app_handle.state::<crate::core::store::TaskStore>();
            let mut has_active_tasks = false;

            // Run Sync
            match sync_tasks(&state, &app_handle).await {
                Ok(tasks) => {
                    // Check for Completions & Activity
                    for task in &tasks {
                        // Adaptive Polling Check
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
                            // Transition detected!
                            log::info!("Task completed: {}", task.filename);
                            if let Err(e) = app_handle.emit("task-completed", task.clone()) {
                                log::warn!("Failed to emit task-completed: {}", e);
                            }
                        }

                        previous_states.insert(task.id.clone(), task.state.clone());
                    }

                    // Emit Event
                    if let Err(e) = app_handle.emit("tasks-update", tasks) {
                        log::warn!("Failed to emit tasks-update: {}", e);
                    }
                }
                Err(e) => {
                    // Sync module handles quiet mode logging, so we don't need to log ERROR here
                    // unless we want debug info.
                    log::debug!("Background sync failed: {}", e);
                    // On error, we assume no active tasks (to back off) or keep standard?
                    // Let's assume backoff on error.
                    has_active_tasks = false;
                }
            }

            // Adaptive Sleep
            let sleep_duration = if has_active_tasks {
                std::time::Duration::from_millis(200) // Ultra Fast Mode (Real-time)
            } else {
                std::time::Duration::from_secs(2) // Idle Mode
            };

            tokio::time::sleep(sleep_duration).await;
        }
    });
}
