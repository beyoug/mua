use crate::aria2::sidecar;
use crate::core::commands;
use crate::core::config::{self, ConfigState};
use crate::core::store::TaskStore;
use crate::core::sync;
use crate::core::types::TaskState;
use crate::ui::tray;
use serde_json::json;
use std::sync::Mutex;
use tauri::{App, Manager};

/// 运行应用启动链路
/// 官方术语：生命周期流水线核心执行器
pub fn run(app: &mut App) -> Result<(), Box<dyn std::error::Error>> {
    let handle = app.handle().clone();

    // --- L1: 基础系统层 ---
    // 1. 初始化托盘图标与菜单
    tray::setup_tray(app)?;

    // 2. 加载持久化配置
    let config = config::load_config(&handle);
    app.manage(ConfigState {
        config: Mutex::new(config.clone()),
    });

    // --- L2: 核心服务层 ---
    // 3. 初始化任务存储引擎
    let store = app.state::<TaskStore>();
    store.init(&handle);

    // 4. 激活 Aria2 Sidecar 进程
    #[cfg(desktop)]
    sidecar::init_aria2_sidecar(handle.clone());

    // 5. 调试环境下注入日志增强插件
    if cfg!(debug_assertions) {
        handle.plugin(
            tauri_plugin_log::Builder::default()
                .level(log::LevelFilter::Info)
                .build(),
        )?;
    }

    // --- L3: 业务逻辑层 ---
    // 6. 执行任务自动恢复 (Auto Resume)
    if config.auto_resume {
        crate::app_info!("Core::Boot", "auto_resume_enabled");
        let app_handle_resume = handle.clone();
        tauri::async_runtime::spawn(async move {
            let state = app_handle_resume.state::<TaskStore>();
            let store_tasks = state.get_all();

            for task in store_tasks {
                if task.state == TaskState::Paused
                    || task.state == TaskState::Waiting
                    || task.state == TaskState::Active
                {
                    crate::app_info!(
                        "Core::Boot",
                        "auto_resume_task",
                        json!({ "gid": task.gid })
                    );
                    let _ = commands::resume_task(state.clone(), task.gid).await;
                }
            }
        });
    }

    // 7. 启动后台同步循环
    sync::start_background_sync(handle.clone());

    // --- L4: 视图管理层 ---
    // 8. 应用窗口启动显隐策略
    if !config.start_minimized {
        if let Some(window) = app.get_webview_window("main") {
            crate::app_info!("Core::Boot", "window_show_default");
            let _ = window.show();
            let _ = window.unminimize();
            let _ = window.set_focus();
        }
    } else {
        if let Some(window) = app.get_webview_window("main") {
            crate::app_info!("Core::Boot", "window_start_minimized");
            let _ = window.hide();
        }
    }

    Ok(())
}
