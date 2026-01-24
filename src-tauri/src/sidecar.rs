use tauri::AppHandle;
use tauri_plugin_shell::ShellExt;
use tauri_plugin_shell::process::CommandEvent;

pub fn init_aria2_sidecar(app: &AppHandle) {
    let sidecar_command = app.shell().sidecar("aria2c").unwrap();
    
    let mut args = vec![
        "--enable-rpc".to_string(),
        "--rpc-listen-all=true".to_string(),
        "--rpc-allow-origin-all".to_string(),
        "--rpc-listen-port=6800".to_string(),
        "--disable-ipv6".to_string(),
        "--log-level=warn".to_string(),
    ];

    // Check for custom configuration file
    // Check for custom configuration file & Setup session
    use tauri::Manager;
    if let Ok(config_dir) = app.path().app_config_dir() {
        // Ensure config directory exists
        if !config_dir.exists() {
            let _ = std::fs::create_dir_all(&config_dir);
        }

        // 1. Custom Config
        let conf_path = config_dir.join("aria2.conf");
        if conf_path.exists() {
             log::info!("Found custom aria2 config: {:?}", conf_path);
             args.push(format!("--conf-path={}", conf_path.to_string_lossy()));
        }

        // 2. Session File (Persistence)
        let session_path = config_dir.join("aria2.session");
        if !session_path.exists() {
            // Must create the file if it doesn't exist, otherwise aria2 fails to start with --input-file
            if let Err(e) = std::fs::File::create(&session_path) {
                log::error!("Failed to create session file: {}", e);
            }
        }
        
        let session_path_str = session_path.to_string_lossy();
        args.push(format!("--input-file={}", session_path_str));
        args.push(format!("--save-session={}", session_path_str));
        args.push("--save-session-interval=30".to_string());
    }

    // 配置 Aria2 参数
    let command = sidecar_command.args(&args);

    let (mut rx, child) = command.spawn().expect("Failed to spawn aria2 sidecar");
    
    let pid = child.pid();
    log::info!("Aria2 sidecar started with PID: {}", pid);

    // 异步监听输出 (可选，用于调试)
    tauri::async_runtime::spawn(async move {
        while let Some(event) = rx.recv().await {
            match event {
                CommandEvent::Stdout(line) => {
                    let log = String::from_utf8_lossy(&line);
                    log::info!("Aria2 stdout: {}", log);
                }
                CommandEvent::Stderr(line) => {
                    let log = String::from_utf8_lossy(&line);
                    log::warn!("Aria2 stderr: {}", log);
                }
                CommandEvent::Terminated(payload) => {
                    log::error!("Aria2 terminated: {:?}", payload);
                }
                _ => {}
            }
        }
    });
}
