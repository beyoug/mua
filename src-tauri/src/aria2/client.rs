use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

use std::sync::OnceLock;

use std::sync::atomic::{AtomicU16, Ordering};

use crate::core::error::{AppError, AppResult};

static ARIA2_PORT: AtomicU16 = AtomicU16::new(6800);
static ARIA2_SECRET: tokio::sync::Mutex<Option<String>> = tokio::sync::Mutex::const_new(None);

static CLIENT: OnceLock<reqwest::Client> = OnceLock::new();

pub fn set_aria2_port(port: u16) {
    ARIA2_PORT.store(port, Ordering::SeqCst);
    crate::app_info!("Aria2::Client", "port_configured", json!({ "port": port }));
}

pub async fn set_aria2_secret(secret: String) {
    let mut s = ARIA2_SECRET.lock().await;
    *s = Some(secret);
}

pub fn get_aria2_port() -> u16 {
    ARIA2_PORT.load(Ordering::SeqCst)
}

fn get_client() -> &'static reqwest::Client {
    CLIENT.get_or_init(reqwest::Client::new)
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Aria2Task {
    pub gid: String,
    pub status: String,
    #[serde(rename = "totalLength")]
    pub total_length: String,
    #[serde(rename = "completedLength")]
    pub completed_length: String,
    #[serde(rename = "downloadSpeed")]
    pub download_speed: String,
    #[serde(rename = "uploadLength")]
    pub upload_length: String,
    #[serde(rename = "uploadSpeed")]
    pub upload_speed: String,
    #[serde(rename = "errorCode")]
    pub error_code: Option<String>,
    #[serde(rename = "errorMessage")]
    pub error_message: Option<String>,
    pub dir: String,
    pub files: Vec<Aria2File>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Aria2File {
    pub index: String,
    pub path: String,
    pub length: String,
    #[serde(rename = "completedLength")]
    pub completed_length: String,
    pub selected: String,
    pub uris: Vec<Aria2Uri>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Aria2Uri {
    pub uri: String,
    pub status: String,
}

async fn send_rpc_request<T>(method: &str, params: Vec<Value>) -> AppResult<T>
where
    T: serde::de::DeserializeOwned,
{
    let client = get_client();

    // Inject Token if exists
    let mut final_params = params;
    {
        let secret_guard = ARIA2_SECRET.lock().await;
        if let Some(secret) = secret_guard.as_ref() {
            // Aria2 requires token:<secret> as the FIRST parameter
            final_params.insert(0, json!(format!("token:{}", secret)));
        }
    }

    let payload = json!({
        "jsonrpc": "2.0",
        "id": "mua-app",
        "method": method,
        "params": final_params
    });

    crate::app_debug!("Aria2::Client", "rpc_request", json!({ "method": method }));

    let port = get_aria2_port();
    let url = format!("http://localhost:{}/jsonrpc", port);

    let response = client
        .post(&url)
        .json(&payload)
        .send()
        .await
        .map_err(|e| AppError::aria2(e.to_string()))?;

    if !response.status().is_success() {
        return Err(AppError::aria2(format!(
            "HTTP Error: {}",
            response.status()
        )));
    }

    let body: Value = response
        .json()
        .await
        .map_err(|e| AppError::aria2(e.to_string()))?;

    if let Some(error) = body.get("error") {
        return Err(AppError::aria2(error.to_string()));
    }

    if let Some(result) = body.get("result") {
        serde_json::from_value(result.clone())
            .map_err(|e| AppError::aria2(format!("Failed to parse result: {}", e)))
    } else {
        // Void 返回通常只是 null 或 "OK"，具体取决于方法，但对于 JSON-RPC 2.0，成功时必须包含 result。
        serde_json::from_value(Value::Null)
            .map_err(|e| AppError::aria2(format!("Missing result: {}", e)))
    }
}

pub async fn add_uri(urls: Vec<String>, options: Option<Value>) -> AppResult<String> {
    let mut params = vec![json!(urls)];
    if let Some(opts) = options {
        params.push(json!(opts));
    }

    send_rpc_request::<String>("aria2.addUri", params).await
}

pub async fn add_torrent(torrent: String, options: Option<Value>) -> AppResult<String> {
    let mut params = vec![json!(torrent), json!([])];
    if let Some(opts) = options {
        params.push(json!(opts));
    }

    send_rpc_request::<String>("aria2.addTorrent", params).await
}

pub async fn get_all_tasks() -> AppResult<Vec<Aria2Task>> {
    let client = get_client();

    // 获取 RPC Secret (如果配置了)
    let token: Option<String> = {
        let guard = ARIA2_SECRET.lock().await;
        guard.as_ref().map(|s| format!("token:{}", s))
    };

    // 构建 multicall 参数，每个子方法都需要注入 token
    let params = vec![
        json!({
            "methodName": "aria2.tellActive",
            "params": if let Some(ref t) = token { vec![json!(t)] } else { vec![] }
        }),
        json!({
            "methodName": "aria2.tellWaiting",
            "params": if let Some(ref t) = token {
                vec![json!(t), json!(0), json!(1000)]
            } else {
                vec![json!(0), json!(1000)]
            }
        }),
        json!({
            "methodName": "aria2.tellStopped",
            "params": if let Some(ref t) = token {
                vec![json!(t), json!(0), json!(1000)]
            } else {
                vec![json!(0), json!(1000)]
            }
        }),
    ];

    let payload = json!({
        "jsonrpc": "2.0",
        "id": "mua-app-multicall",
        "method": "system.multicall",
        "params": [params]
    });

    // Multicall 具有特定的返回结构，不进行转换很难清晰地使用通用辅助函数
    let port = get_aria2_port();
    let url = format!("http://localhost:{}/jsonrpc", port);

    match client.post(&url).json(&payload).send().await {
        Ok(response) => {
            if response.status().is_success() {
                let body: Value = response
                    .json()
                    .await
                    .map_err(|e| AppError::aria2(e.to_string()))?;

                if let Some(results) = body.get("result") {
                    if let Some(results_array) = results.as_array() {
                        let mut all_tasks = Vec::new();

                        for result_value in results_array {
                            if let Some(tasks_val) = result_value.get(0) {
                                // 每个调用的结果都包裹在一个数组中 [result]
                                if let Ok(tasks) =
                                    serde_json::from_value::<Vec<Aria2Task>>(tasks_val.clone())
                                {
                                    all_tasks.extend(tasks);
                                } else if let Ok(tasks) =
                                    serde_json::from_value::<Vec<Aria2Task>>(result_value.clone())
                                {
                                    all_tasks.extend(tasks);
                                }
                            }
                        }

                        Ok(all_tasks)
                    } else {
                        Err(AppError::aria2("Result is not an array"))
                    }
                } else if let Some(error) = body.get("error") {
                    Err(AppError::aria2(error.to_string()))
                } else {
                    Err(AppError::aria2("Unknown response format"))
                }
            } else {
                Err(AppError::aria2(format!(
                    "HTTP Error: {}",
                    response.status()
                )))
            }
        }
        Err(e) => Err(AppError::aria2(e.to_string())),
    }
}

pub async fn tell_active(keys: Vec<&str>) -> AppResult<Vec<Aria2Task>> {
    send_rpc_request::<Vec<Aria2Task>>("aria2.tellActive", vec![json!(keys)]).await
}

pub async fn tell_waiting(offset: usize, num: usize, keys: Vec<&str>) -> AppResult<Vec<Aria2Task>> {
    send_rpc_request::<Vec<Aria2Task>>(
        "aria2.tellWaiting",
        vec![json!(offset), json!(num), json!(keys)],
    )
    .await
}

pub async fn tell_stopped(offset: usize, num: usize, keys: Vec<&str>) -> AppResult<Vec<Aria2Task>> {
    send_rpc_request::<Vec<Aria2Task>>(
        "aria2.tellStopped",
        vec![json!(offset), json!(num), json!(keys)],
    )
    .await
}

pub async fn tell_status(gid: String, keys: Vec<&str>) -> AppResult<Aria2Task> {
    send_rpc_request::<Aria2Task>("aria2.tellStatus", vec![json!(gid), json!(keys)]).await
}

pub async fn pause(gid: String) -> AppResult<String> {
    send_rpc_request::<String>("aria2.pause", vec![json!(gid)]).await
}

pub async fn resume(gid: String) -> AppResult<String> {
    send_rpc_request::<String>("aria2.unpause", vec![json!(gid)]).await
}

pub async fn remove(gid: String) -> AppResult<String> {
    send_rpc_request::<String>("aria2.remove", vec![json!(gid)]).await
}

pub async fn purge(gid: String) -> AppResult<String> {
    send_rpc_request::<String>("aria2.removeDownloadResult", vec![json!(gid)]).await
}

pub async fn pause_all() -> AppResult<String> {
    send_rpc_request::<String>("aria2.pauseAll", vec![]).await
}

pub async fn unpause_all() -> AppResult<String> {
    send_rpc_request::<String>("aria2.unpauseAll", vec![]).await
}

pub async fn change_global_option(options: Value) -> AppResult<String> {
    send_rpc_request::<String>("aria2.changeGlobalOption", vec![options]).await
}
