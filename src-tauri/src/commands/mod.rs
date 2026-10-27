use crate::ai::{process_natural_command, AiAssistantResponse};
use crate::db::DatabaseManager;
use crate::engine::holidays::list_holidays_for_years;
use crate::models::{
    AppSettings, BootstrapPayload, CreateItinerePairInput, FullWorkspaceBackup, HolidayInfo,
    SessionSnapshot, UpsertCourseInput, UpsertExamInput, UpsertGenericEventInput,
    UpsertProfileInput, UpsertSessionInput, UpsertTodoInput, UserProfile,
};
use crate::win32::{
    apply_window_backdrop_and_theme, get_system_theme_state, update_maximize_button_bounds,
    MaxButtonBounds, SystemThemeState,
};
use crate::{APP_AUTHOR, APP_VERSION};
use tauri::{State, WebviewWindow};

#[tauri::command]
pub fn health_check() -> String {
    format!(
        "Core Rust Tauri v2 + SQLite WAL operativo · MySession v{} · Autore: {}",
        APP_VERSION, APP_AUTHOR
    )
}

#[tauri::command]
pub fn bootstrap_app(db: State<'_, DatabaseManager>) -> Result<BootstrapPayload, String> {
    db.build_bootstrap_payload(APP_VERSION)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn upsert_user_profile(
    db: State<'_, DatabaseManager>,
    input: UpsertProfileInput,
) -> Result<UserProfile, String> {
    db.with_conn(|conn| DatabaseManager::upsert_profile(conn, &input))
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn get_app_settings(db: State<'_, DatabaseManager>) -> Result<AppSettings, String> {
    db.with_conn(|conn| DatabaseManager::get_settings(conn))
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn save_app_settings(
    window: WebviewWindow,
    db: State<'_, DatabaseManager>,
    settings: AppSettings,
) -> Result<AppSettings, String> {
    let saved = db
        .with_conn(|conn| DatabaseManager::save_settings(conn, &settings))
        .map_err(|e| e.to_string())?;
    let _ = apply_window_backdrop_and_theme(&window, &saved.backdrop_material, &saved.theme);
    Ok(saved)
}

#[tauri::command]
pub fn apply_window_material(
    window: WebviewWindow,
    backdrop_material: String,
    theme: String,
) -> Result<SystemThemeState, String> {
    apply_window_backdrop_and_theme(&window, &backdrop_material, &theme)
}

#[tauri::command]
pub fn get_system_theme_info(db: State<'_, DatabaseManager>) -> Result<SystemThemeState, String> {
    let settings = db
        .with_conn(|conn| DatabaseManager::get_settings(conn))
        .map_err(|e| e.to_string())?;
    Ok(get_system_theme_state(
        &settings.theme,
        &settings.backdrop_material,
    ))
}

#[tauri::command]
pub fn set_maximize_button_bounds(bounds: MaxButtonBounds) -> Result<(), String> {
    update_maximize_button_bounds(bounds);
    Ok(())
}

#[tauri::command]
pub fn get_session_snapshot(
    db: State<'_, DatabaseManager>,
    session_id: String,
) -> Result<SessionSnapshot, String> {
    db.with_conn(|conn| {
        let settings = DatabaseManager::get_settings(conn)?;
        DatabaseManager::build_session_snapshot(conn, &session_id, &settings)
    })
    .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn upsert_academic_session(
    db: State<'_, DatabaseManager>,
    input: UpsertSessionInput,
) -> Result<BootstrapPayload, String> {
    db.with_conn(|conn| {
        let _ = DatabaseManager::upsert_session(conn, &input)?;
        Ok(())
    })
    .map_err(|e| e.to_string())?;
    db.build_bootstrap_payload(APP_VERSION)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn set_active_academic_session(
    db: State<'_, DatabaseManager>,
    session_id: String,
) -> Result<BootstrapPayload, String> {
    db.with_conn(|conn| DatabaseManager::set_active_session(conn, &session_id))
        .map_err(|e| e.to_string())?;
    db.build_bootstrap_payload(APP_VERSION)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn delete_academic_session(
    db: State<'_, DatabaseManager>,
    session_id: String,
) -> Result<BootstrapPayload, String> {
    db.with_conn(|conn| DatabaseManager::delete_session(conn, &session_id))
        .map_err(|e| e.to_string())?;
    db.build_bootstrap_payload(APP_VERSION)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn upsert_course(
    db: State<'_, DatabaseManager>,
    input: UpsertCourseInput,
) -> Result<SessionSnapshot, String> {
    db.with_conn(|conn| {
        let _ = DatabaseManager::upsert_course(conn, &input)?;
        let settings = DatabaseManager::get_settings(conn)?;
        DatabaseManager::build_session_snapshot(conn, &input.session_id, &settings)
    })
    .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn delete_course(
    db: State<'_, DatabaseManager>,
    session_id: String,
    course_id: String,
) -> Result<SessionSnapshot, String> {
    db.with_conn(|conn| {
        DatabaseManager::delete_course(conn, &course_id)?;
        let settings = DatabaseManager::get_settings(conn)?;
        DatabaseManager::build_session_snapshot(conn, &session_id, &settings)
    })
    .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn toggle_study_unit(
    db: State<'_, DatabaseManager>,
    session_id: String,
    unit_id: String,
    is_completed: bool,
) -> Result<SessionSnapshot, String> {
    db.with_conn(|conn| {
        DatabaseManager::toggle_study_unit(conn, &unit_id, is_completed)?;
        let settings = DatabaseManager::get_settings(conn)?;
        DatabaseManager::build_session_snapshot(conn, &session_id, &settings)
    })
    .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn assign_study_unit_date(
    db: State<'_, DatabaseManager>,
    session_id: String,
    unit_id: String,
    assigned_date: Option<String>,
) -> Result<SessionSnapshot, String> {
    db.with_conn(|conn| {
        DatabaseManager::assign_study_unit_date(conn, &unit_id, assigned_date)?;
        let settings = DatabaseManager::get_settings(conn)?;
        DatabaseManager::build_session_snapshot(conn, &session_id, &settings)
    })
    .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn upsert_exam(
    db: State<'_, DatabaseManager>,
    input: UpsertExamInput,
) -> Result<SessionSnapshot, String> {
    db.with_conn(|conn| {
        let _ = DatabaseManager::upsert_exam(conn, &input)?;
        let settings = DatabaseManager::get_settings(conn)?;
        DatabaseManager::build_session_snapshot(conn, &input.session_id, &settings)
    })
    .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn create_itinere_pair(
    db: State<'_, DatabaseManager>,
    input: CreateItinerePairInput,
) -> Result<SessionSnapshot, String> {
    db.with_conn(|conn| {
        let _ = DatabaseManager::create_itinere_pair(conn, &input)?;
        let settings = DatabaseManager::get_settings(conn)?;
        DatabaseManager::build_session_snapshot(conn, &input.session_id, &settings)
    })
    .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn delete_exam(
    db: State<'_, DatabaseManager>,
    session_id: String,
    exam_id: String,
) -> Result<SessionSnapshot, String> {
    db.with_conn(|conn| {
        DatabaseManager::delete_exam(conn, &exam_id)?;
        let settings = DatabaseManager::get_settings(conn)?;
        DatabaseManager::build_session_snapshot(conn, &session_id, &settings)
    })
    .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn upsert_generic_event(
    db: State<'_, DatabaseManager>,
    input: UpsertGenericEventInput,
) -> Result<SessionSnapshot, String> {
    db.with_conn(|conn| {
        let _ = DatabaseManager::upsert_generic_event(conn, &input)?;
        let settings = DatabaseManager::get_settings(conn)?;
        DatabaseManager::build_session_snapshot(conn, &input.session_id, &settings)
    })
    .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn delete_generic_event(
    db: State<'_, DatabaseManager>,
    session_id: String,
    event_id: String,
) -> Result<SessionSnapshot, String> {
    db.with_conn(|conn| {
        DatabaseManager::delete_generic_event(conn, &event_id)?;
        let settings = DatabaseManager::get_settings(conn)?;
        DatabaseManager::build_session_snapshot(conn, &session_id, &settings)
    })
    .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn upsert_todo(
    db: State<'_, DatabaseManager>,
    input: UpsertTodoInput,
) -> Result<SessionSnapshot, String> {
    db.with_conn(|conn| {
        let _ = DatabaseManager::upsert_todo(conn, &input)?;
        let settings = DatabaseManager::get_settings(conn)?;
        DatabaseManager::build_session_snapshot(conn, &input.session_id, &settings)
    })
    .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn toggle_todo(
    db: State<'_, DatabaseManager>,
    session_id: String,
    todo_id: String,
) -> Result<SessionSnapshot, String> {
    db.with_conn(|conn| {
        DatabaseManager::toggle_todo(conn, &todo_id)?;
        let settings = DatabaseManager::get_settings(conn)?;
        DatabaseManager::build_session_snapshot(conn, &session_id, &settings)
    })
    .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn delete_todo(
    db: State<'_, DatabaseManager>,
    session_id: String,
    todo_id: String,
) -> Result<SessionSnapshot, String> {
    db.with_conn(|conn| {
        DatabaseManager::delete_todo(conn, &todo_id)?;
        let settings = DatabaseManager::get_settings(conn)?;
        DatabaseManager::build_session_snapshot(conn, &session_id, &settings)
    })
    .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn save_daily_note(
    db: State<'_, DatabaseManager>,
    session_id: String,
    date: String,
    content_markdown: String,
) -> Result<SessionSnapshot, String> {
    db.with_conn(|conn| {
        let _ = DatabaseManager::save_daily_note(conn, &session_id, &date, &content_markdown)?;
        let settings = DatabaseManager::get_settings(conn)?;
        DatabaseManager::build_session_snapshot(conn, &session_id, &settings)
    })
    .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn get_holidays_range(start_year: i32, end_year: i32) -> Result<Vec<HolidayInfo>, String> {
    let clean_start = start_year.clamp(1970, 2100);
    let clean_end = end_year.clamp(clean_start, 2100);
    Ok(list_holidays_for_years(clean_start, clean_end))
}

#[tauri::command]
pub fn export_backup_json(db: State<'_, DatabaseManager>) -> Result<String, String> {
    db.with_conn(|conn| {
        let backup = DatabaseManager::export_workspace_backup(conn, APP_VERSION)?;
        let json = serde_json::to_string_pretty(&backup)?;
        Ok(json)
    })
    .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn import_backup_json(
    db: State<'_, DatabaseManager>,
    json_payload: String,
) -> Result<BootstrapPayload, String> {
    let backup: FullWorkspaceBackup = serde_json::from_str(&json_payload)
        .map_err(|e| format!("File JSON di backup non valido: {}", e))?;
    db.with_conn(|conn| DatabaseManager::import_workspace_backup(conn, &backup))
        .map_err(|e| e.to_string())?;
    db.build_bootstrap_payload(APP_VERSION)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn execute_ai_command(
    window: WebviewWindow,
    db: State<'_, DatabaseManager>,
    session_id: String,
    prompt: String,
) -> Result<AiAssistantResponse, String> {
    let response = db
        .with_conn(|conn| {
            process_natural_command(conn, &session_id, &prompt)
                .map_err(crate::db::DbError::NotFound)
        })
        .map_err(|e| e.to_string())?;

    if let Some(ref updated_settings) = response.updated_settings {
        let _ = apply_window_backdrop_and_theme(
            &window,
            &updated_settings.backdrop_material,
            &updated_settings.theme,
        );
    }

    Ok(response)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::StudyUnitDraft;

    #[test]
    fn test_sqlite_migrations_and_full_crud_flow() {
        let db = DatabaseManager::init_in_memory().expect("In-memory SQLite init failed");

        // 1. Verify initial empty multi-user onboarding state (zero hardcoded courses!)
        let initial = db.build_bootstrap_payload(APP_VERSION).unwrap();
        assert!(!initial.profile.onboarding_completed);
        assert!(initial.sessions.is_empty());
        assert!(initial.active_snapshot.is_none());

        // 2. Complete user profile
        db.with_conn(|conn| {
            let prof = DatabaseManager::upsert_profile(
                conn,
                &UpsertProfileInput {
                    student_name: "Studente Test".to_string(),
                    university: "Politecnico".to_string(),
                    faculty: "Ingegneria".to_string(),
                    degree_course: "Meccanica".to_string(),
                    enrollment_year: Some(2026),
                    onboarding_completed: true,
                },
            )?;
            assert!(prof.onboarding_completed);
            Ok(())
        })
        .unwrap();

        // 3. Create academic session
        let session = db
            .with_conn(|conn| {
                DatabaseManager::upsert_session(
                    conn,
                    &UpsertSessionInput {
                        id: None,
                        name: "Sessione Invernale".to_string(),
                        subtitle: "Ingegneria".to_string(),
                        session_type: "invernale".to_string(),
                        academic_year: "2026/2027".to_string(),
                        start_date: "2027-01-01".to_string(),
                        end_date: "2027-02-28".to_string(),
                        notes: None,
                    },
                )
            })
            .unwrap();

        // 4. Create course with study units
        let course = db
            .with_conn(|conn| {
                DatabaseManager::upsert_course(
                    conn,
                    &UpsertCourseInput {
                        id: None,
                        session_id: session.id.clone(),
                        sigla: "MAM".to_string(),
                        name: "Meccanica Applicata".to_string(),
                        code: "054828".to_string(),
                        cfu: 10,
                        professor: "Docente".to_string(),
                        color_hex: "#0284C7".to_string(),
                        difficulty_weight: Some(1.2),
                        target_hours_override: None,
                        notes: None,
                        units: Some(vec![
                            StudyUnitDraft {
                                id: None,
                                unit_number: 1,
                                title: "Cinematica".to_string(),
                                description: None,
                                estimated_hours: Some(12.0),
                                is_completed: Some(true),
                                assigned_date: Some("2027-01-02".to_string()),
                            },
                            StudyUnitDraft {
                                id: None,
                                unit_number: 2,
                                title: "Dinamica".to_string(),
                                description: None,
                                estimated_hours: Some(15.0),
                                is_completed: Some(false),
                                assigned_date: None,
                            },
                        ]),
                    },
                )
            })
            .unwrap();

        // 5. Create in-itinere pair
        let pair = db
            .with_conn(|conn| {
                DatabaseManager::create_itinere_pair(
                    conn,
                    &CreateItinerePairInput {
                        session_id: session.id.clone(),
                        course_id: course.id.clone(),
                        date1: "2026-11-15".to_string(),
                        date2: "2027-01-15".to_string(),
                        time_slot1: Some("09:00".to_string()),
                        time_slot2: Some("09:00".to_string()),
                        classroom: Some("Aula B.2.1".to_string()),
                        details: None,
                    },
                )
            })
            .unwrap();
        assert_eq!(pair.len(), 2);
        assert_eq!(pair[0].pair_id, pair[1].pair_id);

        // 6. Verify snapshot & backup export/import roundtrip
        let payload = db.build_bootstrap_payload(APP_VERSION).unwrap();
        let snap = payload.active_snapshot.unwrap();
        assert_eq!(snap.courses.len(), 1);
        assert_eq!(snap.study_units.len(), 2);
        assert_eq!(snap.exams.len(), 2);
        assert_eq!(snap.telemetry.total_cfu, 10);
        assert_eq!(snap.telemetry.completed_units_total, 1);

        let exported_json = db
            .with_conn(|conn| {
                let b = DatabaseManager::export_workspace_backup(conn, APP_VERSION)?;
                Ok(serde_json::to_string(&b)?)
            })
            .unwrap();

        let restored: FullWorkspaceBackup = serde_json::from_str(&exported_json).unwrap();
        db.with_conn(|conn| DatabaseManager::import_workspace_backup(conn, &restored))
            .unwrap();

        let after_import = db.build_bootstrap_payload(APP_VERSION).unwrap();
        assert_eq!(after_import.sessions.len(), 1);
    }
}
