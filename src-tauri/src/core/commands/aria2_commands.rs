//! Aria2 相关命令
//! 包含 Aria2 配置与版本信息操作

use crate::core::error::{AppError, AppResult};
use tauri::{AppHandle, Manager};

#[tauri::command]
pub async fn get_aria2_config_path(app: AppHandle) -> AppResult<String> {
    let config_dir = app
        .path()
        .app_config_dir()
        .map_err(|e| AppError::config(e.to_string()))?;
    let path = config_dir.join("aria2.conf");
    Ok(path.to_string_lossy().to_string())
}

#[tauri::command]
pub async fn read_aria2_config(app: AppHandle) -> AppResult<String> {
    let config_dir = app
        .path()
        .app_config_dir()
        .map_err(|e| AppError::config(e.to_string()))?;
    let path = config_dir.join("aria2.conf");

    if path.exists() {
        std::fs::read_to_string(path).map_err(|e| AppError::Fs(e.to_string()))
    } else {
        Ok("".to_string())
    }
}

#[tauri::command]
pub async fn import_aria2_config(app: AppHandle, path: String) -> AppResult<()> {
    let config_dir = app
        .path()
        .app_config_dir()
        .map_err(|e| AppError::config(e.to_string()))?;

    if !config_dir.exists() {
        std::fs::create_dir_all(&config_dir)?;
    }

    let dest_path = config_dir.join("aria2.conf");
    std::fs::copy(&path, &dest_path)?;

    Ok(())
}

