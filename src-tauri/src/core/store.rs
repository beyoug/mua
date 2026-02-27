use crate::core::types::TaskState;
use chrono::Local;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::Mutex;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use tauri::{AppHandle, Manager};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PersistedTask {
    pub gid: String,
    pub filename: String,
    pub url: String,
    pub save_path: String,
    pub added_at: String,
    pub state: TaskState,
    pub total_length: String,
    pub completed_length: String,
    pub download_speed: String,
    #[serde(default)]
    pub completed_at: Option<String>,
    // Add other fields we want to persist if Aria2 loses them
    #[serde(default)]
    pub error_message: String,
    #[serde(default)]
    pub user_agent: String,
    #[serde(default)]
    pub referer: String,
    #[serde(default)]
    pub proxy: String,
    #[serde(default)]
    pub headers: Vec<String>,
    #[serde(default)]
    pub max_download_limit: String,
    // Torrent specific fields
    #[serde(default)]
    pub select_file: Option<String>,
    #[serde(default)]
    pub trackers: Option<String>,
}

impl PersistedTask {
    /// 状态迁移：并自动维护 completed_at 时间戳。
    /// 返回 true 表示状态发生了变化。
    pub fn transition_state(&mut self, new_state: TaskState) -> bool {
        if self.state == new_state {
            return false;
        }
        let old_state = self.state;
        self.state = new_state;

        if self.state.is_terminal() || self.state == TaskState::Paused {
            if self.completed_at.is_none() || old_state != TaskState::Complete {
                self.completed_at = Some(Local::now().to_rfc3339());
            }
        } else if self.state == TaskState::Active || self.state == TaskState::Waiting {
            self.completed_at = None;
        }

        true
    }
}

/// Minimum interval between save operations (ms)
const SAVE_DEBOUNCE_MS: u64 = 1500;

pub struct TaskStore {
    pub tasks: Mutex<HashMap<String, PersistedTask>>,
    file_path: Mutex<Option<PathBuf>>,
    last_save_time: AtomicU64,
    /// Trailing debounce: set to true when a save is skipped due to debounce
    trailing_pending: AtomicBool,
    /// Guard to prevent multiple trailing save threads
    trailing_scheduled: AtomicBool,
}

impl TaskStore {
    pub fn new() -> Self {
        Self {
            tasks: Mutex::new(HashMap::new()),
            file_path: Mutex::new(None),
            last_save_time: AtomicU64::new(0),
            trailing_pending: AtomicBool::new(false),
            trailing_scheduled: AtomicBool::new(false),
        }
    }

    pub fn init(&self, app: &AppHandle) {
        if let Ok(app_data_dir) = app.path().app_data_dir() {
            let path = app_data_dir.join("tasks.json");
            if let Ok(mut fp) = self.file_path.lock() {
                *fp = Some(path.clone());
            }

            // Load existing
            if path.exists() {
                if let Ok(content) = fs::read_to_string(&path) {
                    if let Ok(list) = serde_json::from_str::<Vec<PersistedTask>>(&content) {
                        if let Ok(mut tasks) = self.tasks.lock() {
                            for t in list {
                                tasks.insert(t.gid.clone(), t);
                            }
                        }
                    }
                }
            }
        }
    }

