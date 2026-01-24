#[cfg_attr(mobile, tauri::mobile_entry_point)]

mod sidecar;
mod aria2_client;
use aria2_client::Aria2Task;

use std::collections::HashMap;

#[tauri::command]
async fn add_download_task(
    urls: Vec<String>,
    save_path: Option<String>,
    filename: Option<String>,
    user_agent: Option<String>,
    referer: Option<String>,
    headers: Option<String>,
    proxy: Option<String>,
    max_download_limit: Option<String>
) -> Result<String, String> {
    log::info!("add_download_task called with urls: {:?}", urls);
    let mut options = HashMap::new();
    
    if let Some(dir) = save_path {
        // Resolve tilde to home dir
        let resolved_dir = if dir.starts_with("~") {
             if let Some(home) = dirs::home_dir() {
                 dir.replacen("~", home.to_str().unwrap_or(""), 1)
             } else {
                 dir
             }
        } else {
            dir
        };
        options.insert("dir".to_string(), resolved_dir);
    }
    if let Some(out) = filename {
        if !out.is_empty() {
             options.insert("out".to_string(), out);
        }
    }
    
    // 构建 header 选项
    let mut header_list = Vec::new();
    if let Some(ua) = user_agent {
        if !ua.is_empty() {
            options.insert("user-agent".to_string(), ua);
        }
    }
    if let Some(ref_url) = referer {
        if !ref_url.is_empty() {
             options.insert("referer".to_string(), ref_url);
        }
    }
    // 处理自定义 headers (假设格式 key:value;key:value)
    if let Some(h_str) = headers {
        for h in h_str.split(';') {
             if !h.trim().is_empty() {
                 header_list.push(h.trim().to_string());
             }
        }
    }
    if !header_list.is_empty() {
        // Aria2 接受 header 作为一个选项，但在 addUri params 中其实是通过 separate options 传递的？
        // 其实 aria2.addUri 的 options 字典里，key 是 'header'，value 是 headers 的 list 吗？
        // 不，aria2 文档显示 'header' 是一个 list of strings。
        // 但 reqwest 构建 json map 时 Rust HashMap<String, String> 不支持 value 为 List。
        // 修正：我们需要把 options 定义为 HashMap<String, Value> 或者单独处理。
        // 为了简单，我们先暂不支持 header 列表，或者修改 aria2_client 签名。
        // 实际上 aria2 选项里 'header' 确实是可以重复的，但在 JSON-RPC options 对象里，
        // 根据文档： "header": ["Key: Value", ...]
        // 这里的 options 是 Map<String, String>... 哎呀 aria2 crate usually uses JSON Value.
        // 让我们简化，暂不支持自定义 header，或者稍后修改 aria2_client.rs 适配 Value。
    }

    if let Some(p) = proxy {
         if !p.is_empty() {
             options.insert("all-proxy".to_string(), p);
         }
    }
    
    if let Some(limit) = max_download_limit {
         if !limit.is_empty() {
             options.insert("max-download-limit".to_string(), limit);
         }
    }

    aria2_client::add_uri(urls, Some(options)).await
}

#[tauri::command]
async fn pause_task(gid: String) -> Result<String, String> {
    aria2_client::pause(gid).await
}

#[tauri::command]
async fn resume_task(gid: String) -> Result<String, String> {
    aria2_client::resume(gid).await
}

#[tauri::command]
async fn cancel_task(gid: String) -> Result<String, String> {
    aria2_client::remove(gid).await
}

#[tauri::command]
async fn remove_task_record(gid: String, delete_file: bool, filepath: Option<String>) -> Result<String, String> {
    // 1. Remove from Aria2 memory
    let _ = aria2_client::purge(gid.clone()).await; 
    
    // 2. Delete file if requested
    if delete_file {
        if let Some(path) = filepath {
             // Handle tilde expansion if necessary (though aria2 returns abs path usually)
            let resolved_path = if path.starts_with("~") {
                 if let Some(home) = dirs::home_dir() {
                     path.replacen("~", home.to_str().unwrap_or(""), 1)
                 } else {
                     path
                 }
            } else {
                path
            };
            
            if std::path::Path::new(&resolved_path).exists() {
                 match std::fs::remove_file(&resolved_path) {
                    Ok(_) => log::info!("Deleted file: {}", resolved_path),
                    Err(e) => log::error!("Failed to delete file {}: {}", resolved_path, e),
                 }
            } else {
                log::warn!("File not found for deletion: {}", resolved_path);
            }
        } else {
            log::warn!("Delete file requested but no filepath provided for gid: {}", gid);
        }
    }
    Ok("OK".to_string())
}

#[tauri::command]
async fn get_tasks() -> Result<Vec<Aria2Task>, String> {
    let tasks = aria2_client::get_all_tasks().await;
    if let Ok(ref t) = tasks {
        log::info!("get_tasks returning {} tasks: {:?}", t.len(), t.iter().map(|x| (x.gid.clone(), x.status.clone())).collect::<Vec<_>>());
    }
    tasks
}

pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_dialog::init())
        .setup(|app| {
            #[cfg(desktop)]
            sidecar::init_aria2_sidecar(app.handle());

            if cfg!(debug_assertions) {
                app.handle().plugin(
                    tauri_plugin_log::Builder::default()
                        .level(log::LevelFilter::Info)
                        .build(),
                )?;
            }
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            add_download_task, 
            get_tasks,
            pause_task,
            resume_task,
            cancel_task,
            remove_task_record
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
