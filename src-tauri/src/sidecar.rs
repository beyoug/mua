use tauri::AppHandle;
use tauri_plugin_shell::ShellExt;
use tauri_plugin_shell::process::CommandEvent;

pub fn init_aria2_sidecar(app: &AppHandle) {
    let sidecar_command = app.shell().sidecar("aria2c").unwrap();
    
    // 配置 Aria2 参数
    let command = sidecar_command.args([
        "--enable-rpc",
        "--rpc-listen-all=true",
        "--rpc-allow-origin-all",
        "--rpc-listen-port=6800",
        // 建议禁用 ipv6 以避免某些网络环境下的连接问题
        "--disable-ipv6", 
        // 保持 session 文件 (可选)
        // "--save-session=aria2.session",
        // "--input-file=aria2.session",
        // 日志级别
        "--log-level=warn"
    ]);

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
