use serde::Serialize;

mod add;
mod control;
mod query;
mod remove;

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BatchCommandResult {
    pub requested: usize,
    pub succeeded_gids: Vec<String>,
    pub failed_gids: Vec<String>,
    pub partial: bool,
}

pub use add::*;
pub use control::*;
pub use query::*;
pub use remove::*;
