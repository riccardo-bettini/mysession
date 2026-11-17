use chrono::Utc;
use rusqlite::{params, Connection, Result};
use std::fs;
use std::path::Path;

pub const CURRENT_SCHEMA_VERSION: i32 = 200;

const MIGRATION_V200_SQL: &str = r#"
CREATE TABLE IF NOT EXISTS schema_migrations (
    version INTEGER PRIMARY KEY,
    applied_at TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS app_settings (
    id INTEGER PRIMARY KEY CHECK (id = 1),
    theme TEXT NOT NULL DEFAULT 'light',
    backdrop_material TEXT NOT NULL DEFAULT 'mica',
    ui_scale INTEGER NOT NULL DEFAULT 100,
    accent_color TEXT NOT NULL DEFAULT '#7C5CFC',
    cat_teoria_color TEXT NOT NULL DEFAULT '#3B82F6',
    cat_esercizi_color TEXT NOT NULL DEFAULT '#10B981',
    cat_temi_esame_color TEXT NOT NULL DEFAULT '#F59E0B',
    cat_dimostrazioni_color TEXT NOT NULL DEFAULT '#8B5CF6',
    cat_domande_orale_color TEXT NOT NULL DEFAULT '#EC4899',
    sidebar_sections_json TEXT NOT NULL DEFAULT '["home","calendar","today","courses","quiz","progress","ai"]',
    hidden_sections_json TEXT NOT NULL DEFAULT '[]',
    plan_folder TEXT NOT NULL DEFAULT '',
    active_session_id TEXT,
    study_hours_per_cfu REAL NOT NULL DEFAULT 15.0,
    start_day_of_week INTEGER NOT NULL DEFAULT 1,
    ai_provider TEXT NOT NULL DEFAULT 'local',
    ai_local_model_id TEXT NOT NULL DEFAULT 'qwen2.5-vl-3b',
    ai_custom_cli_command TEXT NOT NULL DEFAULT '',
    ai_idle_timeout_minutes INTEGER NOT NULL DEFAULT 5
);

CREATE TABLE IF NOT EXISTS academic_sessions (
    id TEXT PRIMARY KEY,
    plan_id TEXT,
    name TEXT NOT NULL,
    subtitle TEXT NOT NULL DEFAULT '',
    session_type TEXT NOT NULL DEFAULT 'invernale',
    academic_year TEXT NOT NULL DEFAULT '',
    start_date TEXT NOT NULL,
    end_date TEXT NOT NULL,
    max_hours_per_day REAL NOT NULL DEFAULT 6.0,
    study_weekdays_json TEXT NOT NULL DEFAULT '[1,2,3,4,5]',
    off_dates_json TEXT NOT NULL DEFAULT '[]',
    review_days_before_exam INTEGER NOT NULL DEFAULT 5,
    notes TEXT NOT NULL DEFAULT '',
    created_at TEXT NOT NULL
);
CREATE INDEX IF NOT EXISTS idx_sessions_plan_id ON academic_sessions(plan_id);

CREATE TABLE IF NOT EXISTS courses (
    id TEXT PRIMARY KEY,
    session_id TEXT NOT NULL,
    plan_course_id TEXT,
    sigla TEXT NOT NULL,
    name TEXT NOT NULL,
    code TEXT NOT NULL DEFAULT '',
    cfu REAL NOT NULL DEFAULT 6.0,
    professor TEXT NOT NULL DEFAULT '',
    color_hex TEXT NOT NULL DEFAULT '#7C5CFC',
    difficulty_weight REAL NOT NULL DEFAULT 1.0,
    target_hours REAL,
    is_active INTEGER NOT NULL DEFAULT 1,
    notes TEXT NOT NULL DEFAULT '',
    sort_order INTEGER NOT NULL DEFAULT 0,
    FOREIGN KEY (session_id) REFERENCES academic_sessions(id) ON DELETE CASCADE
);
CREATE INDEX IF NOT EXISTS idx_courses_session ON courses(session_id);

CREATE TABLE IF NOT EXISTS study_units (
    id TEXT PRIMARY KEY,
    course_id TEXT NOT NULL,
    session_id TEXT NOT NULL,
    plan_unit_id TEXT,
    unit_number INTEGER NOT NULL DEFAULT 1,
    chapter TEXT NOT NULL DEFAULT '',
    title TEXT NOT NULL,
    description TEXT NOT NULL DEFAULT '',
    source TEXT NOT NULL DEFAULT '',
    estimated_hours REAL NOT NULL DEFAULT 2.0,
    priority INTEGER NOT NULL DEFAULT 2,
    is_completed INTEGER NOT NULL DEFAULT 0,
    completed_at TEXT,
    FOREIGN KEY (course_id) REFERENCES courses(id) ON DELETE CASCADE,
    FOREIGN KEY (session_id) REFERENCES academic_sessions(id) ON DELETE CASCADE
);
CREATE INDEX IF NOT EXISTS idx_study_units_course ON study_units(course_id);
CREATE INDEX IF NOT EXISTS idx_study_units_session ON study_units(session_id);

CREATE TABLE IF NOT EXISTS exams (
    id TEXT PRIMARY KEY,
    session_id TEXT NOT NULL,
    course_id TEXT NOT NULL,
    plan_exam_id TEXT,
    date TEXT NOT NULL,
    time_slot TEXT NOT NULL DEFAULT '',
    duration_minutes INTEGER NOT NULL DEFAULT 120,
    round TEXT NOT NULL DEFAULT '1',
    exam_type TEXT NOT NULL DEFAULT 'scritto_orale',
    classroom TEXT NOT NULL DEFAULT '',
    title TEXT NOT NULL,
    details TEXT NOT NULL DEFAULT '',
    status TEXT NOT NULL DEFAULT 'planned',
    grade TEXT,
    FOREIGN KEY (session_id) REFERENCES academic_sessions(id) ON DELETE CASCADE,
    FOREIGN KEY (course_id) REFERENCES courses(id) ON DELETE CASCADE
);
CREATE INDEX IF NOT EXISTS idx_exams_session_date ON exams(session_id, date);
CREATE INDEX IF NOT EXISTS idx_exams_course ON exams(course_id);

CREATE TABLE IF NOT EXISTS daily_operational_plans (
    session_id TEXT NOT NULL,
    date TEXT NOT NULL,
    daily_goal TEXT NOT NULL DEFAULT '',
    strategy_note TEXT NOT NULL DEFAULT '',
    PRIMARY KEY (session_id, date),
    FOREIGN KEY (session_id) REFERENCES academic_sessions(id) ON DELETE CASCADE
);

CREATE TABLE IF NOT EXISTS operational_blocks (
    id TEXT PRIMARY KEY,
    session_id TEXT NOT NULL,
    date TEXT NOT NULL,
    course_id TEXT,
    block_order INTEGER NOT NULL DEFAULT 1,
    category TEXT NOT NULL DEFAULT 'teoria',
    title TEXT NOT NULL,
    chapter TEXT NOT NULL DEFAULT '',
    topic TEXT NOT NULL DEFAULT '',
    pages TEXT NOT NULL DEFAULT '',
    source TEXT NOT NULL DEFAULT '',
    duration_hours REAL NOT NULL DEFAULT 1.0,
    instructions TEXT NOT NULL DEFAULT '',
    done INTEGER NOT NULL DEFAULT 0,
    plan_block_id TEXT,
    FOREIGN KEY (session_id) REFERENCES academic_sessions(id) ON DELETE CASCADE
);
CREATE INDEX IF NOT EXISTS idx_op_blocks_session_date ON operational_blocks(session_id, date);

CREATE TABLE IF NOT EXISTS tasks (
    id TEXT PRIMARY KEY,
    session_id TEXT NOT NULL,
    course_id TEXT,
    unit_id TEXT,
    date TEXT NOT NULL,
    category TEXT NOT NULL DEFAULT 'teoria',
    title TEXT NOT NULL,
    chapter TEXT NOT NULL DEFAULT '',
    topic TEXT NOT NULL DEFAULT '',
    pages TEXT NOT NULL DEFAULT '',
    source TEXT NOT NULL DEFAULT '',
    planned_hours REAL NOT NULL DEFAULT 1.0,
    done INTEGER NOT NULL DEFAULT 0,
    priority INTEGER NOT NULL DEFAULT 2,
    plan_task_id TEXT,
    created_at TEXT NOT NULL,
    FOREIGN KEY (session_id) REFERENCES academic_sessions(id) ON DELETE CASCADE
);
CREATE INDEX IF NOT EXISTS idx_tasks_session_date ON tasks(session_id, date);
CREATE INDEX IF NOT EXISTS idx_tasks_course ON tasks(course_id);
CREATE UNIQUE INDEX IF NOT EXISTS idx_tasks_plan_task ON tasks(session_id, plan_task_id) WHERE plan_task_id IS NOT NULL;

CREATE TABLE IF NOT EXISTS task_subitems (
    id TEXT PRIMARY KEY,
    task_id TEXT NOT NULL,
    text TEXT NOT NULL,
    category TEXT NOT NULL DEFAULT '',
    pages TEXT NOT NULL DEFAULT '',
    done INTEGER NOT NULL DEFAULT 0,
    sort_order INTEGER NOT NULL DEFAULT 0,
    plan_subtask_id TEXT,
    FOREIGN KEY (task_id) REFERENCES tasks(id) ON DELETE CASCADE
);
CREATE INDEX IF NOT EXISTS idx_task_subitems_task ON task_subitems(task_id);

CREATE TABLE IF NOT EXISTS daily_quizzes (
    id TEXT PRIMARY KEY,
    session_id TEXT NOT NULL,
    course_id TEXT NOT NULL,
    date TEXT NOT NULL,
    title TEXT NOT NULL,
    topic_summary TEXT NOT NULL DEFAULT '',
    plan_quiz_id TEXT,
    FOREIGN KEY (session_id) REFERENCES academic_sessions(id) ON DELETE CASCADE,
    FOREIGN KEY (course_id) REFERENCES courses(id) ON DELETE CASCADE
);
CREATE INDEX IF NOT EXISTS idx_quizzes_session_date ON daily_quizzes(session_id, date);
CREATE UNIQUE INDEX IF NOT EXISTS idx_quizzes_plan_quiz ON daily_quizzes(session_id, plan_quiz_id) WHERE plan_quiz_id IS NOT NULL;

CREATE TABLE IF NOT EXISTS quiz_questions (
    id TEXT PRIMARY KEY,
    quiz_id TEXT NOT NULL,
    question_order INTEGER NOT NULL DEFAULT 1,
    question_type TEXT NOT NULL DEFAULT 'multiple_choice',
    category TEXT NOT NULL DEFAULT 'teoria',
    chapter TEXT NOT NULL DEFAULT '',
    topic TEXT NOT NULL DEFAULT '',
    prompt TEXT NOT NULL,
    options_json TEXT NOT NULL DEFAULT '[]',
    correct_index INTEGER,
    explanation TEXT NOT NULL DEFAULT '',
    plan_question_id TEXT,
    FOREIGN KEY (quiz_id) REFERENCES daily_quizzes(id) ON DELETE CASCADE
);
CREATE INDEX IF NOT EXISTS idx_quiz_questions_quiz ON quiz_questions(quiz_id);

CREATE TABLE IF NOT EXISTS quiz_attempts (
    id TEXT PRIMARY KEY,
    quiz_id TEXT NOT NULL,
    session_id TEXT NOT NULL,
    course_id TEXT NOT NULL,
    score_pct REAL NOT NULL,
    correct_count INTEGER NOT NULL,
    total_count INTEGER NOT NULL,
    answers_json TEXT NOT NULL DEFAULT '[]',
    wrong_question_ids_json TEXT NOT NULL DEFAULT '[]',
    completed_at TEXT NOT NULL,
    FOREIGN KEY (quiz_id) REFERENCES daily_quizzes(id) ON DELETE CASCADE
);
CREATE INDEX IF NOT EXISTS idx_quiz_attempts_quiz ON quiz_attempts(quiz_id);
CREATE INDEX IF NOT EXISTS idx_quiz_attempts_session ON quiz_attempts(session_id);

CREATE TABLE IF NOT EXISTS weekly_lessons (
    id TEXT PRIMARY KEY,
    session_id TEXT NOT NULL,
    course_id TEXT,
    course_name TEXT NOT NULL,
    day_of_week INTEGER NOT NULL CHECK (day_of_week BETWEEN 1 AND 7),
    start_time TEXT NOT NULL,
    end_time TEXT NOT NULL,
    classroom TEXT NOT NULL DEFAULT '',
    professor TEXT NOT NULL DEFAULT '',
    color_hex TEXT NOT NULL DEFAULT '#7C5CFC',
    FOREIGN KEY (session_id) REFERENCES academic_sessions(id) ON DELETE CASCADE
);
CREATE INDEX IF NOT EXISTS idx_weekly_lessons_session ON weekly_lessons(session_id);

CREATE TABLE IF NOT EXISTS calendar_events (
    id TEXT PRIMARY KEY,
    session_id TEXT NOT NULL,
    course_id TEXT,
    date TEXT NOT NULL,
    time_slot TEXT NOT NULL DEFAULT '',
    title TEXT NOT NULL,
    category TEXT NOT NULL DEFAULT 'personale',
    details TEXT NOT NULL DEFAULT '',
    FOREIGN KEY (session_id) REFERENCES academic_sessions(id) ON DELETE CASCADE
);
CREATE INDEX IF NOT EXISTS idx_calendar_events_session_date ON calendar_events(session_id, date);

CREATE TABLE IF NOT EXISTS daily_notes (
    session_id TEXT NOT NULL,
    date TEXT NOT NULL,
    content_markdown TEXT NOT NULL DEFAULT '',
    updated_at TEXT NOT NULL,
    PRIMARY KEY (session_id, date),
    FOREIGN KEY (session_id) REFERENCES academic_sessions(id) ON DELETE CASCADE
);

CREATE TABLE IF NOT EXISTS plan_state (
    plan_id TEXT PRIMARY KEY,
    session_id TEXT NOT NULL,
    applied_revision INTEGER NOT NULL DEFAULT 0,
    base_json TEXT NOT NULL,
    pending_revision INTEGER,
    source_path TEXT NOT NULL DEFAULT '',
    updated_at TEXT NOT NULL,
    FOREIGN KEY (session_id) REFERENCES academic_sessions(id) ON DELETE CASCADE
);
CREATE INDEX IF NOT EXISTS idx_plan_state_session ON plan_state(session_id);

CREATE TABLE IF NOT EXISTS ai_undo_snapshots (
    id TEXT PRIMARY KEY,
    session_id TEXT NOT NULL,
    description TEXT NOT NULL,
    snapshot_json TEXT NOT NULL,
    created_at TEXT NOT NULL
);
CREATE INDEX IF NOT EXISTS idx_ai_undo_session ON ai_undo_snapshots(session_id);
"#;

/// Se esiste un database v1.x (versione schema < 200), esegue un backup in `database_v1.bak`
/// e rimuove i file SQLite correnti per consentire una creazione v2.00 pulita.
pub fn backup_and_reset_legacy_v1_if_needed(db_path: &Path) {
    if !db_path.exists() {
        return;
    }

    let is_legacy = {
        match Connection::open(db_path) {
            Ok(conn) => {
                let ver: i32 = conn
                    .query_row(
                        "SELECT COALESCE(MAX(version), 0) FROM schema_migrations",
                        [],
                        |row| row.get(0),
                    )
                    .unwrap_or(0);
                ver < CURRENT_SCHEMA_VERSION
            }
            Err(_) => true,
        }
    };

    if is_legacy {
        let bak_path = db_path.with_file_name("database_v1.bak");
        let _ = fs::copy(db_path, &bak_path);
        let _ = fs::remove_file(db_path);
        let wal = db_path.with_file_name("database.sqlite-wal");
        let shm = db_path.with_file_name("database.sqlite-shm");
        if wal.exists() {
            let _ = fs::remove_file(wal);
        }
        if shm.exists() {
            let _ = fs::remove_file(shm);
        }
    }
}

pub fn run_migrations(conn: &Connection) -> Result<()> {
    conn.execute_batch(
        r#"
        PRAGMA foreign_keys = ON;
        CREATE TABLE IF NOT EXISTS schema_migrations (
            version INTEGER PRIMARY KEY,
            applied_at TEXT NOT NULL
        );
        "#,
    )?;

    let current_version: i32 = conn.query_row(
        "SELECT COALESCE(MAX(version), 0) FROM schema_migrations",
        [],
        |row| row.get(0),
    )?;

    if current_version < 200 {
        conn.execute_batch(MIGRATION_V200_SQL)?;
        conn.execute(
            "INSERT OR IGNORE INTO schema_migrations (version, applied_at) VALUES (?1, ?2)",
            params![200, Utc::now().to_rfc3339()],
        )?;
        conn.execute(
            "INSERT OR IGNORE INTO app_settings (id) VALUES (1)",
            [],
        )?;
    }

    Ok(())
}