    pub fn save(&self) {
        // Debounce: skip if last save was too recent
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_millis() as u64)
            .unwrap_or(0);
        let last = self.last_save_time.load(Ordering::Relaxed);
        if now.saturating_sub(last) < SAVE_DEBOUNCE_MS {
            // Mark dirty so a trailing save will pick up the final state
            self.trailing_pending.store(true, Ordering::Release);

            // Schedule a deferred trailing write if not already scheduled
            if !self.trailing_scheduled.swap(true, Ordering::AcqRel) {
                // Snapshot data eagerly for the deferred write
                let snapshot = self.snapshot_for_write();
                if let Some((path, list)) = snapshot {
                    std::thread::spawn(move || {
                        std::thread::sleep(Duration::from_millis(SAVE_DEBOUNCE_MS));
                        Self::write_to_disk(path, list);
                    });
                }
            }
            return;
        }
        self.trailing_pending.store(false, Ordering::Release);
        self.trailing_scheduled.store(false, Ordering::Release);
        self.last_save_time.store(now, Ordering::Relaxed);
        self.save_inner();
    }

    /// Actual persistence logic (no debounce check)
    fn save_inner(&self) {
        if let Some((path, list)) = self.snapshot_for_write() {
            // Offload IO to thread
            std::thread::spawn(move || {
                Self::write_to_disk(path, list);
            });
        }
    }

    /// Snapshot current state for writing. Returns (path, sorted task list) or None.
    fn snapshot_for_write(&self) -> Option<(PathBuf, Vec<PersistedTask>)> {
        let path = self.file_path.lock().ok()?.clone()?;
        let mut list: Vec<PersistedTask> = self.tasks.lock().ok()?.values().cloned().collect();
        // Sort for stable JSON output
        list.sort_by(|a, b| b.added_at.cmp(&a.added_at));
        Some((path, list))
    }

    /// Write task list to disk (blocking, intended for background threads)
    fn write_to_disk(path: PathBuf, list: Vec<PersistedTask>) {
        if let Ok(json) = serde_json::to_string_pretty(&list) {
            if let Err(e) = crate::utils::atomic_write(&path, &json) {
                crate::app_error!(
                    "Core::Store",
                    "tasks_save_failed",
                    json!({ "path": path.to_string_lossy(), "error": e.to_string() })
                );
            }
        }
    }

    /// Force save without debounce (for critical operations like app exit)
    pub fn force_save(&self) {
        self.trailing_pending.store(false, Ordering::Release);
        self.last_save_time.store(0, Ordering::Relaxed);
        self.save_inner();
    }

    pub fn add_task(&self, task: PersistedTask) {
        if let Ok(mut tasks) = self.tasks.lock() {
            tasks.insert(task.gid.clone(), task);
        }
        self.save();
    }

    pub fn get_task(&self, gid: &str) -> Option<PersistedTask> {
        if let Ok(tasks) = self.tasks.lock() {
            return tasks.get(gid).cloned();
        }
        None
    }

    pub fn update_task_state(&self, gid: &str, state: TaskState) {
        if let Ok(mut tasks) = self.tasks.lock() {
            if let Some(t) = tasks.get_mut(gid) {
                t.transition_state(state);
            }
        }
        self.save();
    }

    pub fn update_filename(&self, gid: &str, filename: &str) {
        if let Ok(mut tasks) = self.tasks.lock() {
            if let Some(t) = tasks.get_mut(gid) {
                t.filename = filename.to_string();
            }
        }
        self.save();
    }

    // Batch update all tasks
    pub fn update_all(&self, updated_tasks: Vec<PersistedTask>) {
        if let Ok(mut tasks) = self.tasks.lock() {
            for task in updated_tasks {
                tasks.insert(task.gid.clone(), task);
            }
        }
        self.save();
    }

    // For sync: update full details from Aria2
    pub fn update_from_aria2(
        &self,
        gid: &str,
        state: TaskState,
        completed: &str,
        speed: &str,
        total: &str,
    ) {
        if let Ok(mut tasks) = self.tasks.lock() {
            if let Some(t) = tasks.get_mut(gid) {
                t.state = state;
                t.completed_length = completed.to_string();
                t.download_speed = speed.to_string();
                t.total_length = total.to_string();
            }
        }
        // Save is usually called after batch update to avoid IO spam
    }

    pub fn get_all(&self) -> Vec<PersistedTask> {
        if let Ok(tasks) = self.tasks.lock() {
            let mut list: Vec<PersistedTask> = tasks.values().cloned().collect();
            // Sort by added_at desc?
            list.sort_by(|a, b| b.added_at.cmp(&a.added_at));
            return list;
        }
        vec![]
    }

    pub fn remove_task(&self, gid: &str) -> bool {
        let mut removed = false;
        if let Ok(mut tasks) = self.tasks.lock() {
            removed = tasks.remove(gid).is_some();
        }
        self.save();
        removed
    }

    pub fn update_all_active_to_paused(&self) {
        if let Ok(mut tasks) = self.tasks.lock() {
            let now = Local::now().to_rfc3339();
            for task in tasks.values_mut() {
                if task.state == TaskState::Active || task.state == TaskState::Waiting {
                    task.state = TaskState::Paused;
                    task.completed_at = Some(now.clone());
                }
            }
        }
        self.save();
    }

    pub fn update_all_paused_to_waiting(&self) {
        if let Ok(mut tasks) = self.tasks.lock() {
            for task in tasks.values_mut() {
                if task.state == TaskState::Paused {
                    task.state = TaskState::Waiting;
                    task.completed_at = None;
                }
            }
        }
        self.save();
    }

    /// 批量更新任务状态（不触发 save，调用方需显式调用 save()）
    pub fn update_batch_state(&self, gids: &[String], state: TaskState) {
        if let Ok(mut tasks) = self.tasks.lock() {
            for gid in gids {
                if let Some(t) = tasks.get_mut(gid) {
                    t.transition_state(state);
                }
            }
        }
        // 不调用 save()，由调用方统一处理
    }

    /// 批量删除任务（不触发 save，调用方需显式调用 save()）
    /// 返回被成功删除的 GID 列表
    pub fn remove_tasks_batch(&self, gids: &[String]) -> Vec<String> {
        let mut removed = Vec::new();
        if let Ok(mut tasks) = self.tasks.lock() {
            for gid in gids {
                if tasks.remove(gid).is_some() {
                    removed.push(gid.clone());
                }
            }
        }
        // 不调用 save()，由调用方统一处理
        removed
    }

    /// Get all filenames currently known to the store (to prevent duplicates)
    pub fn get_active_filenames(&self) -> Vec<String> {
        if let Ok(tasks) = self.tasks.lock() {
            tasks
                .values()
                .map(|t| t.filename.clone())
                .filter(|n| !n.is_empty())
                .collect()
        } else {
            vec![]
        }
    }
}
