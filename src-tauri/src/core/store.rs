use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Mutex;
use std::time::{SystemTime, UNIX_EPOCH};
use tauri::{AppHandle, Manager};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PersistedTask {
    pub gid: String,
    pub filename: String,
    pub url: String,
    pub save_path: String,
    pub added_at: String,
    pub state: String, // 'active' | 'waiting' | 'paused' | 'completed' | 'error' | 'removed' | 'cancelled'
    pub total_length: String,
    pub completed_length: String,
    pub download_speed: String,
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
}

/// Minimum interval between save operations (ms)
const SAVE_DEBOUNCE_MS: u64 = 100;

pub struct TaskStore {
    pub tasks: Mutex<HashMap<String, PersistedTask>>,
    file_path: Mutex<Option<PathBuf>>,
    last_save_time: AtomicU64,
}

impl TaskStore {
    pub fn new() -> Self {
        Self {
            tasks: Mutex::new(HashMap::new()),
            file_path: Mutex::new(None),
            last_save_time: AtomicU64::new(0),
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
            return;
        }
        self.last_save_time.store(now, Ordering::Relaxed);

        // 1. Snapshot Path (Fast lock)
        let path = if let Ok(path_opt) = self.file_path.lock() {
            path_opt.clone()
        } else {
            None
        };

        if let Some(path) = path {
            // 2. Snapshot Data (Fast lock)
            let tasks = if let Ok(tasks_guard) = self.tasks.lock() {
                Some(
                    tasks_guard
                        .values()
                        .cloned()
                        .collect::<Vec<PersistedTask>>(),
                )
            } else {
                None
            };

            if let Some(mut list) = tasks {
                // 3. Offload IO to thread
                std::thread::spawn(move || {
                    // Sort for stable JSON output
                    list.sort_by(|a, b| b.added_at.cmp(&a.added_at));

                    if let Ok(json) = serde_json::to_string_pretty(&list) {
                        if let Err(e) = crate::utils::atomic_write(&path, &json) {
                            log::error!("Failed to save tasks: {}", e);
                        }
                    }
                });
            }
        }
    }

    /// Force save without debounce (for critical operations like app exit)
    pub fn force_save(&self) {
        self.last_save_time.store(0, Ordering::Relaxed);
        self.save();
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

    pub fn update_task_state(&self, gid: &str, state: &str) {
        if let Ok(mut tasks) = self.tasks.lock() {
            if let Some(t) = tasks.get_mut(gid) {
                t.state = state.to_string();
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

    // Batch update all tasks (Performance Optimization)
    pub fn update_all(&self, updated_tasks: Vec<PersistedTask>) {
        if let Ok(mut tasks) = self.tasks.lock() {
            // Update existing or insert new, but typically this comes from get_all() so it's updates.
            // However, sync_tasks iterates mutable list, so we might just want to replace the values
            // based on the list we passed back?
            // Actually, we should probably iterate and update.
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
        state: &str,
        completed: &str,
        speed: &str,
        total: &str,
    ) {
        if let Ok(mut tasks) = self.tasks.lock() {
            if let Some(t) = tasks.get_mut(gid) {
                t.state = state.to_string();
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
            for task in tasks.values_mut() {
                if task.state == "downloading" || task.state == "waiting" {
                    task.state = "paused".to_string();
                }
            }
        }
        self.save();
    }

    pub fn update_all_paused_to_waiting(&self) {
        if let Ok(mut tasks) = self.tasks.lock() {
            for task in tasks.values_mut() {
                if task.state == "paused" {
                    task.state = "waiting".to_string();
                }
            }
        }
        self.save();
    }

    /// 批量更新任务状态（不触发 save，调用方需显式调用 save()）
    pub fn update_batch_state(&self, gids: &[String], state: &str) {
        if let Ok(mut tasks) = self.tasks.lock() {
            for gid in gids {
                if let Some(t) = tasks.get_mut(gid) {
                    t.state = state.to_string();
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
