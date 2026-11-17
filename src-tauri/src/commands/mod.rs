pub mod ai;

use crate::db::DatabaseManager;
use crate::engine::holidays::list_holidays_for_years;
use crate::models::{
    AcademicSession, AppSettings, BootstrapPayload, Course, HolidayInfo, PendingPlanNotification,
    QuizAttempt, SessionSnapshot, SubmitQuizAttemptInput, TaskItem, UpsertCalendarEventInput,
    UpsertCourseInput, UpsertExamInput, UpsertSessionInput, UpsertStudyUnitInput, UpsertTaskInput,
    UpsertTaskSubItemInput, UpsertWeeklyLessonInput, WeeklyLesson,
};
use crate::plan::apply::apply_study_plan;
use crate::plan::diff::{compute_plan_diff, DbTaskState, PlanDiffSummary};
use crate::plan::model::PlanPayload;
use crate::plan::validate::validate_plan;
use crate::plan::watcher::scan_folder_for_pending_plans;
use crate::win32::{
    apply_window_backdrop_and_theme, get_system_theme_state, update_maximize_button_bounds,
    MaxButtonBounds, SystemThemeState,
};
use chrono::Datelike;
use rusqlite::{params, OptionalExtension};
use std::collections::HashMap;
use std::fs;
use std::path::Path;
use tauri::{AppHandle, Manager, State};

#[tauri::command]
pub fn health_check() -> String {
    format!("MySession v{} OK", crate::APP_VERSION)
}

