use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::collections::HashMap;
use std::sync::OnceLock;

use std::sync::atomic::{AtomicU16, Ordering};

static ARIA2_PORT: AtomicU16 = AtomicU16::new(6800);

static CLIENT: OnceLock<reqwest::Client> = OnceLock::new();

pub fn set_aria2_port(port: u16) {
    ARIA2_PORT.store(port, Ordering::SeqCst);
    log::info!("Aria2 client configured to use port: {}", port);
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


async fn send_rpc_request<T>(method: &str, params: Vec<Value>) -> Result<T, String> 
where
    T: serde::de::DeserializeOwned,
{
    let client = get_client();
    let payload = json!({
        "jsonrpc": "2.0",
        "id": "mua-app",
        "method": method,
        "params": params
    });

    log::debug!("Sending RPC request: {} {:?}", method, params);

    let port = get_aria2_port();
    let url = format!("http://localhost:{}/jsonrpc", port);

    let response = client.post(&url)
        .json(&payload)
        .send()
        .await
        .map_err(|e| e.to_string())?;

    if !response.status().is_success() {
        return Err(format!("HTTP Error: {}", response.status()));
    }

    let body: Value = response.json().await.map_err(|e| e.to_string())?;
    
    if let Some(error) = body.get("error") {
        return Err(error.to_string());
    }

    if let Some(result) = body.get("result") {
        serde_json::from_value(result.clone()).map_err(|e| format!("Failed to parse result: {}", e))
    } else {
        // Void 返回通常只是 null 或 "OK"，具体取决于方法，但对于 JSON-RPC 2.0，成功时必须包含 result。
        // 如果 result 为 null，我们尝试从 null 反序列化 T。
        serde_json::from_value(Value::Null).map_err(|e| format!("Missing result: {}", e))
    }
}

pub async fn add_uri(urls: Vec<String>, options: Option<HashMap<String, String>>) -> Result<String, String> {
    let mut params = vec![json!(urls)];
    if let Some(opts) = options {
        params.push(json!(opts));
    }
    
    send_rpc_request::<String>("aria2.addUri", params).await
}

pub async fn get_all_tasks() -> Result<Vec<Aria2Task>, String> {
    let client = get_client();

    // 构建 multicall 参数
    let params = vec![
        json!({
            "methodName": "aria2.tellActive",
            "params": []
        }),
        json!({
            "methodName": "aria2.tellWaiting",
            "params": [0, 1000]
        }),
        json!({
            "methodName": "aria2.tellStopped",
            "params": [0, 1000]
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
                let body: Value = response.json().await.map_err(|e| e.to_string())?;
                
                if let Some(results) = body.get("result") {
                    if let Some(results_array) = results.as_array() {
                        let mut all_tasks = Vec::new();
                        
                        for result_value in results_array {
                             if let Some(tasks_val) = result_value.get(0) { // 每个调用的结果都包裹在一个数组中 [result]
                                 if let Ok(tasks) = serde_json::from_value::<Vec<Aria2Task>>(tasks_val.clone()) {
                                     all_tasks.extend(tasks);
                                 } else if let Ok(tasks) = serde_json::from_value::<Vec<Aria2Task>>(result_value.clone()) {
                                     all_tasks.extend(tasks);
                                 }
                             }
                        }
                        
                        Ok(all_tasks)
                    } else {
                        Err("Result is not an array".to_string())
                    }
                } else if let Some(error) = body.get("error") {
                    Err(error.to_string())
                } else {
                     Err("Unknown response format".to_string())
                }
            } else {
                Err(format!("HTTP Error: {}", response.status()))
            }
        },
        Err(e) => Err(e.to_string())
    }
}

pub async fn pause(gid: String) -> Result<String, String> {
    send_rpc_request::<String>("aria2.pause", vec![json!(gid)]).await
}

pub async fn resume(gid: String) -> Result<String, String> {
    send_rpc_request::<String>("aria2.unpause", vec![json!(gid)]).await
}

pub async fn remove(gid: String) -> Result<String, String> {
    send_rpc_request::<String>("aria2.remove", vec![json!(gid)]).await
}

pub async fn purge(gid: String) -> Result<String, String> {
    send_rpc_request::<String>("aria2.removeDownloadResult", vec![json!(gid)]).await
}

pub async fn pause_all() -> Result<String, String> {
    send_rpc_request::<String>("aria2.pauseAll", vec![]).await
}

pub async fn unpause_all() -> Result<String, String> {
    send_rpc_request::<String>("aria2.unpauseAll", vec![]).await
}
