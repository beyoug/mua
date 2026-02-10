//! 核心类型定义
//! 包含任务状态等共享类型

use serde::{Deserialize, Serialize};

/// 任务状态枚举
/// 与前端 `src/lib/types/download.ts::DownloadState` 保持一致
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum TaskState {
    /// 活跃中（下载中、做种中等）
    Active,
    /// 等待中
    Waiting,
    /// 已暂停
    Paused,
    /// 已完成
    Complete,
    /// 错误
    Error,
    /// 已移除/已取消
    Removed,
    /// 文件缺失
    Missing,
}

impl TaskState {
    /// 从 Aria2 状态字符串映射
    pub fn from_aria2_status(status: &str) -> Self {
        match status {
            "active" | "downloading" => Self::Active,
            "waiting" => Self::Waiting,
            "paused" => Self::Paused,
            "complete" | "completed" => Self::Complete,
            "error" => Self::Error,
            "removed" | "cancelled" => Self::Removed,
            "missing" => Self::Missing,
            _ => Self::Waiting,
        }
    }

    /// 转换为状态字符串（与 Aria2 原生状态对齐）
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Active => "active",
            Self::Waiting => "waiting",
            Self::Paused => "paused",
            Self::Complete => "complete",
            Self::Error => "error",
            Self::Removed => "removed",
            Self::Missing => "missing",
        }
    }

    /// 获取状态排序分数（用于任务列表排序）
    /// Active/Waiting/Paused 合并为同一分数，避免状态过渡期的排序抖动
    pub fn score(&self) -> i32 {
        match self {
            Self::Active | Self::Waiting | Self::Paused => 1,
            _ => 0,
        }
    }

    /// 判断是否为活跃状态（下载中、等待中、已暂停）
    pub fn is_active(&self) -> bool {
        matches!(self, Self::Active | Self::Waiting | Self::Paused)
    }

    /// 判断是否为终态（已完成、已取消、错误、缺失）
    pub fn is_terminal(&self) -> bool {
        matches!(
            self,
            Self::Complete | Self::Removed | Self::Error | Self::Missing
        )
    }
}

impl std::fmt::Display for TaskState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl From<&str> for TaskState {
    fn from(s: &str) -> Self {
        match s {
            "active" | "downloading" => Self::Active,
            "waiting" => Self::Waiting,
            "paused" => Self::Paused,
            "complete" | "completed" => Self::Complete,
            "error" => Self::Error,
            "removed" | "cancelled" => Self::Removed,
            "missing" => Self::Missing,
            _ => Self::Waiting,
        }
    }
}

impl From<String> for TaskState {
    fn from(s: String) -> Self {
        Self::from(s.as_str())
    }
}

/// 下载任务配置（用于前端传参）
#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct DownloadConfig {
    pub urls: Vec<String>,
    pub save_path: Option<String>,
    pub filename: Option<String>,
    pub user_agent: Option<String>,
    pub referer: Option<String>,
    pub headers: Option<String>,
    pub proxy: Option<String>,
    pub max_download_limit: Option<String>,
}
