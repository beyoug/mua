#![cfg_attr(mobile, tauri::mobile_entry_point)]

pub mod aria2;
pub mod core;
pub mod ui;
pub mod utils;

use core::commands::*;
use tauri::Manager;
use ui::tray::{self, update_tray_icon_with_speed};

pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_notification::init())
        .plugin(tauri_plugin_shell::init())
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
            recent_logs: std::sync::Mutex::new(Vec::new()),
        })
        .manage(crate::aria2::sidecar::LogStreamEnabled(
            std::sync::atomic::AtomicBool::new(false),
        ))
        .manage(crate::core::store::TaskStore::new()) // Initialize TaskStore
        .setup(|app| {
            // 初始化托盘 (封装)
            tray::setup_tray(app)?;

            // 初始化配置
            let config = core::config::load_config(app.handle());
            app.manage(core::config::ConfigState {
                config: std::sync::Mutex::new(config.clone()),
            });

            // Initialize Store Path
            let store = app.state::<crate::core::store::TaskStore>();
            store.init(app.handle());

            // 初始化 Aria2 Sidecar
            #[cfg(desktop)]
            crate::aria2::sidecar::init_aria2_sidecar(app.handle().clone());

            if cfg!(debug_assertions) {
                app.handle().plugin(
                    tauri_plugin_log::Builder::default()
                        .level(log::LevelFilter::Info)
                        .build(),
                )?;
            }

            // --- Auto Resume Logic ---
            let config_state = app.state::<crate::core::config::ConfigState>();
            let auto_resume = config_state
                .config
                .lock()
                .map(|c| c.auto_resume)
                .unwrap_or(false);

            if auto_resume {
                log::info!("Auto Resume enabled. Attempting to resume tasks...");
                let app_handle_resume = app.handle().clone();

                tauri::async_runtime::spawn(async move {
                    let state = app_handle_resume.state::<crate::core::store::TaskStore>();
                    let store_tasks = state.get_all();

                    for task in store_tasks {
                        if task.state == "paused"
                            || task.state == "waiting"
                            || task.state == "downloading"
                        {
                            log::info!("Auto-resuming task: {}", task.gid);
                            let _ = core::commands::resume_task(state.clone(), task.gid).await;
                        }
                    }
                });
            }

            // --- Background Sync Loop ---
            crate::core::sync::start_background_sync(app.handle().clone());

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            add_download_task,
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
            stop_log_stream
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
                }
                // 如果为 false，则允许正常关闭。
                // 注意：在 Mac 上，关闭最后一个窗口默认不会退出应用（Tauri 行为），
                // 但这通常是符合预期的。
                // 用户需求："退出应用或最小化到托盘"。
                // 如果选择"退出应用"，我们可能需要强制退出，或者让标准行为接管。
                // 标准行为是：窗口关闭，应用保留在 Dock 中运行。
                // 暂时假设用户只想切换"最小化到托盘"（隐藏窗口）的行为。
                // 如果他们禁用了它，那就像普通的窗口关闭一样。
            }
        })
        .build(tauri::generate_context!())
        .expect("error while building tauri application")
        .run(|app, event| match event {
            tauri::RunEvent::Exit => {
                let child = {
                    let state = app.state::<crate::aria2::sidecar::SidecarState>();
                    state.child.lock().ok().and_then(|mut c| c.take())
                };

                if let Some(child) = child {
                    log::info!("Killing aria2 sidecar process...");
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
