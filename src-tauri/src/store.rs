use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use std::sync::Mutex;
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
}

pub struct TaskStore {
    pub tasks: Mutex<HashMap<String, PersistedTask>>,
    file_path: Mutex<Option<PathBuf>>,
}

impl TaskStore {
    pub fn new() -> Self {
        Self {
            tasks: Mutex::new(HashMap::new()),
            file_path: Mutex::new(None),
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
        if let Ok(path_opt) = self.file_path.lock() {
            if let Some(path) = path_opt.as_ref() {
                if let Ok(tasks) = self.tasks.lock() {
                    let list: Vec<&PersistedTask> = tasks.values().collect();
                    if let Ok(json) = serde_json::to_string_pretty(&list) {
                        // Ensure dir exists
                        if let Some(parent) = path.parent() {
                            let _ = fs::create_dir_all(parent);
                        }
                        let _ = fs::write(path, json);
                    }
                }
            }
        }
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

    pub fn remove_task(&self, gid: &str) {
        if let Ok(mut tasks) = self.tasks.lock() {
            tasks.remove(gid);
        }
        self.save();
    }
}
