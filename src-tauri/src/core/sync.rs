use crate::aria2::client::{self as aria2_client, Aria2Task};
use crate::core::store::TaskStore;
use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, Ordering};
use tauri::AppHandle;

// Status tracker for connection logging
// true = connected, false = disconnected
static LAST_CONNECTION_STATUS: AtomicBool = AtomicBool::new(true);

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
    pub added_at: String,
    pub save_path: String,
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
        // Sync with Aria2 if available
        if let Some(aria_task) = aria2_map.get(&task.gid) {
            // Detect changes
            if task.state != aria_task.status || task.completed_length != aria_task.completed_length
            {
                dirty = true;
            }

            task.state = aria_task.status.clone();
            task.total_length = aria_task.total_length.clone();
            task.completed_length = aria_task.completed_length.clone();
            task.download_speed = aria_task.download_speed.clone();

            // Try to resolve filename if empty
            if task.filename.is_empty() {
                if let Some(file) = aria_task.files.get(0) {
                    if !file.path.is_empty() {
                        // Extract basename
                        let path = std::path::Path::new(&file.path);
                        if let Some(name) = path.file_name() {
                            if let Some(name_str) = name.to_str() {
                                task.filename = name_str.to_string();
                                dirty = true;
                            }
                        }
                    }
                }
            }
        } else {
            // Handle missing tasks (transition or validation)
            if task.state != "error"
                && task.state != "removed"
                && task.state != "completed"
                && task.state != "cancelled"
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
                    ],
                )
                .await
                {
                    Ok(aria_task) => {
                        task.state = aria_task.status.clone();
                        task.total_length = aria_task.total_length.clone();
                        task.completed_length = aria_task.completed_length.clone();
                        task.download_speed = aria_task.download_speed.clone();
                        dirty = true;

                        // Try to resolve filename if empty
                        if task.filename.is_empty() {
                            if let Some(file) = aria_task.files.get(0) {
                                if !file.path.is_empty() {
                                    let path = std::path::Path::new(&file.path);
                                    if let Some(name) = path.file_name() {
                                        if let Some(name_str) = name.to_str() {
                                            task.filename = name_str.to_string();
                                        }
                                    }
                                }
                            }
                        }
                    }
                    Err(e) => {
                        // Only mark as error if explicitly not found (Error 1) OR HTTP Error (Aria2 rejected request)
                        let lower_msg = e.to_lowercase();
                        if lower_msg.contains("not found")
                            || lower_msg.contains("error 1")
                            || lower_msg.contains("http error")
                        // 400 Bad Request, etc.
                        {
                            task.state = "error".to_string();
                            task.download_speed = "0".to_string();
                            dirty = true;
                        } else {
                            log::warn!(
                                "Failed to fetch status for {}: {}. Keeping last known state.",
                                task.gid,
                                e
                            );
                        }
                    }
                }
            }
        }

        // View Model Calculation
        let total = task.total_length.parse::<u64>().unwrap_or(0);
        let completed = task.completed_length.parse::<u64>().unwrap_or(0);
        let speed = task.download_speed.parse::<u64>().unwrap_or(0);

        let progress = if total > 0 {
            (completed as f64 / total as f64) * 100.0
        } else {
            0.0
        };

        let remaining = if speed > 0 && total > completed {
            let seconds = (total - completed) / speed;
            crate::utils::format_duration(seconds)
        } else {
            "".to_string()
        };

        // Tray Icon Stats
        if task.state == "active" {
            total_dl += speed;
        }

        result.push(FrontendTask {
            id: task.gid.clone(),
            filename: task.filename.clone(),
            url: task.url.clone(),
            progress,
            speed: crate::utils::format_bytes(speed) + "/s",
            speed_u64: speed,
            downloaded: crate::utils::format_bytes(completed),
            downloaded_u64: completed,
            total: crate::utils::format_bytes(total),
            total_u64: total,
            remaining,
            state: crate::utils::map_status(&task.state),
            added_at: task.added_at.clone(),
            save_path: task.save_path.clone(),
        });
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
