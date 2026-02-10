#![cfg_attr(mobile, tauri::mobile_entry_point)]

pub mod aria2;
pub mod core;
pub mod ui;
pub mod utils;

use core::commands::*;
use tauri::Manager;
use ui::tray::update_tray_icon_with_speed;

pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_autostart::Builder::new().build())
        .plugin(tauri_plugin_notification::init())
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_process::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_single_instance::init(|app, _argv, _cwd| {
            use tauri::Manager;
            if let Some(window) = app.get_webview_window("main") {
                let _ = window.unminimize();
                let _ = window.set_focus();
            }
        }))
        .manage(crate::aria2::sidecar::SidecarState {
            child: std::sync::Mutex::new(None),
            native_child: std::sync::Mutex::new(None),
            recent_logs: std::sync::Mutex::new(Vec::new()),
        })
        .manage(crate::aria2::sidecar::ShutdownState(
            std::sync::atomic::AtomicBool::new(false),
        ))
        .manage(crate::aria2::sidecar::LogStreamEnabled(
            std::sync::atomic::AtomicBool::new(false),
        ))
        .manage(crate::core::store::TaskStore::new()) // Initialize TaskStore
        .setup(|app| {
            crate::core::boot::run(app)?;
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            add_download_task,
            add_download_tasks,
            get_tasks,
            pause_task,
            resume_task,
            cancel_task,
            remove_task_record,
            get_aria2_config_path,
            read_aria2_config,
            import_aria2_config,
            update_tray_icon_with_speed,
            get_app_config,
            save_app_config,
            show_task_in_folder,
            pause_all_tasks,
            resume_all_tasks,
            remove_tasks,
            cancel_tasks,
            start_log_stream,
            stop_log_stream,
            import_custom_binary,
            get_aria2_version_info
        ])
        .on_window_event(|window, event| {
            if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                let app = window.app_handle();
                let config = app.state::<crate::core::config::ConfigState>();

                // 获取配置，如果锁获取失败则默认为 true (保持应用运行)
                let close_to_tray = config
                    .config
                    .lock()
                    .map(|c| c.close_to_tray)
                    .unwrap_or(true);

                if close_to_tray {
                    api.prevent_close();
                    let _ = window.hide();
                } else {
                    // 如果选择"退出程序"，显式退出应用。
                    // 解决 macOS 上窗口关闭不默认退出应用的问题。
                    app.exit(0);
                }
            }
        })
        .build(tauri::generate_context!())
        .expect("error while building tauri application")
        .run(|app, event| match event {
            tauri::RunEvent::Exit => {
                // Signal shutdown to sidecar loop
                if let Some(state) = app.try_state::<crate::aria2::sidecar::ShutdownState>() {
                    state.0.store(true, std::sync::atomic::Ordering::SeqCst);
                }

                let (child, native_child) = {
                    let state = app.state::<crate::aria2::sidecar::SidecarState>();
                    (
                        state.child.lock().ok().and_then(|mut c| c.take()),
                        state.native_child.lock().ok().and_then(|mut c| c.take()),
                    )
                };

                if let Some(child) = child {
                    log::info!("Killing aria2 sidecar process...");
                    let _ = child.kill();
                }

                if let Some(mut child) = native_child {
                    log::info!("Killing custom aria2 process...");
                    let _ = child.kill();
                }
            }
            #[cfg(target_os = "macos")]
            tauri::RunEvent::Reopen { .. } => {
                if let Some(window) = app.get_webview_window("main") {
                    let _ = window.show();
                    let _ = window.set_focus();
                }
            }
            _ => {}
        });
}
