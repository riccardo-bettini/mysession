use crate::ai::catalog::{get_ai_model_catalog, AiModelCatalogItem};
use crate::ai::hardware::{detect_system_hardware, SystemHardwareInfo};
use crate::ai::runtime::AiRuntimeStatus;
use crate::ai::{get_download_manager, get_runtime_manager};
use crate::db::DatabaseManager;
use std::fs;
use tauri::{AppHandle, State};

#[tauri::command]
pub fn get_system_hardware_info() -> Result<SystemHardwareInfo, String> {
    Ok(detect_system_hardware())
}

#[tauri::command]
pub fn get_ai_models_status(db: State<'_, DatabaseManager>) -> Result<Vec<AiModelCatalogItem>, String> {
    let app_data_dir = DatabaseManager::resolve_app_data_dir();
    let models_dir = app_data_dir.join("models");

    let settings = db
        .with_conn(|conn| DatabaseManager::get_settings(conn))
        .map_err(|e| e.to_string())?;

    let catalog = get_ai_model_catalog();
    let enriched = catalog
        .into_iter()
        .map(|mut item| {
            let model_file = models_dir.join(&item.file_name);
            item.is_downloaded = model_file.exists();
            item.is_active = settings.ai_local_model_id == item.id;
            item
        })
        .collect();

    Ok(enriched)
}

#[tauri::command]
pub fn start_ai_model_download(app_handle: AppHandle, model_id: String) -> Result<(), String> {
    let app_data_dir = DatabaseManager::resolve_app_data_dir();
    let downloader = get_download_manager();
    downloader.start_download(app_handle, app_data_dir, model_id)
}

#[tauri::command]
pub fn cancel_ai_model_download(model_id: String) -> Result<(), String> {
    let downloader = get_download_manager();
    downloader.cancel(&model_id);
    Ok(())
}

#[tauri::command]
pub fn delete_ai_model(model_id: String) -> Result<(), String> {
    let app_data_dir = DatabaseManager::resolve_app_data_dir();
    let models_dir = app_data_dir.join("models");

    let catalog = get_ai_model_catalog();
    let item = catalog
        .into_iter()
        .find(|m| m.id == model_id)
        .ok_or_else(|| format!("Modello '{}' non trovato", model_id))?;

    let model_file = models_dir.join(&item.file_name);
    let mmproj_file = models_dir.join(&item.mmproj_file_name);

    if model_file.exists() {
        let _ = fs::remove_file(model_file);
    }
    if mmproj_file.exists() {
        let _ = fs::remove_file(mmproj_file);
    }

    Ok(())
}

#[tauri::command]
pub fn get_ai_runtime_status() -> Result<AiRuntimeStatus, String> {
    let runtime = get_runtime_manager();
    Ok(runtime.get_status())
}

#[tauri::command]
pub fn stop_ai_runtime() -> Result<(), String> {
    let runtime = get_runtime_manager();
    runtime.stop();
    Ok(())
}

#[tauri::command]
pub fn start_ai_runtime(db: State<'_, DatabaseManager>, model_id: Option<String>) -> Result<(), String> {
    let app_data_dir = DatabaseManager::resolve_app_data_dir();
    let settings = db
        .with_conn(|conn| DatabaseManager::get_settings(conn))
        .map_err(|e| e.to_string())?;

    let target_model = model_id.unwrap_or(settings.ai_local_model_id);
    let runtime = get_runtime_manager();
    runtime.ensure_started(
        &app_data_dir,
        &target_model,
        &settings.ai_hardware_preference,
        settings.ai_idle_timeout_minutes,
    )?;

    Ok(())
}
