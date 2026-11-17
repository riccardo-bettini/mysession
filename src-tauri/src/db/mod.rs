pub mod migrations;

use crate::engine::{conflicts::detect_session_conflicts, workload::compute_session_telemetry};
use crate::models::{
    AcademicSession, AppSettings, BootstrapPayload, CalendarEvent, Course, DailyNote,
    DailyOperationalPlan, DailyQuiz, Exam, OperationalBlock, QuizAttempt, QuizQuestion,
    SessionSnapshot, StudyUnit, SubmitQuizAttemptInput, TaskItem, TaskSubItem, UpsertCalendarEventInput,
    UpsertCourseInput, UpsertExamInput, UpsertSessionInput, UpsertStudyUnitInput, UpsertTaskInput,
    UpsertTaskSubItemInput, UpsertWeeklyLessonInput, WeeklyLesson, DEFAULT_SIDEBAR_SECTIONS,
};
use crate::plan::watcher::scan_folder_for_pending_plans;
use chrono::Utc;
use rusqlite::{params, Connection, OptionalExtension};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use thiserror::Error;
use uuid::Uuid;

#[derive(Debug, Error)]
pub enum DbError {
    #[error("Errore SQLite: {0}")]
    Sqlite(#[from] rusqlite::Error),
    #[error("Errore I/O: {0}")]
    Io(#[from] std::io::Error),
    #[error("Errore JSON: {0}")]
    Serde(#[from] serde_json::Error),
    #[error("{0}")]
    Logic(String),
}

pub type DbResult<T> = Result<T, DbError>;

#[derive(Clone)]
pub struct DatabaseManager {
    conn: Arc<Mutex<Connection>>,
    pub db_path: PathBuf,
    pub app_data_dir: PathBuf,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct SessionUndoPayload {
    tasks: Vec<TaskItem>,
    exams: Vec<Exam>,
    courses: Vec<Course>,
    weekly_lessons: Vec<WeeklyLesson>,
}

impl DatabaseManager {
    pub fn resolve_app_data_dir() -> PathBuf {
        if let Ok(appdata) = std::env::var("APPDATA") {
            let dir = PathBuf::from(appdata).join("com.mysession.app");
            let _ = fs::create_dir_all(&dir);
            return dir;
        }
        let fallback = std::env::temp_dir().join("com.mysession.app");
        let _ = fs::create_dir_all(&fallback);
        fallback
    }

    pub fn init_in_dir(app_data_dir: &Path) -> DbResult<Self> {
        fs::create_dir_all(app_data_dir)?;
        let db_path = app_data_dir.join("database.sqlite");

        // Se esiste un DB v1.x, fa backup in database_v1.bak e resetta per la v2.00 pulita
        migrations::backup_and_reset_legacy_v1_if_needed(&db_path);

        let conn = Connection::open(&db_path)?;
        conn.execute_batch(
            r#"
            PRAGMA journal_mode = WAL;
            PRAGMA synchronous = NORMAL;
            PRAGMA foreign_keys = ON;
            PRAGMA busy_timeout = 5000;
            "#,
        )?;
        migrations::run_migrations(&conn)?;

        Ok(Self {
            conn: Arc::new(Mutex::new(conn)),
            db_path,
            app_data_dir: app_data_dir.to_path_buf(),
        })
    }

    pub fn init_in_memory() -> DbResult<Self> {
        let conn = Connection::open_in_memory()?;
        conn.execute_batch("PRAGMA foreign_keys = ON;")?;
        migrations::run_migrations(&conn)?;
        Ok(Self {
            conn: Arc::new(Mutex::new(conn)),
            db_path: PathBuf::from(":memory:"),
            app_data_dir: std::env::temp_dir().join("com.mysession.app.test"),
        })
    }

    pub fn with_conn<F, R>(&self, f: F) -> DbResult<R>
    where
        F: FnOnce(&mut Connection) -> DbResult<R>,
    {
        let mut guard = self
            .conn
            .lock()
            .map_err(|_| DbError::Logic("Mutex SQLite bloccato".into()))?;
        f(&mut guard)
    }

    // =========================================================================
    // Settings
    // =========================================================================

    pub fn get_settings(conn: &Connection) -> DbResult<AppSettings> {
        conn.query_row(
            r#"
            SELECT theme, backdrop_material, ui_scale, accent_color,
                   cat_teoria_color, cat_esercizi_color, cat_temi_esame_color,
                   cat_dimostrazioni_color, cat_domande_orale_color,
                   sidebar_sections_json, hidden_sections_json,
                   plan_folder, active_session_id, study_hours_per_cfu, start_day_of_week,
                   ai_provider, ai_local_model_id, ai_custom_cli_command, ai_idle_timeout_minutes
            FROM app_settings WHERE id = 1
            "#,
            [],
            |row| {
                let sections_raw: String = row.get(9)?;
                let hidden_raw: String = row.get(10)?;
                let mut sidebar_sections: Vec<String> = serde_json::from_str(&sections_raw)
                    .unwrap_or_else(|_| DEFAULT_SIDEBAR_SECTIONS.iter().map(|s| s.to_string()).collect());
                // Ensure all 7 sections exist in sidebar_sections
                for s in DEFAULT_SIDEBAR_SECTIONS {
                    if !sidebar_sections.iter().any(|x| x == s) {
                        sidebar_sections.push(s.to_string());
                    }
                }
                let hidden_sections: Vec<String> =
                    serde_json::from_str(&hidden_raw).unwrap_or_default();

                Ok(AppSettings {
                    theme: row.get(0)?,
                    backdrop_material: row.get(1)?,
                    ui_scale: row.get(2)?,
                    accent_color: row.get(3)?,
                    cat_teoria_color: row.get(4)?,
                    cat_esercizi_color: row.get(5)?,
                    cat_temi_esame_color: row.get(6)?,
                    cat_dimostrazioni_color: row.get(7)?,
                    cat_domande_orale_color: row.get(8)?,
                    sidebar_sections,
                    hidden_sections,
                    plan_folder: row.get(11)?,
                    active_session_id: row.get(12)?,
                    study_hours_per_cfu: row.get(13)?,
                    start_day_of_week: row.get(14)?,
                    ai_provider: row.get(15)?,
                    ai_local_model_id: row.get(16)?,
                    ai_custom_cli_command: row.get(17)?,
                    ai_idle_timeout_minutes: row.get(18)?,
                })
            },
        )
        .map_err(DbError::from)
    }

    pub fn save_settings(conn: &Connection, s: &AppSettings) -> DbResult<AppSettings> {
        let sections_json = serde_json::to_string(&s.sidebar_sections)?;
        let hidden_json = serde_json::to_string(&s.hidden_sections)?;
        conn.execute(
            r#"
            UPDATE app_settings SET
                theme = ?1,
                backdrop_material = ?2,
                ui_scale = ?3,
                accent_color = ?4,
                cat_teoria_color = ?5,
                cat_esercizi_color = ?6,
                cat_temi_esame_color = ?7,
                cat_dimostrazioni_color = ?8,
                cat_domande_orale_color = ?9,
                sidebar_sections_json = ?10,
                hidden_sections_json = ?11,
                plan_folder = ?12,
                active_session_id = ?13,
                study_hours_per_cfu = ?14,
                start_day_of_week = ?15,
                ai_provider = ?16,
                ai_local_model_id = ?17,
                ai_custom_cli_command = ?18,
                ai_idle_timeout_minutes = ?19
            WHERE id = 1
            "#,
            params![
                &s.theme,
                &s.backdrop_material,
                s.ui_scale.clamp(80, 140),
                &s.accent_color,
                &s.cat_teoria_color,
                &s.cat_esercizi_color,
                &s.cat_temi_esame_color,
                &s.cat_dimostrazioni_color,
                &s.cat_domande_orale_color,
                &sections_json,
                &hidden_json,
                &s.plan_folder,
                &s.active_session_id,
                s.study_hours_per_cfu,
                s.start_day_of_week,
                &s.ai_provider,
                &s.ai_local_model_id,
                &s.ai_custom_cli_command,
                s.ai_idle_timeout_minutes.clamp(0, 120)
            ],
        )?;
        Self::get_settings(conn)
    }

    // =========================================================================
    // Sessions
    // =========================================================================

    pub fn list_sessions(conn: &Connection) -> DbResult<Vec<AcademicSession>> {
        let mut stmt = conn.prepare(
            r#"
            SELECT id, plan_id, name, subtitle, session_type, academic_year,
                   start_date, end_date, max_hours_per_day, study_weekdays_json,
                   off_dates_json, review_days_before_exam, notes, created_at
            FROM academic_sessions
            ORDER BY start_date DESC, created_at DESC
            "#,
        )?;
        let rows = stmt.query_map([], |row| {
            let wd_raw: String = row.get(9)?;
            let off_raw: String = row.get(10)?;
            Ok(AcademicSession {
                id: row.get(0)?,
                plan_id: row.get(1)?,
                name: row.get(2)?,
                subtitle: row.get(3)?,
                session_type: row.get(4)?,
                academic_year: row.get(5)?,
                start_date: row.get(6)?,
                end_date: row.get(7)?,
                max_hours_per_day: row.get(8)?,
                study_weekdays: serde_json::from_str(&wd_raw).unwrap_or_else(|_| vec![1, 2, 3, 4, 5]),
                off_dates: serde_json::from_str(&off_raw).unwrap_or_default(),
                review_days_before_exam: row.get(11)?,
                notes: row.get(12)?,
                created_at: row.get(13)?,
            })
        })?;
        let mut out = Vec::new();
        for r in rows {
            out.push(r?);
        }
        Ok(out)
    }

    pub fn get_session(conn: &Connection, session_id: &str) -> DbResult<Option<AcademicSession>> {
        conn.query_row(
            r#"
            SELECT id, plan_id, name, subtitle, session_type, academic_year,
                   start_date, end_date, max_hours_per_day, study_weekdays_json,
                   off_dates_json, review_days_before_exam, notes, created_at
            FROM academic_sessions WHERE id = ?1
            "#,
            params![session_id],
            |row| {
                let wd_raw: String = row.get(9)?;
                let off_raw: String = row.get(10)?;
                Ok(AcademicSession {
                    id: row.get(0)?,
                    plan_id: row.get(1)?,
                    name: row.get(2)?,
                    subtitle: row.get(3)?,
                    session_type: row.get(4)?,
                    academic_year: row.get(5)?,
                    start_date: row.get(6)?,
                    end_date: row.get(7)?,
                    max_hours_per_day: row.get(8)?,
                    study_weekdays: serde_json::from_str(&wd_raw).unwrap_or_else(|_| vec![1, 2, 3, 4, 5]),
                    off_dates: serde_json::from_str(&off_raw).unwrap_or_default(),
                    review_days_before_exam: row.get(11)?,
                    notes: row.get(12)?,
                    created_at: row.get(13)?,
                })
            },
        )
        .optional()
        .map_err(DbError::from)
    }

    pub fn upsert_session(conn: &Connection, input: &UpsertSessionInput) -> DbResult<AcademicSession> {
        let now = Utc::now().to_rfc3339();
        let weekdays_json = serde_json::to_string(
            input.study_weekdays.as_deref().unwrap_or(&[1, 2, 3, 4, 5]),
        )?;
        let off_json = serde_json::to_string(input.off_dates.as_deref().unwrap_or(&[]))?;
        let max_h = input.max_hours_per_day.unwrap_or(6.0);
        let rev_days = input.review_days_before_exam.unwrap_or(5);
        let notes = input.notes.as_deref().unwrap_or("");

        let id = match input.id.as_deref().filter(|s| !s.trim().is_empty()) {
            Some(existing_id) => {
                conn.execute(
                    r#"
                    UPDATE academic_sessions
                    SET name = ?1, subtitle = ?2, session_type = ?3, academic_year = ?4,
                        start_date = ?5, end_date = ?6, max_hours_per_day = ?7,
                        study_weekdays_json = ?8, off_dates_json = ?9,
                        review_days_before_exam = ?10, notes = ?11
                    WHERE id = ?12
                    "#,
                    params![
                        input.name.trim(),
                        input.subtitle.trim(),
                        input.session_type.trim(),
                        input.academic_year.trim(),
                        input.start_date.trim(),
                        input.end_date.trim(),
                        max_h,
                        &weekdays_json,
                        &off_json,
                        rev_days,
                        notes,
                        existing_id
                    ],
                )?;
                existing_id.to_string()
            }
            None => {
                let new_id = format!("sess_{}", Uuid::new_v4().simple());
                conn.execute(
                    r#"
                    INSERT INTO academic_sessions (
                        id, plan_id, name, subtitle, session_type, academic_year,
                        start_date, end_date, max_hours_per_day, study_weekdays_json,
                        off_dates_json, review_days_before_exam, notes, created_at
                    ) VALUES (?1, NULL, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13)
                    "#,
                    params![
                        &new_id,
                        input.name.trim(),
                        input.subtitle.trim(),
                        input.session_type.trim(),
                        input.academic_year.trim(),
                        input.start_date.trim(),
                        input.end_date.trim(),
                        max_h,
                        &weekdays_json,
                        &off_json,
                        rev_days,
                        notes,
                        &now
                    ],
                )?;
                conn.execute(
                    "UPDATE app_settings SET active_session_id = ?1 WHERE id = 1",
                    params![&new_id],
                )?;
                new_id
            }
        };

        Self::get_session(conn, &id)?
            .ok_or_else(|| DbError::Logic("Sessione non trovata dopo il salvataggio".into()))
    }

    pub fn set_active_session(conn: &Connection, session_id: Option<&str>) -> DbResult<AppSettings> {
        conn.execute(
            "UPDATE app_settings SET active_session_id = ?1 WHERE id = 1",
            params![session_id],
        )?;
        Self::get_settings(conn)
    }

    pub fn delete_session(conn: &Connection, session_id: &str) -> DbResult<()> {
        conn.execute("DELETE FROM academic_sessions WHERE id = ?1", params![session_id])?;
        let remaining: Option<String> = conn
            .query_row(
                "SELECT id FROM academic_sessions ORDER BY start_date DESC LIMIT 1",
                [],
                |row| row.get(0),
            )
            .optional()?;
        conn.execute(
            "UPDATE app_settings SET active_session_id = ?1 WHERE id = 1 AND active_session_id = ?2",
            params![remaining, session_id],
        )?;
        Ok(())
    }

    // =========================================================================
    // Courses & Units
    // =========================================================================

    pub fn list_courses(conn: &Connection, session_id: &str) -> DbResult<Vec<Course>> {
        let mut stmt = conn.prepare(
            r#"
            SELECT id, session_id, plan_course_id, sigla, name, code, cfu, professor,
                   color_hex, difficulty_weight, target_hours, is_active, notes, sort_order
            FROM courses WHERE session_id = ?1 ORDER BY sort_order ASC, name ASC
            "#,
        )?;
        let rows = stmt.query_map(params![session_id], |row| {
            Ok(Course {
                id: row.get(0)?,
                session_id: row.get(1)?,
                plan_course_id: row.get(2)?,
                sigla: row.get(3)?,
                name: row.get(4)?,
                code: row.get(5)?,
                cfu: row.get(6)?,
                professor: row.get(7)?,
                color_hex: row.get(8)?,
                difficulty_weight: row.get(9)?,
                target_hours: row.get(10)?,
                is_active: row.get::<_, i32>(11)? != 0,
                notes: row.get(12)?,
                sort_order: row.get(13)?,
            })
        })?;
        let mut out = Vec::new();
        for r in rows {
            out.push(r?);
        }
        Ok(out)
    }

    pub fn upsert_course(conn: &Connection, input: &UpsertCourseInput) -> DbResult<Course> {
        let sigla = input.sigla.trim().to_uppercase();
        let id = match input.id.as_deref().filter(|s| !s.trim().is_empty()) {
            Some(eid) => {
                conn.execute(
                    r#"
                    UPDATE courses SET
                        sigla = ?1, name = ?2, code = ?3, cfu = ?4, professor = ?5,
                        color_hex = ?6, difficulty_weight = ?7, target_hours = ?8,
                        is_active = ?9, notes = ?10
                    WHERE id = ?11
                    "#,
                    params![
                        &sigla,
                        input.name.trim(),
                        input.code.as_deref().unwrap_or("").trim(),
                        input.cfu,
                        input.professor.as_deref().unwrap_or("").trim(),
                        input.color_hex.trim(),
                        input.difficulty_weight.unwrap_or(1.0),
                        input.target_hours,
                        if input.is_active.unwrap_or(true) { 1 } else { 0 },
                        input.notes.as_deref().unwrap_or("").trim(),
                        eid
                    ],
                )?;
                eid.to_string()
            }
            None => {
                let new_id = format!("crs_{}", Uuid::new_v4().simple());
                let max_order: i32 = conn
                    .query_row(
                        "SELECT COALESCE(MAX(sort_order), -1) + 1 FROM courses WHERE session_id = ?1",
                        params![&input.session_id],
                        |r| r.get(0),
                    )
                    .unwrap_or(0);
                conn.execute(
                    r#"
                    INSERT INTO courses (
                        id, session_id, plan_course_id, sigla, name, code, cfu, professor,
                        color_hex, difficulty_weight, target_hours, is_active, notes, sort_order
                    ) VALUES (?1, ?2, NULL, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13)
                    "#,
                    params![
                        &new_id,
                        &input.session_id,
                        &sigla,
                        input.name.trim(),
                        input.code.as_deref().unwrap_or("").trim(),
                        input.cfu,
                        input.professor.as_deref().unwrap_or("").trim(),
                        input.color_hex.trim(),
                        input.difficulty_weight.unwrap_or(1.0),
                        input.target_hours,
                        if input.is_active.unwrap_or(true) { 1 } else { 0 },
                        input.notes.as_deref().unwrap_or("").trim(),
                        max_order
                    ],
                )?;
                new_id
            }
        };

        Self::list_courses(conn, &input.session_id)?
            .into_iter()
            .find(|c| c.id == id)
            .ok_or_else(|| DbError::Logic("Corso non trovato".into()))
    }

    pub fn delete_course(conn: &Connection, course_id: &str) -> DbResult<()> {
        conn.execute("DELETE FROM courses WHERE id = ?1", params![course_id])?;
        Ok(())
    }

    pub fn list_study_units(conn: &Connection, session_id: &str) -> DbResult<Vec<StudyUnit>> {
        let mut stmt = conn.prepare(
            r#"
            SELECT id, course_id, session_id, plan_unit_id, unit_number, chapter, title,
                   description, source, estimated_hours, priority, is_completed, completed_at
            FROM study_units WHERE session_id = ?1 ORDER BY unit_number ASC
            "#,
        )?;
        let rows = stmt.query_map(params![session_id], |row| {
            Ok(StudyUnit {
                id: row.get(0)?,
                course_id: row.get(1)?,
                session_id: row.get(2)?,
                plan_unit_id: row.get(3)?,
                unit_number: row.get(4)?,
                chapter: row.get(5)?,
                title: row.get(6)?,
                description: row.get(7)?,
                source: row.get(8)?,
                estimated_hours: row.get(9)?,
                priority: row.get(10)?,
                is_completed: row.get::<_, i32>(11)? != 0,
                completed_at: row.get(12)?,
            })
        })?;
        let mut out = Vec::new();
        for r in rows {
            out.push(r?);
        }
        Ok(out)
    }

    pub fn upsert_study_unit(conn: &Connection, input: &UpsertStudyUnitInput) -> DbResult<()> {
        match input.id.as_deref().filter(|s| !s.trim().is_empty()) {
            Some(uid) => {
                conn.execute(
                    r#"
                    UPDATE study_units SET
                        unit_number = ?1, chapter = ?2, title = ?3, description = ?4,
                        source = ?5, estimated_hours = ?6, priority = ?7
                    WHERE id = ?8
                    "#,
                    params![
                        input.unit_number,
                        input.chapter.as_deref().unwrap_or("").trim(),
                        input.title.trim(),
                        input.description.as_deref().unwrap_or("").trim(),
                        input.source.as_deref().unwrap_or("").trim(),
                        input.estimated_hours,
                        input.priority.unwrap_or(2),
                        uid
                    ],
                )?;
            }
            None => {
                let uid = format!("unit_{}", Uuid::new_v4().simple());
                conn.execute(
                    r#"
                    INSERT INTO study_units (
                        id, course_id, session_id, plan_unit_id, unit_number, chapter,
                        title, description, source, estimated_hours, priority, is_completed, completed_at
                    ) VALUES (?1, ?2, ?3, NULL, ?4, ?5, ?6, ?7, ?8, ?9, ?10, 0, NULL)
                    "#,
                    params![
                        &uid,
                        &input.course_id,
                        &input.session_id,
                        input.unit_number,
                        input.chapter.as_deref().unwrap_or("").trim(),
                        input.title.trim(),
                        input.description.as_deref().unwrap_or("").trim(),
                        input.source.as_deref().unwrap_or("").trim(),
                        input.estimated_hours,
                        input.priority.unwrap_or(2)
                    ],
                )?;
            }
        }
        Ok(())
    }

    pub fn toggle_study_unit(conn: &Connection, unit_id: &str, completed: bool) -> DbResult<()> {
        let completed_at = if completed {
            Some(Utc::now().to_rfc3339())
        } else {
            None
        };
        conn.execute(
            "UPDATE study_units SET is_completed = ?1, completed_at = ?2 WHERE id = ?3",
            params![if completed { 1 } else { 0 }, completed_at, unit_id],
        )?;
        Ok(())
    }

    // =========================================================================
    // Exams
    // =========================================================================

    pub fn list_exams(conn: &Connection, session_id: &str) -> DbResult<Vec<Exam>> {
        let mut stmt = conn.prepare(
            r#"
            SELECT id, session_id, course_id, plan_exam_id, date, time_slot, duration_minutes,
                   round, exam_type, classroom, title, details, status, grade
            FROM exams WHERE session_id = ?1 ORDER BY date ASC, time_slot ASC
            "#,
        )?;
        let rows = stmt.query_map(params![session_id], |row| {
            Ok(Exam {
                id: row.get(0)?,
                session_id: row.get(1)?,
                course_id: row.get(2)?,
                plan_exam_id: row.get(3)?,
                date: row.get(4)?,
                time_slot: row.get(5)?,
                duration_minutes: row.get(6)?,
                round: row.get(7)?,
                exam_type: row.get(8)?,
                classroom: row.get(9)?,
                title: row.get(10)?,
                details: row.get(11)?,
                status: row.get(12)?,
                grade: row.get(13)?,
            })
        })?;
        let mut out = Vec::new();
        for r in rows {
            out.push(r?);
        }
        Ok(out)
    }

    pub fn upsert_exam(conn: &Connection, input: &UpsertExamInput) -> DbResult<()> {
        let title = input
            .title
            .as_deref()
            .filter(|s| !s.trim().is_empty())
            .map(|s| s.trim().to_string())
            .unwrap_or_else(|| format!("Appello {}", input.round));

        match input.id.as_deref().filter(|s| !s.trim().is_empty()) {
            Some(eid) => {
                conn.execute(
                    r#"
                    UPDATE exams SET
                        course_id = ?1, date = ?2, time_slot = ?3, duration_minutes = ?4,
                        round = ?5, exam_type = ?6, classroom = ?7, title = ?8,
                        details = ?9, status = ?10, grade = ?11
                    WHERE id = ?12
                    "#,
                    params![
                        &input.course_id,
                        input.date.trim(),
                        input.time_slot.as_deref().unwrap_or("").trim(),
                        input.duration_minutes.unwrap_or(120),
                        input.round.trim(),
                        input.exam_type.as_deref().unwrap_or("scritto_orale").trim(),
                        input.classroom.as_deref().unwrap_or("").trim(),
                        &title,
                        input.details.as_deref().unwrap_or("").trim(),
                        input.status.as_deref().unwrap_or("planned").trim(),
                        &input.grade,
                        eid
                    ],
                )?;
            }
            None => {
                let eid = format!("exam_{}", Uuid::new_v4().simple());
                conn.execute(
                    r#"
                    INSERT INTO exams (
                        id, session_id, course_id, plan_exam_id, date, time_slot, duration_minutes,
                        round, exam_type, classroom, title, details, status, grade
                    ) VALUES (?1, ?2, ?3, NULL, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13)
                    "#,
                    params![
                        &eid,
                        &input.session_id,
                        &input.course_id,
                        input.date.trim(),
                        input.time_slot.as_deref().unwrap_or("").trim(),
                        input.duration_minutes.unwrap_or(120),
                        input.round.trim(),
                        input.exam_type.as_deref().unwrap_or("scritto_orale").trim(),
                        input.classroom.as_deref().unwrap_or("").trim(),
                        &title,
                        input.details.as_deref().unwrap_or("").trim(),
                        input.status.as_deref().unwrap_or("planned").trim(),
                        &input.grade
                    ],
                )?;
            }
        }
        Ok(())
    }

    pub fn delete_exam(conn: &Connection, exam_id: &str) -> DbResult<()> {
        conn.execute("DELETE FROM exams WHERE id = ?1", params![exam_id])?;
        Ok(())
    }

    // =========================================================================
    // Operational Plans, Tasks & Subtasks
    // =========================================================================

    pub fn list_operational_plans(
        conn: &Connection,
        session_id: &str,
    ) -> DbResult<Vec<DailyOperationalPlan>> {
        let mut blocks_by_date: HashMap<String, Vec<OperationalBlock>> = HashMap::new();
        {
            let mut bstmt = conn.prepare(
                r#"
                SELECT id, session_id, date, course_id, block_order, category, title,
                       chapter, topic, pages, source, duration_hours, instructions, done, plan_block_id
                FROM operational_blocks WHERE session_id = ?1 ORDER BY date ASC, block_order ASC
                "#,
            )?;
            let brows = bstmt.query_map(params![session_id], |row| {
                Ok(OperationalBlock {
                    id: row.get(0)?,
                    session_id: row.get(1)?,
                    date: row.get(2)?,
                    course_id: row.get(3)?,
                    block_order: row.get(4)?,
                    category: row.get(5)?,
                    title: row.get(6)?,
                    chapter: row.get(7)?,
                    topic: row.get(8)?,
                    pages: row.get(9)?,
                    source: row.get(10)?,
                    duration_hours: row.get(11)?,
                    instructions: row.get(12)?,
                    done: row.get::<_, i32>(13)? != 0,
                    plan_block_id: row.get(14)?,
                })
            })?;
            for r in brows {
                let b = r?;
                blocks_by_date.entry(b.date.clone()).or_default().push(b);
            }
        }

        let mut stmt = conn.prepare(
            "SELECT session_id, date, daily_goal, strategy_note FROM daily_operational_plans WHERE session_id = ?1 ORDER BY date ASC",
        )?;
        let rows = stmt.query_map(params![session_id], |row| {
            let date: String = row.get(1)?;
            Ok(DailyOperationalPlan {
                session_id: row.get(0)?,
                date: date.clone(),
                daily_goal: row.get(2)?,
                strategy_note: row.get(3)?,
                blocks: Vec::new(),
            })
        })?;

        let mut out = Vec::new();
        for r in rows {
            let mut plan = r?;
            plan.blocks = blocks_by_date.remove(&plan.date).unwrap_or_default();
            out.push(plan);
        }
        Ok(out)
    }

    pub fn toggle_operational_block(conn: &Connection, block_id: &str, done: bool) -> DbResult<()> {
        conn.execute(
            "UPDATE operational_blocks SET done = ?1 WHERE id = ?2",
            params![if done { 1 } else { 0 }, block_id],
        )?;
        Ok(())
    }

    pub fn list_tasks(conn: &Connection, session_id: &str) -> DbResult<Vec<TaskItem>> {
        let mut sub_by_task: HashMap<String, Vec<TaskSubItem>> = HashMap::new();
        {
            let mut sstmt = conn.prepare(
                r#"
                SELECT s.id, s.task_id, s.text, s.category, s.pages, s.done, s.sort_order, s.plan_subtask_id
                FROM task_subitems s
                INNER JOIN tasks t ON s.task_id = t.id
                WHERE t.session_id = ?1
                ORDER BY s.sort_order ASC
                "#,
            )?;
            let srows = sstmt.query_map(params![session_id], |row| {
                Ok(TaskSubItem {
                    id: row.get(0)?,
                    task_id: row.get(1)?,
                    text: row.get(2)?,
                    category: row.get(3)?,
                    pages: row.get(4)?,
                    done: row.get::<_, i32>(5)? != 0,
                    sort_order: row.get(6)?,
                    plan_subtask_id: row.get(7)?,
                })
            })?;
            for sr in srows {
                let s = sr?;
                sub_by_task.entry(s.task_id.clone()).or_default().push(s);
            }
        }

        let mut stmt = conn.prepare(
            r#"
            SELECT id, session_id, course_id, unit_id, date, category, title,
                   chapter, topic, pages, source, planned_hours, done, priority, plan_task_id, created_at
            FROM tasks WHERE session_id = ?1 ORDER BY date ASC, priority ASC, created_at ASC
            "#,
        )?;
        let rows = stmt.query_map(params![session_id], |row| {
            Ok(TaskItem {
                id: row.get(0)?,
                session_id: row.get(1)?,
                course_id: row.get(2)?,
                unit_id: row.get(3)?,
                date: row.get(4)?,
                category: row.get(5)?,
                title: row.get(6)?,
                chapter: row.get(7)?,
                topic: row.get(8)?,
                pages: row.get(9)?,
                source: row.get(10)?,
                planned_hours: row.get(11)?,
                done: row.get::<_, i32>(12)? != 0,
                priority: row.get(13)?,
                plan_task_id: row.get(14)?,
                created_at: row.get(15)?,
                subtasks: Vec::new(),
            })
        })?;
        let mut out = Vec::new();
        for r in rows {
            let mut t = r?;
            t.subtasks = sub_by_task.remove(&t.id).unwrap_or_default();
            out.push(t);
        }
        Ok(out)
    }

    pub fn upsert_task(conn: &Connection, input: &UpsertTaskInput) -> DbResult<TaskItem> {
        let now = Utc::now().to_rfc3339();
        let tid = match input.id.as_deref().filter(|s| !s.trim().is_empty()) {
            Some(eid) => {
                conn.execute(
                    r#"
                    UPDATE tasks SET
                        course_id = ?1, unit_id = ?2, date = ?3, category = ?4, title = ?5,
                        chapter = ?6, topic = ?7, pages = ?8, source = ?9,
                        planned_hours = ?10, priority = ?11
                    WHERE id = ?12
                    "#,
                    params![
                        &input.course_id,
                        &input.unit_id,
                        input.date.trim(),
                        input.category.trim(),
                        input.title.trim(),
                        input.chapter.as_deref().unwrap_or("").trim(),
                        input.topic.as_deref().unwrap_or("").trim(),
                        input.pages.as_deref().unwrap_or("").trim(),
                        input.source.as_deref().unwrap_or("").trim(),
                        input.planned_hours.unwrap_or(1.0),
                        input.priority.unwrap_or(2),
                        eid
                    ],
                )?;
                eid.to_string()
            }
            None => {
                let new_id = format!("tsk_{}", Uuid::new_v4().simple());
                conn.execute(
                    r#"
                    INSERT INTO tasks (
                        id, session_id, course_id, unit_id, date, category, title,
                        chapter, topic, pages, source, planned_hours, done, priority, plan_task_id, created_at
                    ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, 0, ?13, NULL, ?14)
                    "#,
                    params![
                        &new_id,
                        &input.session_id,
                        &input.course_id,
                        &input.unit_id,
                        input.date.trim(),
                        input.category.trim(),
                        input.title.trim(),
                        input.chapter.as_deref().unwrap_or("").trim(),
                        input.topic.as_deref().unwrap_or("").trim(),
                        input.pages.as_deref().unwrap_or("").trim(),
                        input.source.as_deref().unwrap_or("").trim(),
                        input.planned_hours.unwrap_or(1.0),
                        input.priority.unwrap_or(2),
                        &now
                    ],
                )?;
                new_id
            }
        };

        Self::list_tasks(conn, &input.session_id)?
            .into_iter()
            .find(|t| t.id == tid)
            .ok_or_else(|| DbError::Logic("Task non trovata".into()))
    }

    pub fn toggle_task(conn: &Connection, task_id: &str, done: bool) -> DbResult<()> {
        let val = if done { 1 } else { 0 };
        conn.execute("UPDATE tasks SET done = ?1 WHERE id = ?2", params![val, task_id])?;
        conn.execute("UPDATE task_subitems SET done = ?1 WHERE task_id = ?2", params![val, task_id])?;
        Ok(())
    }

    pub fn delete_task(conn: &Connection, task_id: &str) -> DbResult<()> {
        conn.execute("DELETE FROM tasks WHERE id = ?1", params![task_id])?;
        Ok(())
    }

    pub fn upsert_task_subitem(conn: &Connection, input: &UpsertTaskSubItemInput) -> DbResult<()> {
        match input.id.as_deref().filter(|s| !s.trim().is_empty()) {
            Some(sid) => {
                conn.execute(
                    "UPDATE task_subitems SET text = ?1, category = ?2, pages = ?3 WHERE id = ?4",
                    params![
                        input.text.trim(),
                        input.category.as_deref().unwrap_or("").trim(),
                        input.pages.as_deref().unwrap_or("").trim(),
                        sid
                    ],
                )?;
            }
            None => {
                let sid = format!("sub_{}", Uuid::new_v4().simple());
                let max_order: i32 = conn
                    .query_row(
                        "SELECT COALESCE(MAX(sort_order), -1) + 1 FROM task_subitems WHERE task_id = ?1",
                        params![&input.task_id],
                        |r| r.get(0),
                    )
                    .unwrap_or(0);
                conn.execute(
                    r#"
                    INSERT INTO task_subitems (id, task_id, text, category, pages, done, sort_order, plan_subtask_id)
                    VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, NULL)
                    "#,
                    params![
                        &sid,
                        &input.task_id,
                        input.text.trim(),
                        input.category.as_deref().unwrap_or("").trim(),
                        input.pages.as_deref().unwrap_or("").trim(),
                        if input.done.unwrap_or(false) { 1 } else { 0 },
                        input.sort_order.unwrap_or(max_order)
                    ],
                )?;
            }
        }
        Ok(())
    }

    pub fn toggle_task_subitem(conn: &Connection, subitem_id: &str, done: bool) -> DbResult<()> {
        conn.execute(
            "UPDATE task_subitems SET done = ?1 WHERE id = ?2",
            params![if done { 1 } else { 0 }, subitem_id],
        )?;
        if let Some(task_id) = conn
            .query_row(
                "SELECT task_id FROM task_subitems WHERE id = ?1",
                params![subitem_id],
                |r| r.get::<_, String>(0),
            )
            .optional()?
        {
            let counts: (i32, i32) = conn
                .query_row(
                    "SELECT COUNT(*), SUM(CASE WHEN done = 1 THEN 1 ELSE 0 END) FROM task_subitems WHERE task_id = ?1",
                    params![&task_id],
                    |r| Ok((r.get(0)?, r.get::<_, Option<i32>>(1)?.unwrap_or(0))),
                )
                .unwrap_or((0, 0));
            if counts.0 > 0 {
                conn.execute(
                    "UPDATE tasks SET done = ?1 WHERE id = ?2",
                    params![if counts.0 == counts.1 { 1 } else { 0 }, &task_id],
                )?;
            }
        }
        Ok(())
    }

    pub fn delete_task_subitem(conn: &Connection, subitem_id: &str) -> DbResult<()> {
        conn.execute("DELETE FROM task_subitems WHERE id = ?1", params![subitem_id])?;
        Ok(())
    }

    // =========================================================================
    // Quizzes & Attempts
    // =========================================================================

    pub fn list_quiz_attempts(conn: &Connection, session_id: &str) -> DbResult<Vec<QuizAttempt>> {
        let mut stmt = conn.prepare(
            r#"
            SELECT id, quiz_id, session_id, course_id, score_pct, correct_count,
                   total_count, answers_json, wrong_question_ids_json, completed_at
            FROM quiz_attempts WHERE session_id = ?1 ORDER BY completed_at DESC
            "#,
        )?;
        let rows = stmt.query_map(params![session_id], |row| {
            let wrong_raw: String = row.get(8)?;
            Ok(QuizAttempt {
                id: row.get(0)?,
                quiz_id: row.get(1)?,
                session_id: row.get(2)?,
                course_id: row.get(3)?,
                score_pct: row.get(4)?,
                correct_count: row.get(5)?,
                total_count: row.get(6)?,
                answers_json: row.get(7)?,
                wrong_question_ids: serde_json::from_str(&wrong_raw).unwrap_or_default(),
                completed_at: row.get(9)?,
            })
        })?;
        let mut out = Vec::new();
        for r in rows {
            out.push(r?);
        }
        Ok(out)
    }

    pub fn list_quizzes(conn: &Connection, session_id: &str) -> DbResult<Vec<DailyQuiz>> {
        let mut questions_by_quiz: HashMap<String, Vec<QuizQuestion>> = HashMap::new();
        {
            let mut qstmt = conn.prepare(
                r#"
                SELECT qq.id, qq.quiz_id, qq.question_order, qq.question_type, qq.category,
                       qq.chapter, qq.topic, qq.prompt, qq.options_json, qq.correct_index,
                       qq.explanation, qq.plan_question_id
                FROM quiz_questions qq
                INNER JOIN daily_quizzes dq ON qq.quiz_id = dq.id
                WHERE dq.session_id = ?1
                ORDER BY qq.question_order ASC
                "#,
            )?;
            let qrows = qstmt.query_map(params![session_id], |row| {
                let opts_raw: String = row.get(8)?;
                Ok(QuizQuestion {
                    id: row.get(0)?,
                    quiz_id: row.get(1)?,
                    question_order: row.get(2)?,
                    question_type: row.get(3)?,
                    category: row.get(4)?,
                    chapter: row.get(5)?,
                    topic: row.get(6)?,
                    prompt: row.get(7)?,
                    options: serde_json::from_str(&opts_raw).unwrap_or_default(),
                    correct_index: row.get(9)?,
                    explanation: row.get(10)?,
                    plan_question_id: row.get(11)?,
                })
            })?;
            for r in qrows {
                let q = r?;
                questions_by_quiz.entry(q.quiz_id.clone()).or_default().push(q);
            }
        }

        let attempts = Self::list_quiz_attempts(conn, session_id)?;
        let mut attempts_by_quiz: HashMap<String, Vec<QuizAttempt>> = HashMap::new();
        for a in attempts {
            attempts_by_quiz.entry(a.quiz_id.clone()).or_default().push(a);
        }

        let mut stmt = conn.prepare(
            r#"
            SELECT id, session_id, course_id, date, title, topic_summary, plan_quiz_id
            FROM daily_quizzes WHERE session_id = ?1 ORDER BY date ASC, title ASC
            "#,
        )?;
        let rows = stmt.query_map(params![session_id], |row| {
            Ok(DailyQuiz {
                id: row.get(0)?,
                session_id: row.get(1)?,
                course_id: row.get(2)?,
                date: row.get(3)?,
                title: row.get(4)?,
                topic_summary: row.get(5)?,
                plan_quiz_id: row.get(6)?,
                questions: Vec::new(),
                latest_attempt: None,
                attempts_count: 0,
            })
        })?;

        let mut out = Vec::new();
        for r in rows {
            let mut quiz = r?;
            quiz.questions = questions_by_quiz.remove(&quiz.id).unwrap_or_default();
            if let Some(list) = attempts_by_quiz.get(&quiz.id) {
                quiz.attempts_count = list.len();
                quiz.latest_attempt = list.first().cloned();
            }
            out.push(quiz);
        }
        Ok(out)
    }

    pub fn submit_quiz_attempt(
        conn: &Connection,
        input: &SubmitQuizAttemptInput,
    ) -> DbResult<QuizAttempt> {
        let id = format!("att_{}", Uuid::new_v4().simple());
        let now = Utc::now().to_rfc3339();
        let wrong_json = serde_json::to_string(&input.wrong_question_ids)?;
        conn.execute(
            r#"
            INSERT INTO quiz_attempts (
                id, quiz_id, session_id, course_id, score_pct, correct_count,
                total_count, answers_json, wrong_question_ids_json, completed_at
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)
            "#,
            params![
                &id,
                &input.quiz_id,
                &input.session_id,
                &input.course_id,
                input.score_pct,
                input.correct_count,
                input.total_count,
                &input.answers_json,
                &wrong_json,
                &now
            ],
        )?;
        Ok(QuizAttempt {
            id,
            quiz_id: input.quiz_id.clone(),
            session_id: input.session_id.clone(),
            course_id: input.course_id.clone(),
            score_pct: input.score_pct,
            correct_count: input.correct_count,
            total_count: input.total_count,
            answers_json: input.answers_json.clone(),
            wrong_question_ids: input.wrong_question_ids.clone(),
            completed_at: now,
        })
    }

    // =========================================================================
    // Weekly Lessons, Calendar Events, Daily Notes
    // =========================================================================

    pub fn list_weekly_lessons(conn: &Connection, session_id: &str) -> DbResult<Vec<WeeklyLesson>> {
        let mut stmt = conn.prepare(
            r#"
            SELECT id, session_id, course_id, course_name, day_of_week,
                   start_time, end_time, classroom, professor, color_hex
            FROM weekly_lessons WHERE session_id = ?1 ORDER BY day_of_week ASC, start_time ASC
            "#,
        )?;
        let rows = stmt.query_map(params![session_id], |row| {
            Ok(WeeklyLesson {
                id: row.get(0)?,
                session_id: row.get(1)?,
                course_id: row.get(2)?,
                course_name: row.get(3)?,
                day_of_week: row.get(4)?,
                start_time: row.get(5)?,
                end_time: row.get(6)?,
                classroom: row.get(7)?,
                professor: row.get(8)?,
                color_hex: row.get(9)?,
            })
        })?;
        let mut out = Vec::new();
        for r in rows {
            out.push(r?);
        }
        Ok(out)
    }

    pub fn upsert_weekly_lesson(
        conn: &Connection,
        input: &UpsertWeeklyLessonInput,
    ) -> DbResult<WeeklyLesson> {
        let lid = match input.id.as_deref().filter(|s| !s.trim().is_empty()) {
            Some(eid) => {
                conn.execute(
                    r#"
                    UPDATE weekly_lessons SET
                        course_id = ?1, course_name = ?2, day_of_week = ?3,
                        start_time = ?4, end_time = ?5, classroom = ?6,
                        professor = ?7, color_hex = ?8
                    WHERE id = ?9
                    "#,
                    params![
                        &input.course_id,
                        input.course_name.trim(),
                        input.day_of_week.clamp(1, 7),
                        input.start_time.trim(),
                        input.end_time.trim(),
                        input.classroom.as_deref().unwrap_or("").trim(),
                        input.professor.as_deref().unwrap_or("").trim(),
                        input.color_hex.as_deref().unwrap_or("#7C5CFC").trim(),
                        eid
                    ],
                )?;
                eid.to_string()
            }
            None => {
                let new_id = format!("les_{}", Uuid::new_v4().simple());
                conn.execute(
                    r#"
                    INSERT INTO weekly_lessons (
                        id, session_id, course_id, course_name, day_of_week,
                        start_time, end_time, classroom, professor, color_hex
                    ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)
                    "#,
                    params![
                        &new_id,
                        &input.session_id,
                        &input.course_id,
                        input.course_name.trim(),
                        input.day_of_week.clamp(1, 7),
                        input.start_time.trim(),
                        input.end_time.trim(),
                        input.classroom.as_deref().unwrap_or("").trim(),
                        input.professor.as_deref().unwrap_or("").trim(),
                        input.color_hex.as_deref().unwrap_or("#7C5CFC").trim()
                    ],
                )?;
                new_id
            }
        };

        Self::list_weekly_lessons(conn, &input.session_id)?
            .into_iter()
            .find(|l| l.id == lid)
            .ok_or_else(|| DbError::Logic("Lezione non trovata".into()))
    }

    pub fn delete_weekly_lesson(conn: &Connection, lesson_id: &str) -> DbResult<()> {
        conn.execute("DELETE FROM weekly_lessons WHERE id = ?1", params![lesson_id])?;
        Ok(())
    }

    pub fn list_calendar_events(conn: &Connection, session_id: &str) -> DbResult<Vec<CalendarEvent>> {
        let mut stmt = conn.prepare(
            r#"
            SELECT id, session_id, course_id, date, time_slot, title, category, details
            FROM calendar_events WHERE session_id = ?1 ORDER BY date ASC, time_slot ASC
            "#,
        )?;
        let rows = stmt.query_map(params![session_id], |row| {
            Ok(CalendarEvent {
                id: row.get(0)?,
                session_id: row.get(1)?,
                course_id: row.get(2)?,
                date: row.get(3)?,
                time_slot: row.get(4)?,
                title: row.get(5)?,
                category: row.get(6)?,
                details: row.get(7)?,
            })
        })?;
        let mut out = Vec::new();
        for r in rows {
            out.push(r?);
        }
        Ok(out)
    }

    pub fn upsert_calendar_event(conn: &Connection, input: &UpsertCalendarEventInput) -> DbResult<()> {
        match input.id.as_deref().filter(|s| !s.trim().is_empty()) {
            Some(eid) => {
                conn.execute(
                    r#"
                    UPDATE calendar_events SET
                        course_id = ?1, date = ?2, time_slot = ?3, title = ?4, category = ?5, details = ?6
                    WHERE id = ?7
                    "#,
                    params![
                        &input.course_id,
                        input.date.trim(),
                        input.time_slot.as_deref().unwrap_or("").trim(),
                        input.title.trim(),
                        input.category.trim(),
                        input.details.as_deref().unwrap_or("").trim(),
                        eid
                    ],
                )?;
            }
            None => {
                let eid = format!("evt_{}", Uuid::new_v4().simple());
                conn.execute(
                    r#"
                    INSERT INTO calendar_events (id, session_id, course_id, date, time_slot, title, category, details)
                    VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)
                    "#,
                    params![
                        &eid,
                        &input.session_id,
                        &input.course_id,
                        input.date.trim(),
                        input.time_slot.as_deref().unwrap_or("").trim(),
                        input.title.trim(),
                        input.category.trim(),
                        input.details.as_deref().unwrap_or("").trim()
                    ],
                )?;
            }
        }
        Ok(())
    }

    pub fn delete_calendar_event(conn: &Connection, event_id: &str) -> DbResult<()> {
        conn.execute("DELETE FROM calendar_events WHERE id = ?1", params![event_id])?;
        Ok(())
    }

    pub fn list_daily_notes(conn: &Connection, session_id: &str) -> DbResult<Vec<DailyNote>> {
        let mut stmt = conn.prepare(
            "SELECT session_id, date, content_markdown, updated_at FROM daily_notes WHERE session_id = ?1 ORDER BY date ASC",
        )?;
        let rows = stmt.query_map(params![session_id], |row| {
            Ok(DailyNote {
                session_id: row.get(0)?,
                date: row.get(1)?,
                content_markdown: row.get(2)?,
                updated_at: row.get(3)?,
            })
        })?;
        let mut out = Vec::new();
        for r in rows {
            out.push(r?);
        }
        Ok(out)
    }

    pub fn save_daily_note(
        conn: &Connection,
        session_id: &str,
        date: &str,
        content_markdown: &str,
    ) -> DbResult<()> {
        let now = Utc::now().to_rfc3339();
        conn.execute(
            r#"
            INSERT INTO daily_notes (session_id, date, content_markdown, updated_at)
            VALUES (?1, ?2, ?3, ?4)
            ON CONFLICT(session_id, date) DO UPDATE SET
                content_markdown = excluded.content_markdown,
                updated_at = excluded.updated_at
            "#,
            params![session_id, date.trim(), content_markdown, &now],
        )?;
        Ok(())
    }

    // =========================================================================
    // AI Undo Snapshots
    // =========================================================================

    pub fn save_ai_undo_snapshot(
        conn: &Connection,
        session_id: &str,
        description: &str,
    ) -> DbResult<String> {
        let tasks = Self::list_tasks(conn, session_id)?;
        let exams = Self::list_exams(conn, session_id)?;
        let courses = Self::list_courses(conn, session_id)?;
        let weekly_lessons = Self::list_weekly_lessons(conn, session_id)?;
        let payload = SessionUndoPayload {
            tasks,
            exams,
            courses,
            weekly_lessons,
        };
        let json = serde_json::to_string(&payload)?;
        let snap_id = format!("undo_{}", Uuid::new_v4().simple());
        conn.execute(
            "INSERT INTO ai_undo_snapshots (id, session_id, description, snapshot_json, created_at) VALUES (?1, ?2, ?3, ?4, ?5)",
            params![&snap_id, session_id, description, &json, Utc::now().to_rfc3339()],
        )?;
        Ok(snap_id)
    }

    pub fn restore_ai_undo_snapshot(conn: &mut Connection, snapshot_id: &str) -> DbResult<String> {
        let (session_id, json): (String, String) = conn
            .query_row(
                "SELECT session_id, snapshot_json FROM ai_undo_snapshots WHERE id = ?1",
                params![snapshot_id],
                |r| Ok((r.get(0)?, r.get(1)?)),
            )
            .map_err(|_| DbError::Logic("Snapshot Undo non trovato".into()))?;

        let payload: SessionUndoPayload = serde_json::from_str(&json)?;
        let tx = conn.transaction()?;

        tx.execute("DELETE FROM tasks WHERE session_id = ?1", params![&session_id])?;
        tx.execute("DELETE FROM exams WHERE session_id = ?1", params![&session_id])?;
        tx.execute("DELETE FROM weekly_lessons WHERE session_id = ?1", params![&session_id])?;
        tx.execute("DELETE FROM courses WHERE session_id = ?1", params![&session_id])?;

        for c in &payload.courses {
            tx.execute(
                r#"
                INSERT INTO courses (
                    id, session_id, plan_course_id, sigla, name, code, cfu, professor,
                    color_hex, difficulty_weight, target_hours, is_active, notes, sort_order
                ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14)
                "#,
                params![
                    &c.id, &c.session_id, &c.plan_course_id, &c.sigla, &c.name, &c.code, c.cfu,
                    &c.professor, &c.color_hex, c.difficulty_weight, c.target_hours,
                    if c.is_active { 1 } else { 0 }, &c.notes, c.sort_order
                ],
            )?;
        }

        for ex in &payload.exams {
            tx.execute(
                r#"
                INSERT INTO exams (
                    id, session_id, course_id, plan_exam_id, date, time_slot, duration_minutes,
                    round, exam_type, classroom, title, details, status, grade
                ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14)
                "#,
                params![
                    &ex.id, &ex.session_id, &ex.course_id, &ex.plan_exam_id, &ex.date, &ex.time_slot,
                    ex.duration_minutes, &ex.round, &ex.exam_type, &ex.classroom, &ex.title,
                    &ex.details, &ex.status, &ex.grade
                ],
            )?;
        }

        for t in &payload.tasks {
            tx.execute(
                r#"
                INSERT INTO tasks (
                    id, session_id, course_id, unit_id, date, category, title,
                    chapter, topic, pages, source, planned_hours, done, priority, plan_task_id, created_at
                ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16)
                "#,
                params![
                    &t.id, &t.session_id, &t.course_id, &t.unit_id, &t.date, &t.category, &t.title,
                    &t.chapter, &t.topic, &t.pages, &t.source, t.planned_hours,
                    if t.done { 1 } else { 0 }, t.priority, &t.plan_task_id, &t.created_at
                ],
            )?;
            for s in &t.subtasks {
                tx.execute(
                    r#"
                    INSERT INTO task_subitems (id, task_id, text, category, pages, done, sort_order, plan_subtask_id)
                    VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)
                    "#,
                    params![
                        &s.id, &s.task_id, &s.text, &s.category, &s.pages,
                        if s.done { 1 } else { 0 }, s.sort_order, &s.plan_subtask_id
                    ],
                )?;
            }
        }

        for l in &payload.weekly_lessons {
            tx.execute(
                r#"
                INSERT INTO weekly_lessons (
                    id, session_id, course_id, course_name, day_of_week,
                    start_time, end_time, classroom, professor, color_hex
                ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)
                "#,
                params![
                    &l.id, &l.session_id, &l.course_id, &l.course_name, l.day_of_week,
                    &l.start_time, &l.end_time, &l.classroom, &l.professor, &l.color_hex
                ],
            )?;
        }

        tx.execute("DELETE FROM ai_undo_snapshots WHERE id = ?1", params![snapshot_id])?;
        tx.commit()?;
        Ok(session_id)
    }

    // =========================================================================
    // Snapshot & Bootstrap
    // =========================================================================

    pub fn build_session_snapshot(
        conn: &Connection,
        session_id: &str,
        settings: &AppSettings,
    ) -> DbResult<SessionSnapshot> {
        let session = Self::get_session(conn, session_id)?
            .ok_or_else(|| DbError::Logic(format!("Sessione '{}' non trovata", session_id)))?;
        let courses = Self::list_courses(conn, session_id)?;
        let study_units = Self::list_study_units(conn, session_id)?;
        let exams = Self::list_exams(conn, session_id)?;
        let operational_plans = Self::list_operational_plans(conn, session_id)?;
        let tasks = Self::list_tasks(conn, session_id)?;
        let quizzes = Self::list_quizzes(conn, session_id)?;
        let quiz_attempts = Self::list_quiz_attempts(conn, session_id)?;
        let weekly_lessons = Self::list_weekly_lessons(conn, session_id)?;
        let calendar_events = Self::list_calendar_events(conn, session_id)?;
        let daily_notes = Self::list_daily_notes(conn, session_id)?;

        let conflicts = detect_session_conflicts(&courses, &study_units, &exams);
        let telemetry = compute_session_telemetry(
            &session,
            &courses,
            &study_units,
            &exams,
            &tasks,
            &quizzes,
            &quiz_attempts,
            settings.study_hours_per_cfu,
            None,
        );

        Ok(SessionSnapshot {
            session,
            courses,
            study_units,
            exams,
            operational_plans,
            tasks,
            quizzes,
            quiz_attempts,
            weekly_lessons,
            calendar_events,
            daily_notes,
            conflicts,
            telemetry,
        })
    }

    pub fn build_bootstrap_payload(&self, conn: &Connection) -> DbResult<BootstrapPayload> {
        let mut settings = Self::get_settings(conn)?;
        let sessions = Self::list_sessions(conn)?;

        let active_id = settings
            .active_session_id
            .as_ref()
            .filter(|id| sessions.iter().any(|s| &s.id == *id))
            .cloned()
            .or_else(|| sessions.first().map(|s| s.id.clone()));

        if active_id != settings.active_session_id {
            settings.active_session_id = active_id.clone();
            let _ = conn.execute(
                "UPDATE app_settings SET active_session_id = ?1 WHERE id = 1",
                params![&active_id],
            );
        }

        let active_snapshot = match active_id {
            Some(ref sid) => Self::build_session_snapshot(conn, sid, &settings).ok(),
            None => None,
        };

        let pending_plans = if !settings.plan_folder.trim().is_empty() {
            scan_folder_for_pending_plans(conn, &settings.plan_folder)
        } else {
            Vec::new()
        };

        Ok(BootstrapPayload {
            app_version: crate::APP_VERSION.to_string(),
            db_path: self.db_path.to_string_lossy().to_string(),
            settings,
            sessions,
            active_snapshot,
            pending_plans,
        })
    }
}
