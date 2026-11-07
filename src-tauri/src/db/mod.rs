pub mod migrations;

use crate::engine::{conflicts::detect_session_conflicts, workload::compute_session_telemetry};
use crate::models::{
    AcademicSession, AppSettings, BootstrapPayload, Course, CreateItinerePairInput, DailyNote,
    Exam, FullWorkspaceBackup, GenericEvent, SessionSnapshot, StudyUnit, StudyUnitDraft, TodoItem,
    UpsertCourseInput, UpsertExamInput, UpsertGenericEventInput, UpsertProfileInput,
    UpsertSessionInput, UpsertTodoInput, UpsertWeeklyLessonInput, UserProfile, WeeklyLesson,
};
use chrono::Utc;
use rusqlite::{params, Connection, OptionalExtension};
use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Mutex;
use thiserror::Error;
use uuid::Uuid;

#[derive(Debug, Error)]
pub enum DbError {
    #[error("Errore database SQLite: {0}")]
    Sqlite(#[from] rusqlite::Error),
    #[error("Errore I/O filesystem: {0}")]
    Io(#[from] std::io::Error),
    #[error("Errore serializzazione JSON: {0}")]
    Json(#[from] serde_json::Error),
    #[error("Errore lock concorrenza database")]
    LockPoisoned,
    #[error("Risorsa non trovata: {0}")]
    NotFound(String),
    #[error("Errore di validazione: {0}")]
    Validation(String),
}

pub type DbResult<T> = std::result::Result<T, DbError>;

pub struct DatabaseManager {
    pub db_path: PathBuf,
    pub conn: Mutex<Connection>,
}

impl DatabaseManager {
    /// Risolve la directory isolata dell'applicazione in %APPDATA%\com.mysession.app
    pub fn resolve_app_data_dir() -> PathBuf {
        if let Ok(appdata) = std::env::var("APPDATA") {
            PathBuf::from(appdata).join("com.mysession.app")
        } else {
            PathBuf::from(".").join("data")
        }
    }

    /// Initializes the SQLite database inside the OS AppData directory (`%APPDATA%\com.mysession.app\mysession.db`)
    pub fn init_in_dir(app_data_dir: &Path) -> DbResult<Self> {
        if !app_data_dir.exists() {
            fs::create_dir_all(app_data_dir)?;
        }
        let db_path = app_data_dir.join("mysession.db");
        let mut conn = Connection::open(&db_path)?;
        migrations::run_migrations(&mut conn)?;

        Ok(Self {
            db_path,
            conn: Mutex::new(conn),
        })
    }

    /// Initializes an in-memory SQLite database (used for deterministic unit tests)
    pub fn init_in_memory() -> DbResult<Self> {
        let mut conn = Connection::open_in_memory()?;
        migrations::run_migrations(&mut conn)?;
        Ok(Self {
            db_path: PathBuf::from(":memory:"),
            conn: Mutex::new(conn),
        })
    }

    pub fn with_conn<F, T>(&self, f: F) -> DbResult<T>
    where
        F: FnOnce(&mut Connection) -> DbResult<T>,
    {
        let mut guard = self.conn.lock().map_err(|_| DbError::LockPoisoned)?;
        f(&mut guard)
    }

    // ========================================================================
    // Profile & Settings
    // ========================================================================

    pub fn get_profile(conn: &Connection) -> DbResult<UserProfile> {
        let profile = conn.query_row(
            r#"
            SELECT id, student_name, university, faculty, degree_course,
                   enrollment_year, onboarding_completed, created_at, updated_at
            FROM user_profile WHERE id = 1
            "#,
            [],
            |row| {
                Ok(UserProfile {
                    id: row.get(0)?,
                    student_name: row.get(1)?,
                    university: row.get(2)?,
                    faculty: row.get(3)?,
                    degree_course: row.get(4)?,
                    enrollment_year: row.get(5)?,
                    onboarding_completed: row.get::<_, i32>(6)? != 0,
                    created_at: row.get(7)?,
                    updated_at: row.get(8)?,
                })
            },
        )?;
        Ok(profile)
    }

    pub fn upsert_profile(conn: &Connection, input: &UpsertProfileInput) -> DbResult<UserProfile> {
        let now_iso = Utc::now().to_rfc3339();
        conn.execute(
            r#"
            UPDATE user_profile
            SET student_name = ?1,
                university = ?2,
                faculty = ?3,
                degree_course = ?4,
                enrollment_year = ?5,
                onboarding_completed = ?6,
                updated_at = ?7
            WHERE id = 1
            "#,
            params![
                input.student_name.trim(),
                input.university.trim(),
                input.faculty.trim(),
                input.degree_course.trim(),
                input.enrollment_year,
                if input.onboarding_completed { 1 } else { 0 },
                now_iso
            ],
        )?;
        Self::get_profile(conn)
    }

    pub fn get_settings(conn: &Connection) -> DbResult<AppSettings> {
        let settings = conn.query_row(
            r#"
            SELECT theme, backdrop_material, window_controls_style, window_frame_style,
                   ui_scale, sidebar_state, default_view, start_day_of_week,
                   weekly_holiday_day, study_hours_per_cfu, accent_color, holiday_color,
                   sunday_color, cat_studio_color, cat_scadenza_color, cat_lezione_color,
                   cat_personale_color, ai_provider, ai_api_key, ai_endpoint_url,
                   ai_model, ai_bar_width, active_session_id, plan_folder,
                   ai_local_model_id, ai_idle_timeout_minutes, ai_boot_strategy, ai_hardware_preference
            FROM app_settings WHERE id = 1
            "#,
            [],
            |row| {
                Ok(AppSettings {
                    theme: row.get(0)?,
                    backdrop_material: row.get(1)?,
                    window_controls_style: row.get(2)?,
                    window_frame_style: row.get(3)?,
                    ui_scale: row.get(4)?,
                    sidebar_state: row.get(5)?,
                    default_view: row.get(6)?,
                    start_day_of_week: row.get(7)?,
                    weekly_holiday_day: row.get(8)?,
                    study_hours_per_cfu: row.get(9)?,
                    accent_color: row.get(10)?,
                    holiday_color: row.get(11)?,
                    sunday_color: row.get(12)?,
                    cat_studio_color: row.get(13)?,
                    cat_scadenza_color: row.get(14)?,
                    cat_lezione_color: row.get(15)?,
                    cat_personale_color: row.get(16)?,
                    ai_provider: row.get(17)?,
                    ai_api_key: row.get(18)?,
                    ai_endpoint_url: row.get(19)?,
                    ai_model: row.get(20)?,
                    ai_bar_width: row.get(21)?,
                    active_session_id: row.get(22)?,
                    plan_folder: row.get::<_, Option<String>>(23)?.unwrap_or_default(),
                    ai_local_model_id: row.get::<_, Option<String>>(24)?.unwrap_or_else(|| "qwen2.5-vl-3b".to_string()),
                    ai_idle_timeout_minutes: row.get::<_, Option<i32>>(25)?.unwrap_or(5),
                    ai_boot_strategy: row.get::<_, Option<String>>(26)?.unwrap_or_else(|| "on_demand".to_string()),
                    ai_hardware_preference: row.get::<_, Option<String>>(27)?.unwrap_or_else(|| "auto".to_string()),
                })
            },
        )?;
        Ok(settings)
    }

    pub fn save_settings(conn: &Connection, s: &AppSettings) -> DbResult<AppSettings> {
        let clamped_scale = s.ui_scale.clamp(80, 150);
        let clamped_ai_w = s.ai_bar_width.clamp(380, 1400);
        conn.execute(
            r#"
            UPDATE app_settings
            SET theme = ?1,
                backdrop_material = ?2,
                window_controls_style = ?3,
                window_frame_style = ?4,
                ui_scale = ?5,
                sidebar_state = ?6,
                default_view = ?7,
                start_day_of_week = ?8,
                weekly_holiday_day = ?9,
                study_hours_per_cfu = ?10,
                accent_color = ?11,
                holiday_color = ?12,
                sunday_color = ?13,
                cat_studio_color = ?14,
                cat_scadenza_color = ?15,
                cat_lezione_color = ?16,
                cat_personale_color = ?17,
                ai_provider = ?18,
                ai_api_key = ?19,
                ai_endpoint_url = ?20,
                ai_model = ?21,
                ai_bar_width = ?22,
                active_session_id = ?23,
                plan_folder = ?24,
                ai_local_model_id = ?25,
                ai_idle_timeout_minutes = ?26,
                ai_boot_strategy = ?27,
                ai_hardware_preference = ?28
            WHERE id = 1
            "#,
            params![
                s.theme,
                s.backdrop_material,
                s.window_controls_style,
                s.window_frame_style,
                clamped_scale,
                s.sidebar_state,
                s.default_view,
                s.start_day_of_week,
                s.weekly_holiday_day,
                s.study_hours_per_cfu,
                s.accent_color,
                s.holiday_color,
                s.sunday_color,
                s.cat_studio_color,
                s.cat_scadenza_color,
                s.cat_lezione_color,
                s.cat_personale_color,
                s.ai_provider,
                s.ai_api_key,
                s.ai_endpoint_url,
                s.ai_model,
                clamped_ai_w,
                s.active_session_id,
                s.plan_folder,
                s.ai_local_model_id,
                s.ai_idle_timeout_minutes,
                s.ai_boot_strategy,
                s.ai_hardware_preference
            ],
        )?;
        Self::get_settings(conn)
    }

    // ========================================================================
    // Academic Sessions
    // ========================================================================

    pub fn list_sessions(conn: &Connection) -> DbResult<Vec<AcademicSession>> {
        let mut stmt = conn.prepare(
            r#"
            SELECT id, name, subtitle, session_type, academic_year, start_date, end_date, notes, created_at, plan_id, is_system
            FROM academic_sessions
            ORDER BY is_system DESC, start_date ASC, created_at ASC
            "#,
        )?;
        let rows = stmt.query_map([], |row| {
            Ok(AcademicSession {
                id: row.get(0)?,
                name: row.get(1)?,
                subtitle: row.get(2)?,
                session_type: row.get(3)?,
                academic_year: row.get(4)?,
                start_date: row.get(5)?,
                end_date: row.get(6)?,
                notes: row.get(7)?,
                created_at: row.get(8)?,
                plan_id: row.get(9)?,
                is_system: row.get::<_, i64>(10).unwrap_or(0) != 0,
            })
        })?;
        let mut out = Vec::new();
        for r in rows {
            out.push(r?);
        }
        Ok(out)
    }

    pub fn get_session(conn: &Connection, session_id: &str) -> DbResult<AcademicSession> {
        conn.query_row(
            r#"
            SELECT id, name, subtitle, session_type, academic_year, start_date, end_date, notes, created_at, plan_id, is_system
            FROM academic_sessions
            WHERE id = ?1
            "#,
            params![session_id],
            |row| {
                Ok(AcademicSession {
                    id: row.get(0)?,
                    name: row.get(1)?,
                    subtitle: row.get(2)?,
                    session_type: row.get(3)?,
                    academic_year: row.get(4)?,
                    start_date: row.get(5)?,
                    end_date: row.get(6)?,
                    notes: row.get(7)?,
                    created_at: row.get(8)?,
                    plan_id: row.get(9)?,
                    is_system: row.get::<_, i64>(10).unwrap_or(0) != 0,
                })
            },
        )
        .optional()?
        .ok_or_else(|| DbError::NotFound(format!("Sessione '{}' non trovata", session_id)))
    }

    pub fn upsert_session(
        conn: &Connection,
        input: &UpsertSessionInput,
    ) -> DbResult<AcademicSession> {
        if let Some(ref sid) = input.id {
            if sid == "session_generale" {
                return Err(DbError::Validation("La sessione 'Generale' è protetta e non può essere modificata.".to_string()));
            }
        }

        let id = input
            .id
            .clone()
            .filter(|s| !s.trim().is_empty())
            .unwrap_or_else(|| format!("sess_{}", Uuid::new_v4().simple()));
        let now_iso = Utc::now().to_rfc3339();
        let notes = input.notes.clone().unwrap_or_default();

        conn.execute(
            r#"
            INSERT INTO academic_sessions (
                id, name, subtitle, session_type, academic_year, start_date, end_date, notes, created_at, plan_id, is_system
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, 0)
            ON CONFLICT(id) DO UPDATE SET
                name = excluded.name,
                subtitle = excluded.subtitle,
                session_type = excluded.session_type,
                academic_year = excluded.academic_year,
                start_date = excluded.start_date,
                end_date = excluded.end_date,
                notes = excluded.notes,
                plan_id = COALESCE(excluded.plan_id, academic_sessions.plan_id)
            WHERE academic_sessions.is_system = 0
            "#,
            params![
                &id,
                input.name.trim(),
                input.subtitle.trim(),
                input.session_type.trim(),
                input.academic_year.trim(),
                input.start_date.trim(),
                input.end_date.trim(),
                notes.trim(),
                &now_iso,
                input.plan_id.as_deref().map(str::trim)
            ],
        )?;

        // Set as active session in settings
        conn.execute(
            "UPDATE app_settings SET active_session_id = ?1 WHERE id = 1",
            params![&id],
        )?;

        Self::get_session(conn, &id)
    }

    pub fn set_active_session(conn: &Connection, session_id: &str) -> DbResult<()> {
        let _ = Self::get_session(conn, session_id)?;
        conn.execute(
            "UPDATE app_settings SET active_session_id = ?1 WHERE id = 1",
            params![session_id],
        )?;
        Ok(())
    }

    pub fn delete_session(conn: &Connection, session_id: &str) -> DbResult<()> {
        if session_id == "session_generale" {
            return Err(DbError::Validation("La sessione 'Generale' è protetta e non può essere eliminata.".to_string()));
        }
        let session = Self::get_session(conn, session_id)?;
        if session.is_system {
            return Err(DbError::Validation("Una sessione di sistema non può essere eliminata.".to_string()));
        }

        conn.execute(
            "DELETE FROM academic_sessions WHERE id = ?1",
            params![session_id],
        )?;

        let remaining = Self::list_sessions(conn)?;
        let new_active = remaining.first().map(|s| s.id.clone());
        conn.execute(
            "UPDATE app_settings SET active_session_id = ?1 WHERE id = 1",
            params![new_active],
        )?;
        Ok(())
    }

    // ========================================================================
    // Courses & Study Units
    // ========================================================================

    pub fn list_courses(conn: &Connection, session_id: &str) -> DbResult<Vec<Course>> {
        let mut stmt = conn.prepare(
            r#"
            SELECT id, session_id, sigla, name, code, cfu, professor, color_hex,
                   difficulty_weight, target_hours_override, notes, sort_order,
                   plan_course_id, is_visible
            FROM courses
            WHERE session_id = ?1
            ORDER BY sort_order ASC, sigla ASC
            "#,
        )?;
        let rows = stmt.query_map(params![session_id], |row| {
            Ok(Course {
                id: row.get(0)?,
                session_id: row.get(1)?,
                sigla: row.get(2)?,
                name: row.get(3)?,
                code: row.get(4)?,
                cfu: row.get(5)?,
                professor: row.get(6)?,
                color_hex: row.get(7)?,
                difficulty_weight: row.get(8)?,
                target_hours_override: row.get(9)?,
                notes: row.get(10)?,
                sort_order: row.get(11)?,
                plan_course_id: row.get(12)?,
                is_visible: row.get::<_, Option<i32>>(13)?.unwrap_or(1) != 0,
            })
        })?;
        let mut out = Vec::new();
        for r in rows {
            out.push(r?);
        }
        Ok(out)
    }

    pub fn set_course_visibility(
        conn: &Connection,
        course_id: &str,
        is_visible: bool,
    ) -> DbResult<()> {
        conn.execute(
            "UPDATE courses SET is_visible = ?1 WHERE id = ?2",
            params![if is_visible { 1 } else { 0 }, course_id],
        )?;
        Ok(())
    }

    pub fn list_study_units(conn: &Connection, session_id: &str) -> DbResult<Vec<StudyUnit>> {
        let mut stmt = conn.prepare(
            r#"
            SELECT id, course_id, session_id, unit_number, title, description,
                   estimated_hours, is_completed, assigned_date, completed_at, plan_unit_id
            FROM study_units
            WHERE session_id = ?1
            ORDER BY course_id ASC, unit_number ASC
            "#,
        )?;
        let rows = stmt.query_map(params![session_id], |row| {
            Ok(StudyUnit {
                id: row.get(0)?,
                course_id: row.get(1)?,
                session_id: row.get(2)?,
                unit_number: row.get(3)?,
                title: row.get(4)?,
                description: row.get(5)?,
                estimated_hours: row.get(6)?,
                is_completed: row.get::<_, i32>(7)? != 0,
                assigned_date: row.get(8)?,
                completed_at: row.get(9)?,
                plan_unit_id: row.get(10)?,
            })
        })?;
        let mut out = Vec::new();
        for r in rows {
            out.push(r?);
        }
        Ok(out)
    }

    pub fn upsert_course(conn: &mut Connection, input: &UpsertCourseInput) -> DbResult<Course> {
        let course_id = input
            .id
            .clone()
            .filter(|s| !s.trim().is_empty())
            .unwrap_or_else(|| format!("crs_{}", Uuid::new_v4().simple()));
        let weight = input.difficulty_weight.unwrap_or(1.0).clamp(0.5, 2.0);
        let notes = input.notes.clone().unwrap_or_default();
        let is_visible = input.is_visible.unwrap_or(true);

        let tx = conn.transaction()?;
        tx.execute(
            r#"
            INSERT INTO courses (
                id, session_id, sigla, name, code, cfu, professor,
                color_hex, difficulty_weight, target_hours_override, notes, sort_order,
                plan_course_id, is_visible
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, 0, ?12, ?13)
            ON CONFLICT(id) DO UPDATE SET
                sigla = excluded.sigla,
                name = excluded.name,
                code = excluded.code,
                cfu = excluded.cfu,
                professor = excluded.professor,
                color_hex = excluded.color_hex,
                difficulty_weight = excluded.difficulty_weight,
                target_hours_override = excluded.target_hours_override,
                notes = excluded.notes,
                plan_course_id = COALESCE(excluded.plan_course_id, courses.plan_course_id),
                is_visible = excluded.is_visible
            "#,
            params![
                &course_id,
                &input.session_id,
                input.sigla.trim().to_uppercase(),
                input.name.trim(),
                input.code.trim(),
                input.cfu.max(1),
                input.professor.trim(),
                input.color_hex.trim(),
                weight,
                input.target_hours_override,
                notes.trim(),
                input.plan_course_id.as_deref().map(str::trim),
                if is_visible { 1 } else { 0 }
            ],
        )?;

        if let Some(ref units) = input.units {
            Self::sync_course_units(&tx, &input.session_id, &course_id, units)?;
        }

        tx.commit()?;

        let courses = Self::list_courses(conn, &input.session_id)?;
        courses
            .into_iter()
            .find(|c| c.id == course_id)
            .ok_or_else(|| DbError::NotFound("Corso salvato non trovato".to_string()))
    }

    fn sync_course_units(
        tx: &rusqlite::Transaction<'_>,
        session_id: &str,
        course_id: &str,
        units: &[StudyUnitDraft],
    ) -> DbResult<()> {
        tx.execute(
            "DELETE FROM study_units WHERE course_id = ?1",
            params![course_id],
        )?;
        for (idx, u) in units.iter().enumerate() {
            let uid = u
                .id
                .clone()
                .filter(|s| !s.trim().is_empty())
                .unwrap_or_else(|| format!("unit_{}", Uuid::new_v4().simple()));
            let is_comp = u.is_completed.unwrap_or(false);
            let comp_at = if is_comp {
                Some(Utc::now().to_rfc3339())
            } else {
                None
            };
            tx.execute(
                r#"
                INSERT INTO study_units (
                    id, course_id, session_id, unit_number, title, description,
                    estimated_hours, is_completed, assigned_date, completed_at, plan_unit_id
                ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)
                "#,
                params![
                    uid,
                    course_id,
                    session_id,
                    if u.unit_number > 0 {
                        u.unit_number
                    } else {
                        (idx as i32) + 1
                    },
                    u.title.trim(),
                    u.description.clone().unwrap_or_default().trim(),
                    u.estimated_hours.unwrap_or(4.0).max(0.5),
                    if is_comp { 1 } else { 0 },
                    u.assigned_date,
                    comp_at,
                    u.plan_unit_id.as_deref().map(str::trim)
                ],
            )?;
        }
        Ok(())
    }

    pub fn delete_course(conn: &Connection, course_id: &str) -> DbResult<()> {
        conn.execute("DELETE FROM courses WHERE id = ?1", params![course_id])?;
        Ok(())
    }

    pub fn toggle_study_unit(
        conn: &Connection,
        unit_id: &str,
        is_completed: bool,
    ) -> DbResult<()> {
        let comp_at = if is_completed {
            Some(Utc::now().to_rfc3339())
        } else {
            None
        };
        conn.execute(
            "UPDATE study_units SET is_completed = ?1, completed_at = ?2 WHERE id = ?3",
            params![if is_completed { 1 } else { 0 }, comp_at, unit_id],
        )?;
        Ok(())
    }

    pub fn assign_study_unit_date(
        conn: &Connection,
        unit_id: &str,
        assigned_date: Option<String>,
    ) -> DbResult<()> {
        conn.execute(
            "UPDATE study_units SET assigned_date = ?1 WHERE id = ?2",
            params![assigned_date, unit_id],
        )?;
        Ok(())
    }

    // ========================================================================
    // Exams & Prove in Itinere
    // ========================================================================

    pub fn list_exams(conn: &Connection, session_id: &str) -> DbResult<Vec<Exam>> {
        let mut stmt = conn.prepare(
            r#"
            SELECT id, session_id, course_id, date, time_slot, duration_minutes,
                   round, exam_type, classroom, weight_fraction, pair_id,
                   title, details, status, grade
            FROM exams
            WHERE session_id = ?1
            ORDER BY date ASC, time_slot ASC
            "#,
        )?;
        let rows = stmt.query_map(params![session_id], |row| {
            Ok(Exam {
                id: row.get(0)?,
                session_id: row.get(1)?,
                course_id: row.get(2)?,
                date: row.get(3)?,
                time_slot: row.get(4)?,
                duration_minutes: row.get(5)?,
                round: row.get(6)?,
                exam_type: row.get(7)?,
                classroom: row.get(8)?,
                weight_fraction: row.get(9)?,
                pair_id: row.get(10)?,
                title: row.get(11)?,
                details: row.get(12)?,
                status: row.get(13)?,
                grade: row.get(14)?,
            })
        })?;
        let mut out = Vec::new();
        for r in rows {
            out.push(r?);
        }
        Ok(out)
    }

    pub fn upsert_exam(conn: &Connection, input: &UpsertExamInput) -> DbResult<Exam> {
        let exam_id = input
            .id
            .clone()
            .filter(|s| !s.trim().is_empty())
            .unwrap_or_else(|| format!("ex_{}", Uuid::new_v4().simple()));

        let course_sigla: String = conn
            .query_row(
                "SELECT sigla FROM courses WHERE id = ?1",
                params![&input.course_id],
                |r| r.get(0),
            )
            .unwrap_or_else(|_| "Esame".to_string());

        let computed_title = input
            .title
            .clone()
            .filter(|t| !t.trim().is_empty())
            .unwrap_or_else(|| match input.round.as_str() {
                "itinere_1" => format!("1ª Prova in Itinere — {}", course_sigla),
                "itinere_2" => format!("2ª Prova in Itinere — {}", course_sigla),
                r => format!("{}° Appello — {}", r, course_sigla),
            });

        conn.execute(
            r#"
            INSERT INTO exams (
                id, session_id, course_id, date, time_slot, duration_minutes,
                round, exam_type, classroom, weight_fraction, pair_id,
                title, details, status, grade
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15)
            ON CONFLICT(id) DO UPDATE SET
                course_id = excluded.course_id,
                date = excluded.date,
                time_slot = excluded.time_slot,
                duration_minutes = excluded.duration_minutes,
                round = excluded.round,
                exam_type = excluded.exam_type,
                classroom = excluded.classroom,
                weight_fraction = excluded.weight_fraction,
                pair_id = excluded.pair_id,
                title = excluded.title,
                details = excluded.details,
                status = excluded.status,
                grade = excluded.grade
            "#,
            params![
                &exam_id,
                &input.session_id,
                &input.course_id,
                input.date.trim(),
                input.time_slot.clone().unwrap_or_default().trim(),
                input.duration_minutes.unwrap_or(120),
                input.round.trim(),
                input
                    .exam_type
                    .clone()
                    .unwrap_or_else(|| "scritto_orale".to_string()),
                input.classroom.clone().unwrap_or_default().trim(),
                input.weight_fraction.unwrap_or(1.0),
                input.pair_id,
                computed_title.trim(),
                input
                    .details
                    .clone()
                    .unwrap_or_else(|| "Appello d'esame programmato".to_string())
                    .trim(),
                input
                    .status
                    .clone()
                    .unwrap_or_else(|| "planned".to_string()),
                input.grade
            ],
        )?;

        let exams = Self::list_exams(conn, &input.session_id)?;
        exams
            .into_iter()
            .find(|e| e.id == exam_id)
            .ok_or_else(|| DbError::NotFound("Esame salvato non trovato".to_string()))
    }

    pub fn create_itinere_pair(
        conn: &mut Connection,
        input: &CreateItinerePairInput,
    ) -> DbResult<Vec<Exam>> {
        let pair_id = format!("pair_{}", Uuid::new_v4().simple());
        let ex1 = Self::upsert_exam(
            conn,
            &UpsertExamInput {
                id: None,
                session_id: input.session_id.clone(),
                course_id: input.course_id.clone(),
                date: input.date1.clone(),
                time_slot: input.time_slot1.clone(),
                duration_minutes: Some(120),
                round: "itinere_1".to_string(),
                exam_type: Some("itinere".to_string()),
                classroom: input.classroom.clone(),
                weight_fraction: Some(0.5),
                pair_id: Some(pair_id.clone()),
                title: None,
                details: input
                    .details
                    .clone()
                    .or_else(|| Some("1ª Prova parziale in itinere".to_string())),
                status: Some("planned".to_string()),
                grade: None,
            },
        )?;

        let ex2 = Self::upsert_exam(
            conn,
            &UpsertExamInput {
                id: None,
                session_id: input.session_id.clone(),
                course_id: input.course_id.clone(),
                date: input.date2.clone(),
                time_slot: input.time_slot2.clone(),
                duration_minutes: Some(120),
                round: "itinere_2".to_string(),
                exam_type: Some("itinere".to_string()),
                classroom: input.classroom.clone(),
                weight_fraction: Some(0.5),
                pair_id: Some(pair_id),
                title: None,
                details: input
                    .details
                    .clone()
                    .or_else(|| Some("2ª Prova parziale in itinere".to_string())),
                status: Some("planned".to_string()),
                grade: None,
            },
        )?;

        Ok(vec![ex1, ex2])
    }

    pub fn delete_exam(conn: &Connection, exam_id: &str) -> DbResult<()> {
        conn.execute("DELETE FROM exams WHERE id = ?1", params![exam_id])?;
        Ok(())
    }

    // ========================================================================
    // Generic Events, To-Dos & Daily Notes
    // ========================================================================

    pub fn list_generic_events(conn: &Connection, session_id: &str) -> DbResult<Vec<GenericEvent>> {
        let mut stmt = conn.prepare(
            r#"
            SELECT id, session_id, course_id, date, time_slot, title, category, details
            FROM generic_events
            WHERE session_id = ?1
            ORDER BY date ASC, time_slot ASC
            "#,
        )?;
        let rows = stmt.query_map(params![session_id], |row| {
            Ok(GenericEvent {
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

    pub fn upsert_generic_event(
        conn: &Connection,
        input: &UpsertGenericEventInput,
    ) -> DbResult<GenericEvent> {
        let id = input
            .id
            .clone()
            .filter(|s| !s.trim().is_empty())
            .unwrap_or_else(|| format!("evt_{}", Uuid::new_v4().simple()));

        conn.execute(
            r#"
            INSERT INTO generic_events (id, session_id, course_id, date, time_slot, title, category, details)
            VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)
            ON CONFLICT(id) DO UPDATE SET
                course_id = excluded.course_id,
                date = excluded.date,
                time_slot = excluded.time_slot,
                title = excluded.title,
                category = excluded.category,
                details = excluded.details
            "#,
            params![
                &id,
                &input.session_id,
                input.course_id,
                input.date.trim(),
                input.time_slot.clone().unwrap_or_default().trim(),
                input.title.trim(),
                input.category.trim(),
                input.details.clone().unwrap_or_default().trim()
            ],
        )?;

        let events = Self::list_generic_events(conn, &input.session_id)?;
        events
            .into_iter()
            .find(|e| e.id == id)
            .ok_or_else(|| DbError::NotFound("Evento salvato non trovato".to_string()))
    }

    pub fn delete_generic_event(conn: &Connection, event_id: &str) -> DbResult<()> {
        conn.execute(
            "DELETE FROM generic_events WHERE id = ?1",
            params![event_id],
        )?;
        Ok(())
    }

    pub fn list_todos(conn: &Connection, session_id: &str) -> DbResult<Vec<TodoItem>> {
        let mut stmt = conn.prepare(
            r#"
            SELECT id, session_id, course_id, date, text, done, priority, created_at,
                   plan_task_id, planned_hours, kind
            FROM todos
            WHERE session_id = ?1
            ORDER BY date ASC, priority DESC, created_at ASC
            "#,
        )?;
        let rows = stmt.query_map(params![session_id], |row| {
            Ok(TodoItem {
                id: row.get(0)?,
                session_id: row.get(1)?,
                course_id: row.get(2)?,
                date: row.get(3)?,
                text: row.get(4)?,
                done: row.get::<_, i32>(5)? != 0,
                priority: row.get(6)?,
                created_at: row.get(7)?,
                plan_task_id: row.get(8)?,
                planned_hours: row.get(9)?,
                kind: row.get(10)?,
            })
        })?;
        let mut out = Vec::new();
        for r in rows {
            out.push(r?);
        }
        Ok(out)
    }

    pub fn upsert_todo(conn: &Connection, input: &UpsertTodoInput) -> DbResult<TodoItem> {
        let id = input
            .id
            .clone()
            .filter(|s| !s.trim().is_empty())
            .unwrap_or_else(|| format!("todo_{}", Uuid::new_v4().simple()));
        let now_iso = Utc::now().to_rfc3339();

        conn.execute(
            r#"
            INSERT INTO todos (
                id, session_id, course_id, date, text, done, priority, created_at,
                plan_task_id, planned_hours, kind
            ) VALUES (?1, ?2, ?3, ?4, ?5, 0, ?6, ?7, ?8, ?9, ?10)
            ON CONFLICT(id) DO UPDATE SET
                course_id = excluded.course_id,
                date = excluded.date,
                text = excluded.text,
                priority = excluded.priority,
                plan_task_id = COALESCE(excluded.plan_task_id, todos.plan_task_id),
                planned_hours = COALESCE(excluded.planned_hours, todos.planned_hours),
                kind = COALESCE(excluded.kind, todos.kind)
            "#,
            params![
                &id,
                &input.session_id,
                input.course_id,
                input.date.trim(),
                input.text.trim(),
                input.priority.unwrap_or(0),
                now_iso,
                input.plan_task_id.as_deref().map(str::trim),
                input.planned_hours,
                input.kind.as_deref().map(str::trim)
            ],
        )?;

        let todos = Self::list_todos(conn, &input.session_id)?;
        todos
            .into_iter()
            .find(|t| t.id == id)
            .ok_or_else(|| DbError::NotFound("To-Do salvato non trovato".to_string()))
    }

    pub fn toggle_todo(conn: &Connection, todo_id: &str) -> DbResult<()> {
        conn.execute(
            "UPDATE todos SET done = CASE WHEN done = 0 THEN 1 ELSE 0 END WHERE id = ?1",
            params![todo_id],
        )?;
        Ok(())
    }

    pub fn delete_todo(conn: &Connection, todo_id: &str) -> DbResult<()> {
        conn.execute("DELETE FROM todos WHERE id = ?1", params![todo_id])?;
        Ok(())
    }

    pub fn list_daily_notes(conn: &Connection, session_id: &str) -> DbResult<Vec<DailyNote>> {
        let mut stmt = conn.prepare(
            r#"
            SELECT session_id, date, content_markdown, updated_at
            FROM daily_notes
            WHERE session_id = ?1
            ORDER BY date ASC
            "#,
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
    ) -> DbResult<DailyNote> {
        let now_iso = Utc::now().to_rfc3339();
        let trimmed = content_markdown.trim();
        if trimmed.is_empty() {
            conn.execute(
                "DELETE FROM daily_notes WHERE session_id = ?1 AND date = ?2",
                params![session_id, date],
            )?;
            return Ok(DailyNote {
                session_id: session_id.to_string(),
                date: date.to_string(),
                content_markdown: String::new(),
                updated_at: now_iso,
            });
        }

        conn.execute(
            r#"
            INSERT INTO daily_notes (session_id, date, content_markdown, updated_at)
            VALUES (?1, ?2, ?3, ?4)
            ON CONFLICT(session_id, date) DO UPDATE SET
                content_markdown = excluded.content_markdown,
                updated_at = excluded.updated_at
            "#,
            params![session_id, date, trimmed, &now_iso],
        )?;

        Ok(DailyNote {
            session_id: session_id.to_string(),
            date: date.to_string(),
            content_markdown: trimmed.to_string(),
            updated_at: now_iso,
        })
    }

    // ========================================================================
    // Weekly Lessons (Orario Settimanale delle Lezioni)
    // ========================================================================

    pub fn list_weekly_lessons(conn: &Connection, session_id: &str) -> DbResult<Vec<WeeklyLesson>> {
        let mut stmt = conn.prepare(
            r#"
            SELECT id, session_id, course_id, course_name, day_of_week, start_time, end_time, classroom, professor, color_hex
            FROM weekly_lessons
            WHERE session_id = ?1
            ORDER BY day_of_week ASC, start_time ASC
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
        let id = input
            .id
            .clone()
            .filter(|s| !s.trim().is_empty())
            .unwrap_or_else(|| format!("les_{}", Uuid::new_v4().simple()));
        let now_iso = Utc::now().to_rfc3339();
        let classroom = input.classroom.clone().unwrap_or_default();
        let professor = input.professor.clone().unwrap_or_default();
        let color_hex = input.color_hex.clone().unwrap_or_else(|| "#AF52DE".to_string());

        conn.execute(
            r#"
            INSERT INTO weekly_lessons (
                id, session_id, course_id, course_name, day_of_week, start_time, end_time, classroom, professor, color_hex, created_at
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)
            ON CONFLICT(id) DO UPDATE SET
                course_id = excluded.course_id,
                course_name = excluded.course_name,
                day_of_week = excluded.day_of_week,
                start_time = excluded.start_time,
                end_time = excluded.end_time,
                classroom = excluded.classroom,
                professor = excluded.professor,
                color_hex = excluded.color_hex
            "#,
            params![
                &id,
                input.session_id.trim(),
                input.course_id.as_deref().map(str::trim),
                input.course_name.trim(),
                input.day_of_week,
                input.start_time.trim(),
                input.end_time.trim(),
                classroom.trim(),
                professor.trim(),
                color_hex.trim(),
                &now_iso
            ],
        )?;

        conn.query_row(
            r#"
            SELECT id, session_id, course_id, course_name, day_of_week, start_time, end_time, classroom, professor, color_hex
            FROM weekly_lessons
            WHERE id = ?1
            "#,
            params![&id],
            |row| {
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
            },
        )
        .map_err(Into::into)
    }

    pub fn delete_weekly_lesson(conn: &Connection, lesson_id: &str) -> DbResult<()> {
        conn.execute(
            "DELETE FROM weekly_lessons WHERE id = ?1",
            params![lesson_id],
        )?;
        Ok(())
    }

    pub fn project_weekly_lessons_to_calendar(
        conn: &mut Connection,
        session_id: &str,
        start_date: &str,
        end_date: &str,
    ) -> DbResult<usize> {
        let lessons = Self::list_weekly_lessons(conn, session_id)?;
        if lessons.is_empty() {
            return Ok(0);
        }

        let start = chrono::NaiveDate::parse_from_str(start_date, "%Y-%m-%d")
            .map_err(|e| DbError::Validation(format!("Data inizio non valida: {}", e)))?;
        let end = chrono::NaiveDate::parse_from_str(end_date, "%Y-%m-%d")
            .map_err(|e| DbError::Validation(format!("Data fine non valida: {}", e)))?;

        if start > end {
            return Err(DbError::Validation("La data di inizio non puo superare la data di fine.".to_string()));
        }

        let tx = conn.transaction()?;
        let mut count = 0;
        let mut curr = start;

        while curr <= end {
            let iso_date = curr.format("%Y-%m-%d").to_string();
            use chrono::Datelike;
            let dow = curr.weekday().number_from_monday() as i32;

            for l in &lessons {
                if l.day_of_week == dow {
                    let ev_id = format!("ev_les_{}_{}", l.id, iso_date.replace('-', ""));
                    let time_slot = format!("{}-{}", l.start_time, l.end_time);
                    let mut details = String::new();
                    if !l.classroom.is_empty() {
                        details.push_str(&format!("Aula: {}", l.classroom));
                    }
                    if !l.professor.is_empty() {
                        if !details.is_empty() { details.push_str(" · "); }
                        details.push_str(&format!("Docente: {}", l.professor));
                    }

                    tx.execute(
                        r#"
                        INSERT OR IGNORE INTO generic_events (
                            id, session_id, course_id, date, time_slot, title, category, details
                        ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, 'lezione', ?7)
                        "#,
                        params![
                            &ev_id,
                            session_id,
                            l.course_id.as_deref(),
                            &iso_date,
                            &time_slot,
                            &l.course_name,
                            &details
                        ],
                    )?;
                    count += 1;
                }
            }
            match curr.succ_opt() {
                Some(next) => curr = next,
                None => break,
            }
        }

        tx.commit()?;
        Ok(count)
    }

    // ========================================================================
    // Ignored Conflicts Engine Storage
    // ========================================================================

    pub fn list_ignored_conflicts(
        conn: &Connection,
        session_id: &str,
    ) -> DbResult<HashSet<String>> {
        let mut stmt = conn.prepare(
            "SELECT conflict_key FROM ignored_conflicts WHERE session_id = ?1",
        )?;
        let rows = stmt.query_map(params![session_id], |row| row.get::<_, String>(0))?;
        let mut set = HashSet::new();
        for r in rows {
            set.insert(r?);
        }
        Ok(set)
    }

    pub fn ignore_conflict(
        conn: &Connection,
        session_id: &str,
        conflict_key: &str,
    ) -> DbResult<()> {
        let now = chrono::Utc::now().to_rfc3339();
        conn.execute(
            "INSERT OR REPLACE INTO ignored_conflicts (session_id, conflict_key, created_at) VALUES (?1, ?2, ?3)",
            params![session_id, conflict_key, &now],
        )?;
        Ok(())
    }

    pub fn restore_conflict(
        conn: &Connection,
        session_id: &str,
        conflict_key: &str,
    ) -> DbResult<()> {
        conn.execute(
            "DELETE FROM ignored_conflicts WHERE session_id = ?1 AND conflict_key = ?2",
            params![session_id, conflict_key],
        )?;
        Ok(())
    }

    pub fn ignore_all_conflicts(
        conn: &Connection,
        session_id: &str,
        conflict_keys: &[String],
    ) -> DbResult<()> {
        let now = chrono::Utc::now().to_rfc3339();
        let tx = conn.unchecked_transaction()?;
        for key in conflict_keys {
            tx.execute(
                "INSERT OR REPLACE INTO ignored_conflicts (session_id, conflict_key, created_at) VALUES (?1, ?2, ?3)",
                params![session_id, key, &now],
            )?;
        }
        tx.commit()?;
        Ok(())
    }

    pub fn restore_all_conflicts(
        conn: &Connection,
        session_id: &str,
    ) -> DbResult<()> {
        conn.execute(
            "DELETE FROM ignored_conflicts WHERE session_id = ?1",
            params![session_id],
        )?;
        Ok(())
    }

    // ========================================================================
    // Snapshot & Bootstrap Builder
    // ========================================================================

    pub fn build_session_snapshot(
        conn: &Connection,
        session_id: &str,
        settings: &AppSettings,
    ) -> DbResult<SessionSnapshot> {
        let session = Self::get_session(conn, session_id)?;
        let courses = Self::list_courses(conn, session_id)?;
        let study_units = Self::list_study_units(conn, session_id)?;
        let exams = Self::list_exams(conn, session_id)?;
        let generic_events = Self::list_generic_events(conn, session_id)?;
        let todos = Self::list_todos(conn, session_id)?;
        let daily_notes = Self::list_daily_notes(conn, session_id)?;
        let weekly_lessons = Self::list_weekly_lessons(conn, session_id)?;

        let all_conflicts = detect_session_conflicts(&courses, &study_units, &exams);
        let ignored_keys = Self::list_ignored_conflicts(conn, session_id).unwrap_or_default();
        let mut active_conflicts = Vec::new();
        let mut ignored_conflicts = Vec::new();

        for cf in all_conflicts {
            if ignored_keys.contains(&cf.id) {
                ignored_conflicts.push(cf);
            } else {
                active_conflicts.push(cf);
            }
        }

        let telemetry = compute_session_telemetry(
            &session,
            &courses,
            &study_units,
            &exams,
            &active_conflicts,
            settings.study_hours_per_cfu,
            settings.weekly_holiday_day,
            None,
        );

        Ok(SessionSnapshot {
            session,
            courses,
            study_units,
            exams,
            generic_events,
            todos,
            daily_notes,
            weekly_lessons,
            conflicts: active_conflicts,
            ignored_conflicts,
            telemetry,
        })
    }

    pub fn build_bootstrap_payload(
        &self,
        app_version: &str,
    ) -> DbResult<BootstrapPayload> {
        self.with_conn(|conn| {
            let profile = Self::get_profile(conn)?;
            let mut settings = Self::get_settings(conn)?;
            let sessions = Self::list_sessions(conn)?;

            // Resolve active session ID
            let active_id = settings
                .active_session_id
                .clone()
                .filter(|id| sessions.iter().any(|s| &s.id == id))
                .or_else(|| sessions.first().map(|s| s.id.clone()));

            if active_id != settings.active_session_id {
                settings.active_session_id = active_id.clone();
                let _ = Self::save_settings(conn, &settings);
            }

            let active_snapshot = match active_id {
                Some(ref sid) => Some(Self::build_session_snapshot(conn, sid, &settings)?),
                None => None,
            };

            Ok(BootstrapPayload {
                app_version: app_version.to_string(),
                db_path: self.db_path.to_string_lossy().to_string(),
                profile,
                settings,
                sessions,
                active_snapshot,
            })
        })
    }

    // ========================================================================
    // Backup Export & Import (.json)
    // ========================================================================

    pub fn export_workspace_backup(
        conn: &Connection,
        app_version: &str,
    ) -> DbResult<FullWorkspaceBackup> {
        let profile = Self::get_profile(conn)?;
        let settings = Self::get_settings(conn)?;
        let sessions = Self::list_sessions(conn)?;

        let mut courses = Vec::new();
        let mut study_units = Vec::new();
        let mut exams = Vec::new();
        let mut generic_events = Vec::new();
        let mut todos = Vec::new();
        let mut daily_notes = Vec::new();
        let mut weekly_lessons = Vec::new();

        for s in &sessions {
            courses.extend(Self::list_courses(conn, &s.id)?);
            study_units.extend(Self::list_study_units(conn, &s.id)?);
            exams.extend(Self::list_exams(conn, &s.id)?);
            generic_events.extend(Self::list_generic_events(conn, &s.id)?);
            todos.extend(Self::list_todos(conn, &s.id)?);
            daily_notes.extend(Self::list_daily_notes(conn, &s.id)?);
            weekly_lessons.extend(Self::list_weekly_lessons(conn, &s.id)?);
        }

        Ok(FullWorkspaceBackup {
            schema_version: migrations::CURRENT_SCHEMA_VERSION,
            app_version: app_version.to_string(),
            exported_at: Utc::now().to_rfc3339(),
            profile,
            settings,
            sessions,
            courses,
            study_units,
            exams,
            generic_events,
            todos,
            daily_notes,
            weekly_lessons,
        })
    }

    pub fn import_workspace_backup(
        conn: &mut Connection,
        backup: &FullWorkspaceBackup,
    ) -> DbResult<()> {
        let tx = conn.transaction()?;

        // Clear existing domain tables cleanly
        tx.execute("DELETE FROM academic_sessions", [])?;

        let now_iso = Utc::now().to_rfc3339();
        tx.execute(
            r#"
            UPDATE user_profile
            SET student_name = ?1,
                university = ?2,
                faculty = ?3,
                degree_course = ?4,
                enrollment_year = ?5,
                onboarding_completed = 1,
                updated_at = ?6
            WHERE id = 1
            "#,
            params![
                backup.profile.student_name,
                backup.profile.university,
                backup.profile.faculty,
                backup.profile.degree_course,
                backup.profile.enrollment_year,
                now_iso
            ],
        )?;

        for s in &backup.sessions {
            tx.execute(
                r#"
                INSERT INTO academic_sessions (
                    id, name, subtitle, session_type, academic_year, start_date, end_date, notes, created_at, plan_id, is_system
                ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)
                ON CONFLICT(id) DO UPDATE SET
                    name = excluded.name,
                    subtitle = excluded.subtitle,
                    session_type = excluded.session_type,
                    academic_year = excluded.academic_year,
                    start_date = excluded.start_date,
                    end_date = excluded.end_date,
                    notes = excluded.notes,
                    plan_id = excluded.plan_id,
                    is_system = excluded.is_system
                "#,
                params![
                    s.id,
                    s.name,
                    s.subtitle,
                    s.session_type,
                    s.academic_year,
                    s.start_date,
                    s.end_date,
                    s.notes,
                    s.created_at,
                    s.plan_id,
                    if s.is_system { 1 } else { 0 }
                ],
            )?;
        }

        tx.execute(
            r#"
            INSERT OR IGNORE INTO academic_sessions (
                id, name, subtitle, session_type, academic_year,
                start_date, end_date, notes, is_system, created_at
            ) VALUES (
                'session_generale', 'Generale', 'Spazio libero continuo', 'generale', '',
                '2020-01-01', '2040-12-31', 'Sessione permanente di default', 1, datetime('now')
            )
            "#,
            [],
        )?;

        for c in &backup.courses {
            tx.execute(
                r#"
                INSERT INTO courses (
                    id, session_id, sigla, name, code, cfu, professor,
                    color_hex, difficulty_weight, target_hours_override, notes, sort_order
                ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12)
                "#,
                params![
                    c.id,
                    c.session_id,
                    c.sigla,
                    c.name,
                    c.code,
                    c.cfu,
                    c.professor,
                    c.color_hex,
                    c.difficulty_weight,
                    c.target_hours_override,
                    c.notes,
                    c.sort_order
                ],
            )?;
        }

        for u in &backup.study_units {
            tx.execute(
                r#"
                INSERT INTO study_units (
                    id, course_id, session_id, unit_number, title, description,
                    estimated_hours, is_completed, assigned_date, completed_at
                ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)
                "#,
                params![
                    u.id,
                    u.course_id,
                    u.session_id,
                    u.unit_number,
                    u.title,
                    u.description,
                    u.estimated_hours,
                    if u.is_completed { 1 } else { 0 },
                    u.assigned_date,
                    u.completed_at
                ],
            )?;
        }

        for e in &backup.exams {
            tx.execute(
                r#"
                INSERT INTO exams (
                    id, session_id, course_id, date, time_slot, duration_minutes,
                    round, exam_type, classroom, weight_fraction, pair_id,
                    title, details, status, grade
                ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15)
                "#,
                params![
                    e.id,
                    e.session_id,
                    e.course_id,
                    e.date,
                    e.time_slot,
                    e.duration_minutes,
                    e.round,
                    e.exam_type,
                    e.classroom,
                    e.weight_fraction,
                    e.pair_id,
                    e.title,
                    e.details,
                    e.status,
                    e.grade
                ],
            )?;
        }

        for g in &backup.generic_events {
            tx.execute(
                r#"
                INSERT INTO generic_events (id, session_id, course_id, date, time_slot, title, category, details)
                VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)
                "#,
                params![
                    g.id,
                    g.session_id,
                    g.course_id,
                    g.date,
                    g.time_slot,
                    g.title,
                    g.category,
                    g.details
                ],
            )?;
        }

        for t in &backup.todos {
            tx.execute(
                r#"
                INSERT INTO todos (id, session_id, course_id, date, text, done, priority, created_at)
                VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)
                "#,
                params![
                    t.id,
                    t.session_id,
                    t.course_id,
                    t.date,
                    t.text,
                    if t.done { 1 } else { 0 },
                    t.priority,
                    t.created_at
                ],
            )?;
        }

        for n in &backup.daily_notes {
            tx.execute(
                r#"
                INSERT INTO daily_notes (session_id, date, content_markdown, updated_at)
                VALUES (?1, ?2, ?3, ?4)
                "#,
                params![n.session_id, n.date, n.content_markdown, n.updated_at],
            )?;
        }

        for l in &backup.weekly_lessons {
            tx.execute(
                r#"
                INSERT INTO weekly_lessons (
                    id, session_id, course_id, course_name, day_of_week, start_time, end_time, classroom, professor, color_hex, created_at
                ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)
                ON CONFLICT(id) DO UPDATE SET
                    course_id = excluded.course_id,
                    course_name = excluded.course_name,
                    day_of_week = excluded.day_of_week,
                    start_time = excluded.start_time,
                    end_time = excluded.end_time,
                    classroom = excluded.classroom,
                    professor = excluded.professor,
                    color_hex = excluded.color_hex
                "#,
                params![
                    l.id,
                    l.session_id,
                    l.course_id,
                    l.course_name,
                    l.day_of_week,
                    l.start_time,
                    l.end_time,
                    l.classroom,
                    l.professor,
                    l.color_hex,
                    now_iso
                ],
            )?;
        }

        tx.commit()?;
        let _ = Self::save_settings(conn, &backup.settings)?;
        Ok(())
    }
}