#[tauri::command]
pub fn bootstrap_app(db: State<'_, DatabaseManager>) -> Result<BootstrapPayload, String> {
    db.with_conn(|conn| db.build_bootstrap_payload(conn))
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn get_app_settings(db: State<'_, DatabaseManager>) -> Result<AppSettings, String> {
    db.with_conn(|conn| DatabaseManager::get_settings(conn))
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn save_app_settings(
    app: AppHandle,
    db: State<'_, DatabaseManager>,
    settings: AppSettings,
) -> Result<AppSettings, String> {
    let saved = db
        .with_conn(|conn| DatabaseManager::save_settings(conn, &settings))
        .map_err(|e| e.to_string())?;

    if let Some(window) = app.get_webview_window("main") {
        let _ = apply_window_backdrop_and_theme(&window, &saved.backdrop_material, &saved.theme);
    }
    Ok(saved)
}

#[tauri::command]
pub fn apply_window_material(
    app: AppHandle,
    material: String,
    theme: String,
) -> Result<SystemThemeState, String> {
    if let Some(window) = app.get_webview_window("main") {
        apply_window_backdrop_and_theme(&window, &material, &theme)
    } else {
        Ok(get_system_theme_state(&theme, &material))
    }
}

#[tauri::command]
pub fn get_system_theme_info(db: State<'_, DatabaseManager>) -> Result<SystemThemeState, String> {
    let s = db
        .with_conn(|conn| DatabaseManager::get_settings(conn))
        .unwrap_or_default();
    Ok(get_system_theme_state(&s.theme, &s.backdrop_material))
}

#[tauri::command]
pub fn set_maximize_button_bounds(bounds: MaxButtonBounds) -> Result<(), String> {
    update_maximize_button_bounds(bounds);
    Ok(())
}

// =========================================================================
// Sessions & Snapshots
// =========================================================================

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
) -> Result<AcademicSession, String> {
    db.with_conn(|conn| DatabaseManager::upsert_session(conn, &input))
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn set_active_academic_session(
    db: State<'_, DatabaseManager>,
    session_id: Option<String>,
) -> Result<SessionSnapshot, String> {
    db.with_conn(|conn| {
        let settings = DatabaseManager::set_active_session(conn, session_id.as_deref())?;
        let target_id = session_id
            .or(settings.active_session_id.clone())
            .ok_or_else(|| crate::db::DbError::Logic("Nessuna sessione attiva".into()))?;
        DatabaseManager::build_session_snapshot(conn, &target_id, &settings)
    })
    .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn delete_academic_session(
    db: State<'_, DatabaseManager>,
    session_id: String,
) -> Result<BootstrapPayload, String> {
    db.with_conn(|conn| {
        DatabaseManager::delete_session(conn, &session_id)?;
        db.build_bootstrap_payload(conn)
    })
    .map_err(|e| e.to_string())
}

// =========================================================================
// Courses & Study Units
// =========================================================================

#[tauri::command]
pub fn upsert_course(
    db: State<'_, DatabaseManager>,
    input: UpsertCourseInput,
) -> Result<Course, String> {
    db.with_conn(|conn| DatabaseManager::upsert_course(conn, &input))
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn delete_course(db: State<'_, DatabaseManager>, course_id: String) -> Result<(), String> {
    db.with_conn(|conn| DatabaseManager::delete_course(conn, &course_id))
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn upsert_study_unit(
    db: State<'_, DatabaseManager>,
    input: UpsertStudyUnitInput,
) -> Result<(), String> {
    db.with_conn(|conn| DatabaseManager::upsert_study_unit(conn, &input))
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn toggle_study_unit(
    db: State<'_, DatabaseManager>,
    unit_id: String,
    completed: bool,
) -> Result<(), String> {
    db.with_conn(|conn| DatabaseManager::toggle_study_unit(conn, &unit_id, completed))
        .map_err(|e| e.to_string())
}

// =========================================================================
// Exams
// =========================================================================

#[tauri::command]
pub fn upsert_exam(
    db: State<'_, DatabaseManager>,
    input: UpsertExamInput,
) -> Result<(), String> {
    db.with_conn(|conn| DatabaseManager::upsert_exam(conn, &input))
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn delete_exam(db: State<'_, DatabaseManager>, exam_id: String) -> Result<(), String> {
    db.with_conn(|conn| DatabaseManager::delete_exam(conn, &exam_id))
        .map_err(|e| e.to_string())
}

// =========================================================================
// Operational Blocks, Tasks & Subitems
// =========================================================================

#[tauri::command]
pub fn toggle_operational_block(
    db: State<'_, DatabaseManager>,
    block_id: String,
    done: bool,
) -> Result<(), String> {
    db.with_conn(|conn| DatabaseManager::toggle_operational_block(conn, &block_id, done))
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn upsert_task(
    db: State<'_, DatabaseManager>,
    input: UpsertTaskInput,
) -> Result<TaskItem, String> {
    db.with_conn(|conn| DatabaseManager::upsert_task(conn, &input))
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn toggle_task(
    db: State<'_, DatabaseManager>,
    task_id: String,
    done: bool,
) -> Result<(), String> {
    db.with_conn(|conn| DatabaseManager::toggle_task(conn, &task_id, done))
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn delete_task(db: State<'_, DatabaseManager>, task_id: String) -> Result<(), String> {
    db.with_conn(|conn| DatabaseManager::delete_task(conn, &task_id))
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn upsert_task_subitem(
    db: State<'_, DatabaseManager>,
    input: UpsertTaskSubItemInput,
) -> Result<(), String> {
    db.with_conn(|conn| DatabaseManager::upsert_task_subitem(conn, &input))
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn toggle_task_subitem(
    db: State<'_, DatabaseManager>,
    subitem_id: String,
    done: bool,
) -> Result<(), String> {
    db.with_conn(|conn| DatabaseManager::toggle_task_subitem(conn, &subitem_id, done))
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn delete_task_subitem(
    db: State<'_, DatabaseManager>,
    subitem_id: String,
) -> Result<(), String> {
    db.with_conn(|conn| DatabaseManager::delete_task_subitem(conn, &subitem_id))
        .map_err(|e| e.to_string())
}

// =========================================================================
// Quizzes
// =========================================================================

#[tauri::command]
pub fn submit_quiz_attempt(
    db: State<'_, DatabaseManager>,
    input: SubmitQuizAttemptInput,
) -> Result<QuizAttempt, String> {
    db.with_conn(|conn| DatabaseManager::submit_quiz_attempt(conn, &input))
        .map_err(|e| e.to_string())
}

// =========================================================================
// Weekly Lessons, Calendar Events, Daily Notes, Holidays
// =========================================================================

#[tauri::command]
pub fn upsert_weekly_lesson(
    db: State<'_, DatabaseManager>,
    input: UpsertWeeklyLessonInput,
) -> Result<WeeklyLesson, String> {
    db.with_conn(|conn| DatabaseManager::upsert_weekly_lesson(conn, &input))
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn delete_weekly_lesson(
    db: State<'_, DatabaseManager>,
    lesson_id: String,
) -> Result<(), String> {
    db.with_conn(|conn| DatabaseManager::delete_weekly_lesson(conn, &lesson_id))
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn upsert_calendar_event(
    db: State<'_, DatabaseManager>,
    input: UpsertCalendarEventInput,
) -> Result<(), String> {
    db.with_conn(|conn| DatabaseManager::upsert_calendar_event(conn, &input))
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn delete_calendar_event(
    db: State<'_, DatabaseManager>,
    event_id: String,
) -> Result<(), String> {
    db.with_conn(|conn| DatabaseManager::delete_calendar_event(conn, &event_id))
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn save_daily_note(
    db: State<'_, DatabaseManager>,
    session_id: String,
    date: String,
    content_markdown: String,
) -> Result<(), String> {
    db.with_conn(|conn| DatabaseManager::save_daily_note(conn, &session_id, &date, &content_markdown))
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn get_holidays_range(start_year: i32, end_year: i32) -> Vec<HolidayInfo> {
    list_holidays_for_years(start_year, end_year)
}

#[tauri::command]
pub fn project_lessons_to_calendar(
    db: State<'_, DatabaseManager>,
    session_id: String,
    start_date: String,
    end_date: String,
) -> Result<usize, String> {
    let s_dt = chrono::NaiveDate::parse_from_str(start_date.trim(), "%Y-%m-%d")
        .map_err(|_| "Data inizio non valida".to_string())?;
    let e_dt = chrono::NaiveDate::parse_from_str(end_date.trim(), "%Y-%m-%d")
        .map_err(|_| "Data fine non valida".to_string())?;

    db.with_conn(|conn| {
        let lessons = DatabaseManager::list_weekly_lessons(conn, &session_id)?;
        conn.execute(
            "DELETE FROM calendar_events WHERE session_id = ?1 AND category = 'lezione'",
            params![&session_id],
        )?;
        let mut count = 0usize;
        let mut cursor = s_dt;
        while cursor <= e_dt {
            let iso_wd = cursor.weekday().number_from_monday() as i32;
            let date_str = cursor.format("%Y-%m-%d").to_string();
            for l in &lessons {
                if l.day_of_week == iso_wd {
                    let details = format!(
                        "{}{}",
                        if l.classroom.is_empty() { String::new() } else { format!("Aula {} ", l.classroom) },
                        if l.professor.is_empty() { String::new() } else { format!("· {}", l.professor) }
                    );
                    DatabaseManager::upsert_calendar_event(
                        conn,
                        &UpsertCalendarEventInput {
                            id: None,
                            session_id: session_id.clone(),
                            course_id: l.course_id.clone(),
                            date: date_str.clone(),
                            time_slot: Some(format!("{}-{}", l.start_time, l.end_time)),
                            title: l.course_name.clone(),
                            category: "lezione".to_string(),
                            details: Some(details.trim().to_string()),
                        },
                    )?;
                    count += 1;
                }
            }
            cursor += chrono::Duration::days(1);
        }
        Ok(count)
    })
    .map_err(|e| e.to_string())
}

// =========================================================================
// Study Plans Import / Sync (v3)
// =========================================================================

#[tauri::command]
pub fn set_plan_folder(
    db: State<'_, DatabaseManager>,
    folder_path: String,
) -> Result<Vec<PendingPlanNotification>, String> {
    db.with_conn(|conn| {
        conn.execute(
            "UPDATE app_settings SET plan_folder = ?1 WHERE id = 1",
            params![folder_path.trim()],
        )?;
        Ok(scan_folder_for_pending_plans(conn, folder_path.trim()))
    })
    .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn scan_pending_plans(db: State<'_, DatabaseManager>) -> Result<Vec<PendingPlanNotification>, String> {
    db.with_conn(|conn| {
        let folder: String = conn
            .query_row("SELECT plan_folder FROM app_settings WHERE id = 1", [], |r| r.get(0))
            .unwrap_or_default();
        if folder.trim().is_empty() {
            return Ok(Vec::new());
        }
        Ok(scan_folder_for_pending_plans(conn, folder.trim()))
    })
    .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn preview_plan_update(
    db: State<'_, DatabaseManager>,
    file_path: String,
) -> Result<PlanDiffSummary, String> {
    let raw = fs::read_to_string(&file_path)
        .map_err(|e| format!("Impossibile leggere il piano '{}': {}", file_path, e))?;
    let theirs: PlanPayload =
        serde_json::from_str(&raw).map_err(|e| format!("JSON piano v3 non valido: {}", e))?;
    validate_plan(&theirs).map_err(|errs| errs.join("\n"))?;

    db.with_conn(|conn| {
        let base_json_opt: Option<(String, String)> = conn
            .query_row(
                "SELECT base_json, session_id FROM plan_state WHERE plan_id = ?1",
                params![&theirs.plan_id],
                |row| Ok((row.get(0)?, row.get(1)?)),
            )
            .optional()?;

        match base_json_opt {
            Some((base_json, session_id)) => {
                let base_plan: Option<PlanPayload> = serde_json::from_str(&base_json).ok();
                let mut stmt = conn.prepare(
                    "SELECT plan_task_id, date, title, done FROM tasks WHERE session_id = ?1 AND plan_task_id IS NOT NULL",
                )?;
                let rows = stmt.query_map(params![&session_id], |r| {
                    Ok(DbTaskState {
                        plan_task_id: r.get(0)?,
                        date: r.get(1)?,
                        title: r.get(2)?,
                        done: r.get::<_, i32>(3)? != 0,
                    })
                })?;
                let mut mine = Vec::new();
                for r in rows {
                    mine.push(r?);
                }
                Ok(compute_plan_diff(base_plan.as_ref(), &theirs, &mine))
            }
            None => Ok(compute_plan_diff(None, &theirs, &[])),
        }
    })
    .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn apply_plan_update(
    db: State<'_, DatabaseManager>,
    file_path: String,
    conflict_resolutions: HashMap<String, String>,
    target_session_id: Option<String>,
) -> Result<SessionSnapshot, String> {
    let raw = fs::read_to_string(&file_path)
        .map_err(|e| format!("Impossibile leggere il piano '{}': {}", file_path, e))?;
    let theirs: PlanPayload =
        serde_json::from_str(&raw).map_err(|e| format!("JSON piano v3 non valido: {}", e))?;
    validate_plan(&theirs).map_err(|errs| errs.join("\n"))?;

    db.with_conn(|conn| {
        let res = apply_study_plan(
            conn,
            &theirs,
            &raw,
            &file_path,
            &conflict_resolutions,
            target_session_id.as_deref(),
        )?;
        let settings = DatabaseManager::get_settings(conn)?;
        DatabaseManager::build_session_snapshot(conn, &res.session_id, &settings)
    })
    .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn open_folder_in_explorer(path: String) -> Result<(), String> {
    let p = Path::new(path.trim());
    let target = if p.is_file() {
        p.parent().unwrap_or(p)
    } else {
        p
    };
    #[cfg(windows)]
    {
        std::process::Command::new("explorer")
            .arg(target)
            .spawn()
            .map_err(|e| e.to_string())?;
    }
    Ok(())
}

#[tauri::command]
pub fn relaunch_app(app: AppHandle) -> Result<(), String> {
    app.restart();
}
