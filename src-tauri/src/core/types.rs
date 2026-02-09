//! 核心类型定义
//! 包含任务状态等共享类型

use serde::{Deserialize, Serialize};

/// 任务状态枚举
/// 与前端 `src/lib/types/download.ts::DownloadState` 保持一致
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum TaskState {
    /// 下载中
    Downloading,
    /// 等待中
    Waiting,
    /// 已暂停
    Paused,
    /// 已完成
    Completed,
    /// 错误
    Error,
    /// 已取消
    Cancelled,
    /// 文件缺失
    Missing,
}

impl TaskState {
    /// 从 Aria2 状态字符串映射
    pub fn from_aria2_status(status: &str) -> Self {
        match status {
            "active" => Self::Downloading,
            "waiting" => Self::Waiting,
            "paused" => Self::Paused,
            "complete" => Self::Completed,
            "error" => Self::Error,
            "removed" | "cancelled" => Self::Cancelled,
            _ => Self::Waiting,
        }
    }

    /// 转换为前端状态字符串
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Downloading => "downloading",
            Self::Waiting => "waiting",
            Self::Paused => "paused",
            Self::Completed => "completed",
            Self::Error => "error",
            Self::Cancelled => "cancelled",
            Self::Missing => "missing",
        }
    }

    /// 获取状态排序分数（用于任务列表排序）
    pub fn score(&self) -> i32 {
        match self {
            Self::Downloading => 3,
            Self::Waiting => 2,
            Self::Paused => 1,
            _ => 0,
        }
    }

    /// 判断是否为活跃状态（下载中、等待中、已暂停）
    pub fn is_active(&self) -> bool {
        matches!(self, Self::Downloading | Self::Waiting | Self::Paused)
    }

    /// 判断是否为终态（已完成、已取消、错误、缺失）
    pub fn is_terminal(&self) -> bool {
        matches!(self, Self::Completed | Self::Cancelled | Self::Error | Self::Missing)
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
            "downloading" => Self::Downloading,
            "waiting" => Self::Waiting,
            "paused" => Self::Paused,
            "completed" => Self::Completed,
            "error" => Self::Error,
            "cancelled" => Self::Cancelled,
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
