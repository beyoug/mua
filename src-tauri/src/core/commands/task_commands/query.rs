use crate::core::error::AppResult;
use crate::core::store::TaskStore;
use crate::core::sync::FrontendTask;
use tauri::AppHandle;

#[tauri::command]
pub async fn get_tasks(
    state: tauri::State<'_, TaskStore>,
    app_handle: AppHandle,
) -> AppResult<Vec<FrontendTask>> {
    crate::core::sync::sync_tasks(&state, &app_handle).await
}
