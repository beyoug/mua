//! 命令模块统一入口
//! 按业务域拆分为三个子模块

mod aria2_commands;
mod config_commands;
mod task_commands;

// 重新导出所有命令供 lib.rs 注册
pub use aria2_commands::*;
pub use config_commands::*;
pub use task_commands::*;