#[tauri::command]
pub async fn import_custom_binary(app: AppHandle, file_path: String) -> AppResult<String> {
    #[cfg(unix)]
    use std::os::unix::fs::PermissionsExt;
    use std::path::Path;

    // 1. 准备目标路径
    let config_dir = app
        .path()
        .app_config_dir()
        .map_err(|e| AppError::config(e.to_string()))?;
    let bin_dir = config_dir.join("custom-bin");
    if !bin_dir.exists() {
        std::fs::create_dir_all(&bin_dir)?;
    }

    let target_name = if cfg!(windows) {
        "aria2c.exe"
    } else {
        "aria2c"
    };
    let target_path = bin_dir.join(target_name);

    // 2. 检查约束
    let source_path = Path::new(&file_path);
    if !source_path.exists() {
        return Err(AppError::validation("源文件不存在"));
    }

    // 3. 复制文件
    std::fs::copy(source_path, &target_path)?;

    let binary_hash = crate::utils::sha256_hex_of_file(&target_path)
        .map_err(|e| AppError::validation(format!("读取二进制失败: {}", e)))?;

    // 4. 设置权限 (Unix)
    #[cfg(unix)]
    {
        let mut perms = std::fs::metadata(&target_path)?.permissions();
        perms.set_mode(0o755);
        std::fs::set_permissions(&target_path, perms)?;
    }

    // 5. 验证 (空运行)
    let output = std::process::Command::new(&target_path)
        .arg("--version")
        .output()
        .map_err(|e| AppError::validation(format!("验证失败 (执行错误): {}", e)))?;

    if !output.status.success() {
        let _ = std::fs::remove_file(&target_path);
        return Err(AppError::validation("验证失败：完整性检查返回非零退出代码"));
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    if !stdout.contains("aria2 version") {
        let _ = std::fs::remove_file(&target_path);
        return Err(AppError::validation(
            "验证失败：不是有效的 aria2 二进制文件",
        ));
    }

    let version_line = stdout.lines().next().unwrap_or("Unknown");

    let mut config = crate::core::config::load_config(&app);
    config.custom_aria2_hash = Some(binary_hash);
    config.custom_aria2_trusted = false;
    config.use_custom_aria2 = false;
    crate::core::config::save_config(&app, &config)?;

    if let Some(state) = app.try_state::<crate::core::config::ConfigState>() {
        if let Ok(mut lock) = state.config.lock() {
            *lock = config;
        }
    }

    Ok(version_line.to_string())
}

#[derive(serde::Serialize)]
pub struct Aria2VersionInfo {
    pub version: String,
    pub is_custom: bool,
    pub path: String,
    pub custom_binary_exists: bool,
    pub custom_binary_version: Option<String>,
    pub custom_binary_trusted: bool,
    pub custom_binary_hash_match: bool,
    pub custom_binary_security_status: String,
}

#[tauri::command]
pub async fn get_aria2_version_info(app: AppHandle) -> AppResult<Aria2VersionInfo> {
    let config_dir = app
        .path()
        .app_config_dir()
        .map_err(|e| AppError::config(e.to_string()))?;
    let target_name = if cfg!(windows) {
        "aria2c.exe"
    } else {
        "aria2c"
    };
    let custom_path = config_dir.join("custom-bin").join(target_name);

    let config = crate::core::config::load_config(&app);
    let expected_hash = config.custom_aria2_hash.clone();
    let trusted = config.custom_aria2_trusted;

    // 检查自定义二进制文件是否存在并获取其版本
    let (has_custom, custom_ver): (bool, Option<String>) = if custom_path.exists() {
        match std::process::Command::new(&custom_path)
            .arg("--version")
            .output()
        {
            Ok(output) if output.status.success() => {
                let stdout = String::from_utf8_lossy(&output.stdout);
                let ver = stdout.lines().next().unwrap_or("Unknown").to_string();
                (true, Some(ver))
            }
            _ => (false, None),
        }
    } else {
        (false, None)
    };

    let hash_match = if has_custom {
        if let Some(expected) = expected_hash {
            if expected.is_empty() {
                false
            } else {
                match crate::utils::sha256_hex_of_file(&custom_path) {
                    Ok(actual) => actual == expected,
                    Err(_) => false,
                }
            }
        } else {
            false
        }
    } else {
        false
    };

    let effective_trusted = trusted && hash_match;
    let security_status = if !has_custom {
        "missing"
    } else if !trusted {
        "untrusted"
    } else if !hash_match {
        "hash_mismatch"
    } else {
        "trusted"
    }
    .to_string();

    // 确定活跃内核的信息
    if config.use_custom_aria2 && has_custom && effective_trusted {
        Ok(Aria2VersionInfo {
            version: custom_ver.clone().unwrap_or_default(),
            is_custom: true,
            path: custom_path.to_string_lossy().to_string(),
            custom_binary_exists: true,
            custom_binary_version: custom_ver,
            custom_binary_trusted: effective_trusted,
            custom_binary_hash_match: hash_match,
            custom_binary_security_status: security_status,
        })
    } else {
        Ok(Aria2VersionInfo {
            version: "Built-in (1.36.0)".to_string(),
            is_custom: false,
            path: "Embedded Sidecar".to_string(),
            custom_binary_exists: has_custom,
            custom_binary_version: custom_ver,
            custom_binary_trusted: effective_trusted,
            custom_binary_hash_match: hash_match,
            custom_binary_security_status: security_status,
        })
    }
}

#[tauri::command]
pub async fn trust_custom_aria2_binary(app: AppHandle) -> AppResult<()> {
    let config_dir = app
        .path()
        .app_config_dir()
        .map_err(|e| AppError::config(e.to_string()))?;

    let target_name = if cfg!(windows) {
        "aria2c.exe"
    } else {
        "aria2c"
    };

    let custom_path = config_dir.join("custom-bin").join(target_name);
    if !custom_path.exists() {
        return Err(AppError::validation("未找到已导入的自定义内核文件"));
    }

    let actual_hash = crate::utils::sha256_hex_of_file(&custom_path)
        .map_err(|e| AppError::validation(format!("读取二进制失败: {}", e)))?;

    let mut config = crate::core::config::load_config(&app);
    config.custom_aria2_hash = Some(actual_hash);
    config.custom_aria2_trusted = true;
    crate::core::config::save_config(&app, &config)?;

    if let Some(state) = app.try_state::<crate::core::config::ConfigState>() {
        if let Ok(mut lock) = state.config.lock() {
            *lock = config;
        }
    }

    Ok(())
}
