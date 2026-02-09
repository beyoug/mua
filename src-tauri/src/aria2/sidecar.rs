use chrono::Local;
use std::sync::Mutex;
use std::time::Duration;
use tauri::{AppHandle, Emitter, Manager};
use tauri_plugin_shell::process::CommandChild;
use tauri_plugin_shell::process::CommandEvent;
use tauri_plugin_shell::ShellExt;

fn find_available_port(start: u16) -> u16 {
    let mut port = start;
    loop {
        // 检查 0.0.0.0，因为 aria2c 使用 --rpc-listen-all=true
        if std::net::TcpListener::bind(("0.0.0.0", port)).is_ok() {
            return port;
        }
        port += 1;
        // 限制搜索范围以防止死循环
        if port > start + 100 {
            log::warn!(
                "Could not find available port in range {}-{}, fallback to {}",
                start,
                start + 100,
                start
            );
            return start;
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
                    log::info!("应用正在关闭，停止 sidecar 循环。");
                    break;
                }
            }

            let sidecar_command = match app.shell().sidecar("binaries/aria2c") {
                Ok(cmd) => cmd,
                Err(e) => {
                    log::error!("创建 aria2 sidecar 命令失败: {}。5秒后重试...", e);
                    tokio::time::sleep(Duration::from_secs(5)).await;
                    continue;
                }
            };

            // 1. 检查现有配置
            let (preferred_port, save_session_interval, existing_secret) = {
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
                    )
                } else {
                    (6800, 30, None)
                }
            };
            log::info!("首选 Aria2 端口: {}", preferred_port);

            // 2. 查找可用端口
            let port = find_available_port(preferred_port);
            log::info!("为 Aria2 选择的端口: {}", port);

            // 3. 更新客户端配置（端口 + Secret）
            crate::aria2::client::set_aria2_port(port);

            // 如果已有配置的 Secret，也要同步给 Client
            if let Some(ref secret) = existing_secret {
                crate::aria2::client::set_aria2_secret(secret.clone()).await;
            }

            let mut args = vec![
                "--enable-rpc".to_string(),
                "--rpc-listen-all=true".to_string(),
                "--rpc-allow-origin-all".to_string(),
                format!("--rpc-listen-port={}", port),
                "--disable-ipv6".to_string(),
                "--log-level=warn".to_string(),
                format!("--stop-with-process={}", std::process::id()), // 父进程退出时自动关闭
            ];

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
                    log::info!("发现自定义 aria2 配置: {:?}", conf_path);
                    args.push(format!("--conf-path={}", conf_path.to_string_lossy()));
                }

                // 2. 会话文件 (持久化)
                let session_path = config_dir.join("aria2.session");
                if !session_path.exists() {
                    if let Err(e) = std::fs::File::create(&session_path) {
                        log::error!("创建会话文件失败: {}", e);
                    }
                }

                let session_path_str = session_path.to_string_lossy();
                args.push(format!("--input-file={}", session_path_str));
                args.push(format!("--save-session={}", session_path_str));
                args.push(format!("--save-session-interval={}", save_session_interval));
            }

            log::info!("正在启动 Aria2 sidecar...");

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
                        log::info!("正在使用自定义 Aria2 二进制文件: {:?}", bin_path);
                        Some(std::process::Command::new(bin_path))
                    } else {
                        log::warn!("启用了自定义 Aria2 但未找到文件，回退到内置版本。");
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
                        log::error!("生成自定义二进制进程失败: {}", e);
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
                        log::error!("生成 sidecar 进程失败: {}", e);
                        tokio::time::sleep(Duration::from_secs(5)).await;
                        continue;
                    }
                }
            };

            log::info!("Aria2 已启动，PID: {}", child_pid);

            // 监控进程
            let mut manually_exited = false;
            let mut stderr_buffer: Vec<String> = Vec::new();

            while let Some(event) = rx.recv().await {
                match event {
                    CommandEvent::Stdout(line) => {
                        let log = String::from_utf8_lossy(&line);
                        if log.contains("Serialized session to") {
                            log::debug!("Aria2 stdout: {}", log);
                        } else {
                            log::info!("Aria2 stdout: {}", log);
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
                                let _ = app.emit("aria2-stdout", log_str);
                            }
                        }
                    }
                    CommandEvent::Stderr(line) => {
                        let log = String::from_utf8_lossy(&line);
                        let now = Local::now().format("%H:%M:%S");
                        let log_str = format!("[{}] [ERROR] {}", now, log.trim());
                        log::warn!("Aria2 stderr: {}", log);
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
                                let _ = app.emit("aria2-stdout", log_str);
                            }
                        }
                    }
                    CommandEvent::Terminated(payload) => {
                        // 首先检查是否正在关闭
                        if let Some(state) = app.try_state::<ShutdownState>() {
                            if state.0.load(std::sync::atomic::Ordering::SeqCst) {
                                log::info!("Aria2 如预期在应用关闭期间终止。");
                                manually_exited = true;
                                break;
                            }
                        }

                        log::warn!("Aria2 已终止: {:?}", payload);

                        // 如果是异常退出（非0状态码或被信号终止），发送事件到前端
                        let is_error = payload.code.map(|c| c != 0).unwrap_or(true)
                            || payload.signal.is_some();

                        if is_error {
                            log::error!("Aria2 崩溃。正在发送错误事件。");
                            let stderr = stderr_buffer.join("");
                            let _ = app.emit(
                                "aria2-sidecar-error",
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
                log::warn!("Aria2 通道意外关闭。");
            }

            // 重启前检查关闭状态
            if let Some(state) = app.try_state::<ShutdownState>() {
                if state.0.load(std::sync::atomic::Ordering::SeqCst) {
                    log::info!("应用正在关闭。不再重启 sidecar。");
                    break;
                }
            }

            log::info!("Aria2 sidecar 已退出。5 秒后重启...");
            tokio::time::sleep(Duration::from_secs(5)).await;
        }
    });
}

pub struct LogStreamEnabled(pub std::sync::atomic::AtomicBool);
