use chrono::Local;
use crate::core::events::{EVENT_ARIA2_SIDECAR_ERROR, EVENT_ARIA2_STDOUT};
use serde_json::json;
use std::sync::Mutex;
use std::time::Duration;
use tauri::{AppHandle, Emitter, Manager};
use tauri_plugin_shell::process::CommandChild;
use tauri_plugin_shell::process::CommandEvent;
use tauri_plugin_shell::ShellExt;
 
const INFINITE_SEED_TIME: &str = "999999999";

fn find_available_port(start: u16) -> Result<u16, String> {
    let mut port = start;
    loop {
        if std::net::TcpListener::bind(("127.0.0.1", port)).is_ok() {
            return Ok(port);
        }
        port += 1;
        // 限制搜索范围以防止死循环
        if port > start + 100 {
            return Err(format!(
                "在范围 {}-{} 内找不到可用的 Aria2 RPC 端口，请检查端口占用情况。",
                start,
                start + 100
            ));
        }
    }
}

pub struct SidecarState {
    pub child: Mutex<Option<CommandChild>>,
    pub native_child: Mutex<Option<std::process::Child>>,
    pub recent_logs: Mutex<Vec<String>>,
}

pub struct ShutdownState(pub std::sync::atomic::AtomicBool);

pub fn init_aria2_sidecar(app: AppHandle) {
    tauri::async_runtime::spawn(async move {
        loop {
            // 启动前检查是否正在关闭（防止启动过快）
            if let Some(state) = app.try_state::<ShutdownState>() {
                if state.0.load(std::sync::atomic::Ordering::SeqCst) {
                    crate::app_info!("Aria2::Sidecar", "shutdown_flag_detected");
                    break;
                }
            }

            let sidecar_command = match app.shell().sidecar("aria2c") {
                Ok(cmd) => cmd,
                Err(e) => {
                    crate::app_error!(
                        "Aria2::Sidecar",
                        "sidecar_command_create_failed",
                        json!({ "error": e.to_string(), "retry_in_secs": 5 })
                    );
                    tokio::time::sleep(Duration::from_secs(5)).await;
                    continue;
                }
            };

            // 1. 检查现有配置
            let (
                preferred_port,
                save_session_interval,
                existing_secret,
                max_concurrent,
                bt_trackers,
                enable_dht,
                enable_peer_exchange,
                enable_seeding,
                seed_ratio,
                dht_listen_port,
                listen_port,
                global_max_upload_limit,
            ) = {
                if let Some(state) = app
                    .state::<crate::core::config::ConfigState>()
                    .config
                    .lock()
                    .ok()
                {
                    (
                        state.rpc_port,
                        state.save_session_interval,
                        state.rpc_secret.clone(),
                        state.max_concurrent_downloads,
                        state.bt_trackers.clone(),
                        state.enable_dht,
                        state.enable_peer_exchange,
                        state.enable_seeding,
                        state.seed_ratio,
                        state.dht_listen_port.clone(),
                        state.listen_port.clone(),
                        state.global_max_upload_limit.clone(),
                    )
                } else {
                    let default = crate::core::config::AppConfig::default();
                    (
                        default.rpc_port,
                        default.save_session_interval,
                        default.rpc_secret.clone(),
                        default.max_concurrent_downloads,
                        default.bt_trackers.clone(),
                        default.enable_dht,
                        default.enable_peer_exchange,
                        default.enable_seeding,
                        default.seed_ratio,
                        default.dht_listen_port.clone(),
                        default.listen_port.clone(),
                        default.global_max_upload_limit.clone(),
                    )
                }
            };
            crate::app_info!(
                "Aria2::Sidecar",
                "port_preferred",
                json!({ "port": preferred_port })
            );

            // 2. 查找可用端口
            let port = match find_available_port(preferred_port) {
                Ok(p) => p,
                Err(e) => {
                    crate::app_error!(
                        "Aria2::Sidecar",
                        "port_allocate_failed",
                        json!({ "error": e })
                    );
                    tokio::time::sleep(Duration::from_secs(10)).await;
                    continue;
                }
            };
            crate::app_info!("Aria2::Sidecar", "port_selected", json!({ "port": port }));

            // 3. 更新客户端配置（端口 + Secret）
            crate::aria2::client::set_aria2_port(port);

            // 如果已有配置的 Secret，也要同步给 Client
            if let Some(ref secret) = existing_secret {
                crate::aria2::client::set_aria2_secret(secret.clone()).await;
            }

            let mut args = vec![
                "--enable-rpc".to_string(),
                "--rpc-listen-all=false".to_string(),
                format!("--rpc-listen-port={}", port),
                "--disable-ipv6".to_string(),
                "--log-level=warn".to_string(),
                format!("--max-concurrent-downloads={}", max_concurrent),
                format!("--stop-with-process={}", std::process::id()), // 父进程退出时自动关闭
                format!("--enable-dht={}", enable_dht),
                format!("--enable-peer-exchange={}", enable_peer_exchange),
                format!("--seed-ratio={}", seed_ratio),
                // 如果启用做种，不传 seed-time (默认无限/受 ratio 控制)
                // 如果禁用做种，传 seed-time=0
                format!(
                    "--seed-time={}",
                    if enable_seeding {
                        INFINITE_SEED_TIME.to_string()
                    } else {
                        "0".to_string()
                    }
                ),
                format!("--dht-listen-port={}", dht_listen_port),
                format!("--listen-port={}", listen_port),
            ];

            if !global_max_upload_limit.is_empty() {
                args.push(format!("--max-overall-upload-limit={}", global_max_upload_limit));
            }

            let normalized_bt_trackers = crate::utils::normalize_bt_trackers(&bt_trackers);
            if !normalized_bt_trackers.is_empty() {
                args.push(format!("--bt-tracker={}", normalized_bt_trackers));
            }

            if let Some(ref secret) = existing_secret {
                args.push(format!("--rpc-secret={}", secret));
            }

            // 检查自定义配置文件
            if let Ok(config_dir) = app.path().app_config_dir() {
                // 确保配置目录存在
                if !config_dir.exists() {
                    let _ = std::fs::create_dir_all(&config_dir);
                }

                // 1. 自定义配置
                let conf_path = config_dir.join("aria2.conf");
                if conf_path.exists() {
                    crate::app_info!(
                        "Aria2::Sidecar",
                        "custom_config_detected",
                        json!({ "path": conf_path.to_string_lossy() })
                    );
                    args.push(format!("--conf-path={}", conf_path.to_string_lossy()));
                }

                // 2. 会话文件 (持久化)
                let session_path = config_dir.join("aria2.session");
                if !session_path.exists() {
                    if let Err(e) = std::fs::File::create(&session_path) {
                        crate::app_error!(
                            "Aria2::Sidecar",
                            "session_file_create_failed",
                            json!({ "path": session_path.to_string_lossy(), "error": e.to_string() })
                        );
                    }
                }

                let session_path_str = session_path.to_string_lossy();
                args.push(format!("--input-file={}", session_path_str));
                args.push(format!("--save-session={}", session_path_str));
                args.push(format!("--save-session-interval={}", save_session_interval));
            }

            crate::app_info!("Aria2::Sidecar", "start_requested");

            // --- 自定义二进制逻辑 ---
            let config_state = app.state::<crate::core::config::ConfigState>();
            let use_custom = config_state
                .config
                .lock()
                .map(|c| c.use_custom_aria2)
                .unwrap_or(false);

            let custom_command = if use_custom {
                if let Ok(config_dir) = app.path().app_config_dir() {
                    let target_name = if cfg!(windows) {
                        "aria2c.exe"
                    } else {
                        "aria2c"
                    };
                    let bin_path = config_dir.join("custom-bin").join(target_name);
                    if bin_path.exists() {
                        crate::app_info!(
                            "Aria2::Sidecar",
                            "custom_binary_selected",
                            json!({ "path": bin_path.to_string_lossy() })
                        );
                        Some(std::process::Command::new(bin_path))
                    } else {
                        crate::app_warn!("Aria2::Sidecar", "custom_binary_missing_fallback_builtin");
                        None
                    }
                } else {
                    None
                }
            } else {
                None
            };

            let (mut rx, child_pid) = if let Some(mut cmd) = custom_command {
                // --- 原生进程生成 ---
                use std::io::{BufRead, BufReader};
                use std::process::Stdio;

                cmd.stdout(Stdio::piped())
                    .stderr(Stdio::piped())
                    .args(&args);

                match cmd.spawn() {
                    Ok(mut child) => {
                        let pid = child.id();
                        let (tx, rx) = tokio::sync::mpsc::channel(128);

                        // 桥接标准输出 (Stdout)
                        if let Some(stdout) = child.stdout.take() {
                            let tx = tx.clone();
                            std::thread::spawn(move || {
                                let reader = BufReader::new(stdout);
                                for line in reader.lines() {
                                    if let Ok(l) = line {
                                        let _ =
                                            tx.blocking_send(CommandEvent::Stdout(l.into_bytes()));
                                    }
                                }
                            });
                        }

                        // 桥接标准错误 (Stderr)
                        if let Some(stderr) = child.stderr.take() {
                            let tx = tx.clone();
                            std::thread::spawn(move || {
                                let reader = BufReader::new(stderr);
                                for line in reader.lines() {
                                    if let Ok(l) = line {
                                        let _ =
                                            tx.blocking_send(CommandEvent::Stderr(l.into_bytes()));
                                    }
                                }
                            });
                        }

                        // 存储原生子进程
                        let state = app.state::<SidecarState>();
                        if let Ok(mut guard) = state.native_child.lock() {
                            *guard = Some(child);
                        }

                        (rx, pid)
                    }
                    Err(e) => {
                        crate::app_error!(
                            "Aria2::Sidecar",
                            "custom_binary_spawn_failed",
                            json!({ "error": e.to_string() })
                        );
                        tokio::time::sleep(Duration::from_secs(5)).await;
                        continue;
                    }
                }
            } else {
                // --- 内置 Sidecar ---
                let command = sidecar_command.args(&args);
                match command.spawn() {
                    Ok((rx, child)) => {
                        let pid = child.pid();
                        // 存储 tauri 子进程
                        let state = app.state::<SidecarState>();
                        if let Ok(mut guard) = state.child.lock() {
                            *guard = Some(child);
                        }
                        (rx, pid)
                    }
                    Err(e) => {
                        crate::app_error!(
                            "Aria2::Sidecar",
                            "builtin_sidecar_spawn_failed",
                            json!({ "error": e.to_string() })
                        );
                        tokio::time::sleep(Duration::from_secs(5)).await;
                        continue;
                    }
                }
            };

            crate::app_info!(
                "Aria2::Sidecar",
                "started",
                json!({ "pid": child_pid, "port": port })
            );

            // 监控进程
            let mut manually_exited = false;
            let mut stderr_buffer: Vec<String> = Vec::new();

            while let Some(event) = rx.recv().await {
                match event {
                    CommandEvent::Stdout(line) => {
                        let log = String::from_utf8_lossy(&line);
                        let log_trimmed = log.trim();

                        // 跳过空行
                        if log_trimmed.is_empty() {
                            continue;
                        }

                        if log_trimmed.contains("Serialized session to") {
                            crate::app_debug!(
                                "Aria2::Sidecar",
                                "stdout_session_persisted",
                                json!({ "line": log_trimmed })
                            );
                        } else {
                            crate::app_debug!(
                                "Aria2::Sidecar",
                                "stdout",
                                json!({ "line": log_trimmed })
                            );
                        }

                        let now = Local::now().format("%H:%M:%S");
                        let log_str = format!("[{}] [INFO] {}", now, log.trim());

                        // 日志缓冲
                        if let Some(state) = app.try_state::<SidecarState>() {
                            if let Ok(mut logs) = state.recent_logs.lock() {
                                logs.push(log_str.clone());
                                if logs.len() > 100 {
                                    logs.remove(0);
                                }
                            }
                        }

                        // 如果启用了日志流，则发送事件
                        if let Some(state) =
                            app.try_state::<crate::aria2::sidecar::LogStreamEnabled>()
                        {
                            if state.0.load(std::sync::atomic::Ordering::Relaxed) {
                                let _ = app.emit(EVENT_ARIA2_STDOUT, log_str);
                            }
                        }
                    }
                    CommandEvent::Stderr(line) => {
                        let log = String::from_utf8_lossy(&line);
                        let now = Local::now().format("%H:%M:%S");
                        let log_str = format!("[{}] [ERROR] {}", now, log.trim());
                        crate::app_warn!(
                            "Aria2::Sidecar",
                            "stderr",
                            json!({ "line": log.trim() })
                        );
                        stderr_buffer.push(log.to_string());
                        if stderr_buffer.len() > 20 {
                            stderr_buffer.remove(0);
                        }

                        // 日志缓冲
                        if let Some(state) = app.try_state::<SidecarState>() {
                            if let Ok(mut logs) = state.recent_logs.lock() {
                                logs.push(log_str.clone());
                                if logs.len() > 100 {
                                    logs.remove(0);
                                }
                            }
                        }

                        // 如果启用了日志流，则发送事件
                        if let Some(state) =
                            app.try_state::<crate::aria2::sidecar::LogStreamEnabled>()
                        {
                            if state.0.load(std::sync::atomic::Ordering::Relaxed) {
                                    let _ = app.emit(EVENT_ARIA2_STDOUT, log_str);
                            }
                        }
                    }
                    CommandEvent::Terminated(payload) => {
                        // 首先检查是否正在关闭
                        if let Some(state) = app.try_state::<ShutdownState>() {
                            if state.0.load(std::sync::atomic::Ordering::SeqCst) {
                                crate::app_info!(
                                    "Aria2::Sidecar",
                                    "terminated_during_shutdown"
                                );
                                manually_exited = true;
                                break;
                            }
                        }

                        crate::app_warn!(
                            "Aria2::Sidecar",
                            "terminated",
                            json!({ "code": payload.code, "signal": payload.signal })
                        );

                        // 如果是异常退出（非0状态码或被信号终止），发送事件到前端
                        let is_error = payload.code.map(|c| c != 0).unwrap_or(true)
                            || payload.signal.is_some();

                        if is_error {
                            crate::app_error!("Aria2::Sidecar", "unexpected_exit_emitting_event");
                            let stderr = stderr_buffer.join("");
                            let _ = app.emit(
        EVENT_ARIA2_SIDECAR_ERROR,
                                serde_json::json!({
                                    "message": "Aria2 sidecar 意外退出",
                                    "code": payload.code,
                                    "signal": payload.signal,
                                    "stderr": stderr
                                }),
                            );
                        }

                        manually_exited = true;
                        break;
                    }
                    _ => {}
                }
            }

            if !manually_exited {
                crate::app_warn!("Aria2::Sidecar", "channel_closed_unexpectedly");
            }

            // 重启前检查关闭状态
            if let Some(state) = app.try_state::<ShutdownState>() {
                if state.0.load(std::sync::atomic::Ordering::SeqCst) {
                    crate::app_info!("Aria2::Sidecar", "shutdown_skip_restart");
                    break;
                }
            }

            crate::app_info!(
                "Aria2::Sidecar",
                "restart_scheduled",
                json!({ "delay_secs": 5 })
            );
            tokio::time::sleep(Duration::from_secs(5)).await;
        }
    });
}

pub struct LogStreamEnabled(pub std::sync::atomic::AtomicBool);
