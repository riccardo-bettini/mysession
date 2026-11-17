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

pub const APP_VERSION: &str = "2.00";
pub const APP_AUTHOR: &str = "Riccardo Bettini";

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_updater::Builder::new().build())
        .setup(|app| {
            let app_data_dir = app
                .path()
                .app_data_dir()
                .expect("Impossibile risolvere la directory %APPDATA% del sistema operativo");

            let db_manager = DatabaseManager::init_in_dir(&app_data_dir)
                .expect("Impossibile inizializzare il database SQLite v2.00 in %APPDATA%");

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
            commands::upsert_study_unit,
            commands::toggle_study_unit,
            commands::upsert_exam,
            commands::delete_exam,
            commands::toggle_operational_block,
            commands::upsert_task,
            commands::toggle_task,
            commands::delete_task,
            commands::upsert_task_subitem,
            commands::toggle_task_subitem,
            commands::delete_task_subitem,
            commands::submit_quiz_attempt,
            commands::upsert_weekly_lesson,
            commands::delete_weekly_lesson,
            commands::upsert_calendar_event,
            commands::delete_calendar_event,
            commands::save_daily_note,
            commands::get_holidays_range,
            commands::project_lessons_to_calendar,
            commands::set_plan_folder,
            commands::scan_pending_plans,
            commands::preview_plan_update,
            commands::apply_plan_update,
            commands::open_folder_in_explorer,
            commands::relaunch_app,
            commands::ai::get_system_hardware_info,
            commands::ai::get_cli_agents_status,
            commands::ai::get_ai_models_status,
            commands::ai::start_ai_model_download,
            commands::ai::cancel_ai_model_download,
            commands::ai::delete_ai_model,
            commands::ai::get_ai_runtime_status,
            commands::ai::stop_ai_runtime,
            commands::ai::start_ai_runtime,
            commands::ai::execute_ai_command,
            commands::ai::undo_ai_action,
            commands::ai::parse_timetable_image,
            commands::ai::parse_timetable_prompt,
        ])
        .run(tauri::generate_context!())
        .expect("Errore critico durante l'avvio di MySession v2.00");
}
