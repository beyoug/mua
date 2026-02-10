use crate::core::error::{AppError, AppResult};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;

#[derive(Debug, Deserialize)]
struct Method {
    // files: Option<Vec<File>>,
    name: String,
    // length: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TorrentInfo {
    pub name: String,
    pub files: Vec<TorrentFile>,
    pub total_length: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TorrentFile {
    pub path: String,
    pub length: u64,
    pub index: usize,
}

// Bencode 结构定义
#[derive(Deserialize, Debug)]
struct Metainfo {
    info: Info,
}

#[derive(Deserialize, Debug)]
struct Info {
    name: String,
    #[serde(default)]
    length: Option<u64>,
    #[serde(default)]
    files: Option<Vec<BencodeFile>>,
}

#[derive(Deserialize, Debug)]
struct BencodeFile {
    length: u64,
    path: Vec<String>,
}

pub fn parse_torrent_file<P: AsRef<Path>>(path: P) -> AppResult<TorrentInfo> {
    let content = std::fs::read(path).map_err(|e| AppError::Fs(e.to_string()))?;
    
    let metainfo: Metainfo = serde_bencode::from_bytes(&content)
        .map_err(|e| AppError::Validation(format!("无效的种子文件: {}", e)))?;

    let info = metainfo.info;
    let mut files = Vec::new();
    let mut total_length = 0;

    if let Some(file_list) = info.files {
        // 多文件模式
        for (idx, f) in file_list.into_iter().enumerate() {
            // let file_path = f.path.join("/");
            let file_path = f.path.join(std::path::MAIN_SEPARATOR.to_string().as_str());
            files.push(TorrentFile {
                path: file_path,
                length: f.length,
                index: idx,
            });
            total_length += f.length;
        }
    } else if let Some(length) = info.length {
        // 单文件模式
        files.push(TorrentFile {
            path: info.name.clone(),
            length,
            index: 0,
        });
        total_length = length;
    } else {
        return Err(AppError::Validation("种子文件既没有单文件长度也没有文件列表".into()));
    }

    Ok(TorrentInfo {
        name: info.name,
        files,
        total_length,
    })
}
