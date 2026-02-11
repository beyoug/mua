use crate::aria2::client as aria2_client;
use crate::core::error::{AppError, AppResult};
use crate::core::store::{PersistedTask, TaskStore};
use crate::core::types::{DownloadConfig, TaskState};
use crate::utils;
use base64::Engine as _;
use chrono::Local;
use futures::future::join_all;
use serde_json::json;

#[tauri::command]
pub async fn add_download_tasks(
    state: tauri::State<'_, TaskStore>,
    configs: Vec<DownloadConfig>,
) -> AppResult<Vec<Option<String>>> {
    let futures = configs
        .into_iter()
        .map(|cfg| add_download_task_inner(&state, cfg));

    let results = join_all(futures).await;

    let mut gids = Vec::new();
    let mut errors = Vec::new();

    for res in results {
        match res {
            Ok(gid) => gids.push(Some(gid)),
            Err(e) => {
                errors.push(e.to_string());
                gids.push(None);
            }
        }
    }

    if !errors.is_empty() {
        crate::app_warn!(
            "Core::TaskAdd",
            "batch_add_partial_failure",
            json!({ "error_count": errors.len(), "errors": errors })
        );
    }

    Ok(gids)
}

#[tauri::command]
pub async fn parse_torrent(path: String) -> AppResult<crate::core::torrent::TorrentInfo> {
    crate::core::torrent::parse_torrent_file(&path)
}

async fn add_download_task_inner(state: &TaskStore, cfg: DownloadConfig) -> AppResult<String> {
    crate::app_info!(
        "Core::TaskAdd",
        "add_task_requested",
        json!({ "url_count": cfg.urls.len(), "is_torrent": cfg.torrent_config.is_some() })
    );

    if let Some(ref torrent_cfg) = cfg.torrent_config {
        return add_torrent_task_inner(
            state,
            torrent_cfg.path.clone(),
            torrent_cfg.select_file.clone(),
            &cfg,
        )
        .await;
    }

    for url in &cfg.urls {
        if !utils::is_valid_url(url) {
            return Err(AppError::validation(format!("无效的 URL: {}", url)));
        }
    }

    let deduced_name = utils::deduce_filename(cfg.filename.clone(), &cfg.urls);

    let resolved_save_path = if let Some(ref path) = cfg.save_path {
        utils::resolve_path(path)
    } else {
        ".".to_string()
    };

    let active_names = state.get_active_filenames();
    let unique_filename =
        utils::get_unique_filename(&resolved_save_path, &deduced_name, &active_names);

    let (options, final_save_path) = utils::build_aria2_options(
        cfg.save_path.clone(),
        Some(unique_filename.clone()),
        cfg.user_agent.clone(),
        cfg.referer.clone(),
        cfg.headers.clone(),
        cfg.proxy.clone(),
        cfg.max_download_limit.clone(),
    );

    match aria2_client::add_uri(cfg.urls.clone(), Some(options)).await {
        Ok(gid) => {
            let task = create_persisted_task(
                gid.clone(),
                unique_filename,
                cfg.urls.get(0).cloned().unwrap_or_default(),
                final_save_path,
                &cfg,
            );
            state.add_task(task);
            Ok(gid)
        }
        Err(e) => Err(e),
    }
}

fn create_persisted_task(
    gid: String,
    filename: String,
    url: String,
    save_path: String,
    cfg: &DownloadConfig,
) -> PersistedTask {
    PersistedTask {
        gid,
        filename,
        url,
        save_path,
        added_at: Local::now().to_rfc3339(),
        state: TaskState::Waiting,
        total_length: "0".to_string(),
        completed_length: "0".to_string(),
        download_speed: "0 B/s".to_string(),
        completed_at: None,
        error_message: "".to_string(),
        user_agent: cfg.user_agent.clone().unwrap_or_default(),
        referer: cfg.referer.clone().unwrap_or_default(),
        proxy: cfg.proxy.clone().unwrap_or_default(),
        headers: cfg
            .headers
            .clone()
            .unwrap_or_default()
            .split(|c| c == ';' || c == '\n')
            .filter(|s| !s.trim().is_empty())
            .map(|s| s.trim().to_string())
            .collect(),
        max_download_limit: cfg.max_download_limit.clone().unwrap_or_default(),
    }
}

async fn add_torrent_task_inner(
    state: &TaskStore,
    path: String,
    select_file: Option<String>,
    base_cfg: &DownloadConfig,
) -> AppResult<String> {
    let content = std::fs::read(&path).map_err(|e| AppError::Fs(e.to_string()))?;
    let torrent_b64 = base64::engine::general_purpose::STANDARD.encode(&content);

    let (mut options_val, final_save_path) = utils::build_aria2_options(
        base_cfg.save_path.clone(),
        None,
        base_cfg.user_agent.clone(),
        base_cfg.referer.clone(),
        base_cfg.headers.clone(),
        base_cfg.proxy.clone(),
        base_cfg.max_download_limit.clone(),
    );

    if let Some(opts) = options_val.as_object_mut() {
        if let Some(sf) = select_file {
            opts.insert("select-file".to_string(), serde_json::Value::String(sf));
        }
    }

    match aria2_client::add_torrent(torrent_b64, Some(options_val)).await {
        Ok(gid) => {
            let task = create_persisted_task(
                gid.clone(),
                std::path::Path::new(&path)
                    .file_name()
                    .unwrap_or_default()
                    .to_string_lossy()
                    .to_string(),
                format!("file://{}", path),
                final_save_path,
                base_cfg,
            );
            state.add_task(task);
            Ok(gid)
        }
        Err(e) => Err(e),
    }
}
