//! 应用统一错误类型
//! 提供标准化的错误处理，支持 Tauri 命令返回

use serde::Serialize;
use std::fmt;

/// 应用错误枚举
/// 实现了 `serde::Serialize` 以支持 Tauri 命令返回
#[derive(Debug, Clone, Serialize)]
#[serde(tag = "type", content = "message")]
pub enum AppError {
    /// Aria2 RPC 通信错误
    Aria2(String),
    /// 配置相关错误
    Config(String),
    /// 文件系统错误
    Fs(String),
    /// 任务不存在
    TaskNotFound(String),
    /// 验证错误
    Validation(String),
    /// 通用错误
    Other(String),
}

impl fmt::Display for AppError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Aria2(msg) => write!(f, "Aria2 错误: {}", msg),
            Self::Config(msg) => write!(f, "配置错误: {}", msg),
            Self::Fs(msg) => write!(f, "文件系统错误: {}", msg),
            Self::TaskNotFound(msg) => write!(f, "任务不存在: {}", msg),
            Self::Validation(msg) => write!(f, "验证错误: {}", msg),
            Self::Other(msg) => write!(f, "{}", msg),
        }
    }
}

impl std::error::Error for AppError {}

// 实现常见错误类型转换，支持 ? 操作符

impl From<std::io::Error> for AppError {
    fn from(err: std::io::Error) -> Self {
        Self::Fs(err.to_string())
    }
}

impl From<serde_json::Error> for AppError {
    fn from(err: serde_json::Error) -> Self {
        Self::Config(err.to_string())
    }
}

impl From<tauri::Error> for AppError {
    fn from(err: tauri::Error) -> Self {
        Self::Other(err.to_string())
    }
}

impl From<String> for AppError {
    fn from(msg: String) -> Self {
        Self::Other(msg)
    }
}

impl From<&str> for AppError {
    fn from(msg: &str) -> Self {
        Self::Other(msg.to_string())
    }
}

/// 便捷类型别名
pub type AppResult<T> = Result<T, AppError>;

// 辅助宏用于快速构建错误
#[macro_export]
macro_rules! app_err {
    ($variant:ident, $msg:expr) => {
        $crate::core::error::AppError::$variant($msg.to_string())
    };
    ($msg:expr) => {
        $crate::core::error::AppError::Other($msg.to_string())
    };
}

// 辅助方法用于从 Aria2 错误转换
impl AppError {
    pub fn aria2<S: Into<String>>(msg: S) -> Self {
        Self::Aria2(msg.into())
    }

    pub fn config<S: Into<String>>(msg: S) -> Self {
        Self::Config(msg.into())
    }

    pub fn task_not_found<S: Into<String>>(gid: S) -> Self {
        Self::TaskNotFound(gid.into())
    }

    pub fn validation<S: Into<String>>(msg: S) -> Self {
        Self::Validation(msg.into())
    }

    pub fn io<S: Into<String>>(msg: S) -> Self {
        Self::Fs(msg.into())
    }
}
