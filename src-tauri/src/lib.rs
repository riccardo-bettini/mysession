pub mod ai;
pub mod commands;
pub mod db;
pub mod engine;
pub mod models;
pub mod plan;
pub mod win32;

use db::DatabaseManager;
use tauri::Manager;
use win32::{apply_window_backdrop_and_theme, install_window_subclass};

pub const APP_VERSION: &str = "1.0.30";
pub const APP_AUTHOR: &str = "Riccardo Bettini";

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .setup(|app| {
            let app_data_dir = app
                .path()
                .app_data_dir()
                .expect("Impossibile risolvere la directory %APPDATA% del sistema operativo");

            let db_manager = DatabaseManager::init_in_dir(&app_data_dir)
                .expect("Impossibile inizializzare il database SQLite in %APPDATA%");

            let initial_settings = db_manager
                .with_conn(|conn| DatabaseManager::get_settings(conn))
                .unwrap_or_default();

            let _watcher = plan::start_plan_watcher(app.handle().clone(), db_manager.db_path.clone());
            app.manage(_watcher);
            app.manage(db_manager);

            if let Some(main_window) = app.get_webview_window("main") {
                let _ = install_window_subclass(&main_window, app.handle().clone());
                let _ = apply_window_backdrop_and_theme(
                    &main_window,
                    &initial_settings.backdrop_material,
                    &initial_settings.theme,
                );
            }

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::health_check,
            commands::bootstrap_app,
            commands::upsert_user_profile,
            commands::get_app_settings,
            commands::save_app_settings,
            commands::apply_window_material,
            commands::get_system_theme_info,
            commands::set_maximize_button_bounds,
            commands::get_session_snapshot,
            commands::upsert_academic_session,
            commands::set_active_academic_session,
            commands::delete_academic_session,
            commands::upsert_course,
            commands::delete_course,
            commands::toggle_study_unit,
            commands::assign_study_unit_date,
            commands::upsert_exam,
            commands::create_itinere_pair,
            commands::delete_exam,
            commands::upsert_generic_event,
            commands::delete_generic_event,
            commands::upsert_todo,
            commands::toggle_todo,
            commands::delete_todo,
            commands::save_daily_note,
            commands::get_holidays_range,
            commands::export_backup_json,
            commands::import_backup_json,
            commands::execute_ai_command,
            commands::set_plan_folder,
            commands::scan_pending_plans,
            commands::preview_plan_update,
            commands::apply_plan_update,
            commands::set_course_visibility,
        ])
        .run(tauri::generate_context!())
        .expect("Errore critico durante l'avvio di MySession (Tauri v2)");
}
