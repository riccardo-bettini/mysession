use crate::ai::catalog::{get_ai_model_catalog, AiModelCatalogItem};
use crate::ai::cli_bridge::detect_cli_agents;
use crate::ai::hardware::{detect_system_hardware, SystemHardwareInfo};
use crate::ai::runtime::AiRuntimeStatus;
use crate::ai::{
    get_download_manager, get_runtime_manager, parse_timetable_image_with_ai,
    parse_timetable_prompt_with_ai, process_natural_command, AiAssistantResponse,
};
use crate::db::DatabaseManager;
use crate::models::{CliAgentInfo, ExtractedLessonItem, SessionSnapshot};
use tauri::{AppHandle, State};

#[tauri::command]
pub fn get_system_hardware_info() -> Result<SystemHardwareInfo, String> {
    Ok(detect_system_hardware())
}

#[tauri::command]
pub fn get_cli_agents_status(db: State<'_, DatabaseManager>) -> Result<Vec<CliAgentInfo>, String> {
    let settings = db
        .with_conn(|conn| DatabaseManager::get_settings(conn))
        .map_err(|e| e.to_string())?;
    Ok(detect_cli_agents(&settings.ai_custom_cli_command))
}

#[tauri::command]
pub fn get_ai_models_status(db: State<'_, DatabaseManager>) -> Result<Vec<AiModelCatalogItem>, String> {
    let app_data_dir = DatabaseManager::resolve_app_data_dir();
    let models_dir = app_data_dir.join("models");
    let settings = db
        .with_conn(|conn| DatabaseManager::get_settings(conn))
        .map_err(|e| e.to_string())?;

    let catalog = get_ai_model_catalog();
    Ok(catalog
        .into_iter()
        .map(|mut item| {
            item.is_downloaded = models_dir.join(&item.file_name).exists();
            item.is_active = settings.ai_local_model_id == item.id;
            item
        })
        .collect())
}

#[tauri::command]
pub fn start_ai_model_download(app_handle: AppHandle, model_id: String) -> Result<(), String> {
    let app_data_dir = DatabaseManager::resolve_app_data_dir();
    get_download_manager().start_download(app_handle, app_data_dir, model_id)
}

#[tauri::command]
pub fn cancel_ai_model_download(model_id: String) -> Result<(), String> {
    get_download_manager().cancel(&model_id);
    Ok(())
}

#[tauri::command]
pub fn delete_ai_model(model_id: String) -> Result<(), String> {
    let app_data_dir = DatabaseManager::resolve_app_data_dir();
    let models_dir = app_data_dir.join("models");
    let item = get_ai_model_catalog()
        .into_iter()
        .find(|m| m.id == model_id)
        .ok_or_else(|| format!("Modello '{}' non trovato", model_id))?;

    let m_path = models_dir.join(&item.file_name);
    let mm_path = models_dir.join(&item.mmproj_file_name);
    if m_path.exists() {
        let _ = std::fs::remove_file(m_path);
    }
    if mm_path.exists() {
        let _ = std::fs::remove_file(mm_path);
    }
    Ok(())
}

#[tauri::command]
pub fn get_ai_runtime_status() -> Result<AiRuntimeStatus, String> {
    Ok(get_runtime_manager().get_status())
}

#[tauri::command]
pub fn stop_ai_runtime() -> Result<(), String> {
    get_runtime_manager().stop();
    Ok(())
}

#[tauri::command]
pub fn start_ai_runtime(db: State<'_, DatabaseManager>, model_id: Option<String>) -> Result<(), String> {
    let app_data_dir = DatabaseManager::resolve_app_data_dir();
    let settings = db
        .with_conn(|conn| DatabaseManager::get_settings(conn))
        .map_err(|e| e.to_string())?;
    let target = model_id.unwrap_or(settings.ai_local_model_id);
    get_runtime_manager().ensure_started(&app_data_dir, &target, "auto", settings.ai_idle_timeout_minutes)?;
    Ok(())
}

#[tauri::command]
pub fn execute_ai_command(
    db: State<'_, DatabaseManager>,
    session_id: String,
    prompt: String,
    image_base64: Option<String>,
    mime_type: Option<String>,
) -> Result<AiAssistantResponse, String> {
    db.with_conn(|conn| {
        process_natural_command(
            conn,
            &session_id,
            &prompt,
            image_base64.as_deref(),
            mime_type.as_deref(),
        )
        .map_err(crate::db::DbError::Logic)
    })
    .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn undo_ai_action(
    db: State<'_, DatabaseManager>,
    snapshot_id: String,
) -> Result<SessionSnapshot, String> {
    db.with_conn(|conn| {
        let session_id = DatabaseManager::restore_ai_undo_snapshot(conn, &snapshot_id)?;
        let settings = DatabaseManager::get_settings(conn)?;
        DatabaseManager::build_session_snapshot(conn, &session_id, &settings)
    })
    .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn parse_timetable_image(
    db: State<'_, DatabaseManager>,
    image_base64: String,
    mime_type: String,
) -> Result<Vec<ExtractedLessonItem>, String> {
    let settings = db
        .with_conn(|conn| DatabaseManager::get_settings(conn))
        .map_err(|e| e.to_string())?;
    parse_timetable_image_with_ai(&settings, &image_base64, &mime_type)
}

#[tauri::command]
pub fn parse_timetable_prompt(
    db: State<'_, DatabaseManager>,
    prompt: String,
) -> Result<Vec<ExtractedLessonItem>, String> {
    let settings = db
        .with_conn(|conn| DatabaseManager::get_settings(conn))
        .map_err(|e| e.to_string())?;
    parse_timetable_prompt_with_ai(&settings, &prompt)
}
