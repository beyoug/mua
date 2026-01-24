use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::collections::HashMap;

const ARIA2_RPC_URL: &str = "http://localhost:6800/jsonrpc";



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

pub async fn add_uri(urls: Vec<String>, options: Option<HashMap<String, String>>) -> Result<String, String> {
    let client = reqwest::Client::new();
    
    let mut params = vec![json!(urls)];
    if let Some(opts) = options {
        params.push(json!(opts));
    }
    
    // 构建 JSON-RPC 请求
    let payload = json!({
        "jsonrpc": "2.0",
        "id": "mua-app",
        "method": "aria2.addUri",
        "params": params
    });



    log::info!("Sending payload to Aria2: {:?}", payload);

    match client.post(ARIA2_RPC_URL).json(&payload).send().await {
        Ok(response) => {
            log::info!("Aria2 response status: {}", response.status());
            if response.status().is_success() {
                let body: Value = response.json().await.map_err(|e| e.to_string())?;
                if let Some(gid) = body.get("result") {
                    Ok(gid.as_str().unwrap_or_default().to_string())
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

pub async fn get_all_tasks() -> Result<Vec<Aria2Task>, String> {
    let client = reqwest::Client::new();
    
    // We need to call tellActive, tellWaiting, and tellStopped
    // For simplicity, let's just make 3 requests. 
    // Optimization: we could use system.multicall to do it in one request, but simple is better for now.

    let active = call_aria2_method(&client, "aria2.tellActive", vec![]).await?;
    let waiting = call_aria2_method(&client, "aria2.tellWaiting", vec![json!(0), json!(1000)]).await?;
    let stopped = call_aria2_method(&client, "aria2.tellStopped", vec![json!(0), json!(1000)]).await?;

    let mut all = Vec::new();
    all.extend(active);
    all.extend(waiting);
    all.extend(stopped);

    Ok(all)
}

pub async fn pause(gid: String) -> Result<String, String> {
    call_aria2_void_method("aria2.pause", vec![json!(gid)]).await
}

pub async fn resume(gid: String) -> Result<String, String> {
    call_aria2_void_method("aria2.unpause", vec![json!(gid)]).await
}

pub async fn remove(gid: String) -> Result<String, String> {
    // aria2.remove removes the download. If it's active, it changes status to 'removed'.
    call_aria2_void_method("aria2.remove", vec![json!(gid)]).await
}

pub async fn purge(gid: String) -> Result<String, String> {
    // aria2.removeDownloadResult removes the task from memory (for completed/error/removed tasks)
    call_aria2_void_method("aria2.removeDownloadResult", vec![json!(gid)]).await
}

async fn call_aria2_void_method(method: &str, params: Vec<Value>) -> Result<String, String> {
    let client = reqwest::Client::new();
    let payload = json!({
        "jsonrpc": "2.0",
        "id": "mua-app",
        "method": method,
        "params": params
    });

    match client.post(ARIA2_RPC_URL).json(&payload).send().await {
        Ok(response) => {
             if response.status().is_success() {
                let body: Value = response.json().await.map_err(|e| e.to_string())?;
                if let Some(result) = body.get("result") {
                    Ok(result.as_str().unwrap_or("OK").to_string())
                } else if let Some(error) = body.get("error") {
                    Err(error.to_string())
                } else {
                    Ok("OK".to_string())
                }
             } else {
                 Err(format!("HTTP Error: {}", response.status()))
             }
        },
        Err(e) => Err(e.to_string())
    }
}

async fn call_aria2_method(client: &reqwest::Client, method: &str, params: Vec<Value>) -> Result<Vec<Aria2Task>, String> {
    let payload = json!({
        "jsonrpc": "2.0",
        "id": "mua-app",
        "method": method,
        "params": params
    });

    match client.post(ARIA2_RPC_URL).json(&payload).send().await {
        Ok(response) => {
             if response.status().is_success() {
                let body: Value = response.json().await.map_err(|e| e.to_string())?;
                if let Some(result) = body.get("result") {
                    let tasks: Vec<Aria2Task> = serde_json::from_value(result.clone())
                        .map_err(|e| format!("Failed to parse tasks: {}", e))?;
                    Ok(tasks)
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
