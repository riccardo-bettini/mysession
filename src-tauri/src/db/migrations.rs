use chrono::Utc;
use rusqlite::{params, Connection, Result};

pub const CURRENT_SCHEMA_VERSION: i32 = 2;

const MIGRATION_V1_SQL: &str = r#"
CREATE TABLE IF NOT EXISTS schema_migrations (
    version INTEGER PRIMARY KEY,
    applied_at TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS user_profile (
    id INTEGER PRIMARY KEY CHECK (id = 1),
    student_name TEXT NOT NULL DEFAULT '',
    university TEXT NOT NULL DEFAULT '',
    faculty TEXT NOT NULL DEFAULT '',
    degree_course TEXT NOT NULL DEFAULT '',
    enrollment_year INTEGER,
    onboarding_completed INTEGER NOT NULL DEFAULT 0,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS app_settings (
    id INTEGER PRIMARY KEY CHECK (id = 1),
    theme TEXT NOT NULL DEFAULT 'auto',
    backdrop_material TEXT NOT NULL DEFAULT 'mica',
    window_controls_style TEXT NOT NULL DEFAULT 'win11-right',
    window_frame_style TEXT NOT NULL DEFAULT 'custom',
    ui_scale INTEGER NOT NULL DEFAULT 100,
    sidebar_state TEXT NOT NULL DEFAULT 'expanded',
    default_view TEXT NOT NULL DEFAULT 'continuous',
    start_day_of_week INTEGER NOT NULL DEFAULT 1,
    weekly_holiday_day INTEGER NOT NULL DEFAULT 0,
    study_hours_per_cfu REAL NOT NULL DEFAULT 15.0,
    accent_color TEXT NOT NULL DEFAULT '#007AFF',
    holiday_color TEXT NOT NULL DEFAULT '#FF3B30',
    sunday_color TEXT NOT NULL DEFAULT '#FF9500',
    cat_studio_color TEXT NOT NULL DEFAULT '#007AFF',
    cat_scadenza_color TEXT NOT NULL DEFAULT '#FF9500',
    cat_lezione_color TEXT NOT NULL DEFAULT '#AF52DE',
    cat_personale_color TEXT NOT NULL DEFAULT '#34C759',
    ai_provider TEXT NOT NULL DEFAULT 'local',
    ai_api_key TEXT NOT NULL DEFAULT '',
    ai_endpoint_url TEXT NOT NULL DEFAULT 'http://127.0.0.1:11434',
    ai_model TEXT NOT NULL DEFAULT 'gemini-2.5-flash',
    ai_bar_width INTEGER NOT NULL DEFAULT 760,
    active_session_id TEXT
);

CREATE TABLE IF NOT EXISTS academic_sessions (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    subtitle TEXT NOT NULL DEFAULT '',
    session_type TEXT NOT NULL DEFAULT 'invernale',
    academic_year TEXT NOT NULL DEFAULT '',
    start_date TEXT NOT NULL,
    end_date TEXT NOT NULL,
    notes TEXT NOT NULL DEFAULT '',
    created_at TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS courses (
    id TEXT PRIMARY KEY,
    session_id TEXT NOT NULL,
    sigla TEXT NOT NULL,
    name TEXT NOT NULL,
    code TEXT NOT NULL DEFAULT '',
    cfu INTEGER NOT NULL DEFAULT 6,
    professor TEXT NOT NULL DEFAULT '',
    color_hex TEXT NOT NULL DEFAULT '#007AFF',
    difficulty_weight REAL NOT NULL DEFAULT 1.0,
    target_hours_override REAL,
    notes TEXT NOT NULL DEFAULT '',
    sort_order INTEGER NOT NULL DEFAULT 0,
    FOREIGN KEY (session_id) REFERENCES academic_sessions(id) ON DELETE CASCADE
);

CREATE INDEX IF NOT EXISTS idx_courses_session ON courses(session_id);

CREATE TABLE IF NOT EXISTS study_units (
    id TEXT PRIMARY KEY,
    course_id TEXT NOT NULL,
    session_id TEXT NOT NULL,
    unit_number INTEGER NOT NULL DEFAULT 1,
    title TEXT NOT NULL,
    description TEXT NOT NULL DEFAULT '',
    estimated_hours REAL NOT NULL DEFAULT 4.0,
    is_completed INTEGER NOT NULL DEFAULT 0,
    assigned_date TEXT,
    completed_at TEXT,
    FOREIGN KEY (course_id) REFERENCES courses(id) ON DELETE CASCADE,
    FOREIGN KEY (session_id) REFERENCES academic_sessions(id) ON DELETE CASCADE
);

CREATE INDEX IF NOT EXISTS idx_study_units_course ON study_units(course_id);
CREATE INDEX IF NOT EXISTS idx_study_units_session_date ON study_units(session_id, assigned_date);

CREATE TABLE IF NOT EXISTS exams (
    id TEXT PRIMARY KEY,
    session_id TEXT NOT NULL,
    course_id TEXT NOT NULL,
    date TEXT NOT NULL,
    time_slot TEXT NOT NULL DEFAULT '',
    duration_minutes INTEGER NOT NULL DEFAULT 120,
    round TEXT NOT NULL DEFAULT '1',
    exam_type TEXT NOT NULL DEFAULT 'scritto_orale',
    classroom TEXT NOT NULL DEFAULT '',
    weight_fraction REAL NOT NULL DEFAULT 1.0,
    pair_id TEXT,
    title TEXT NOT NULL,
    details TEXT NOT NULL DEFAULT '',
    status TEXT NOT NULL DEFAULT 'planned',
    grade TEXT,
    FOREIGN KEY (session_id) REFERENCES academic_sessions(id) ON DELETE CASCADE,
    FOREIGN KEY (course_id) REFERENCES courses(id) ON DELETE CASCADE
);

CREATE INDEX IF NOT EXISTS idx_exams_session_date ON exams(session_id, date);
CREATE INDEX IF NOT EXISTS idx_exams_course ON exams(course_id);

CREATE TABLE IF NOT EXISTS generic_events (
    id TEXT PRIMARY KEY,
    session_id TEXT NOT NULL,
    course_id TEXT,
    date TEXT NOT NULL,
    time_slot TEXT NOT NULL DEFAULT '',
    title TEXT NOT NULL,
    category TEXT NOT NULL DEFAULT 'studio',
    details TEXT NOT NULL DEFAULT '',
    FOREIGN KEY (session_id) REFERENCES academic_sessions(id) ON DELETE CASCADE
);

CREATE INDEX IF NOT EXISTS idx_generic_events_session_date ON generic_events(session_id, date);

CREATE TABLE IF NOT EXISTS todos (
    id TEXT PRIMARY KEY,
    session_id TEXT NOT NULL,
    course_id TEXT,
    date TEXT NOT NULL,
    text TEXT NOT NULL,
    done INTEGER NOT NULL DEFAULT 0,
    priority INTEGER NOT NULL DEFAULT 0,
    created_at TEXT NOT NULL,
    FOREIGN KEY (session_id) REFERENCES academic_sessions(id) ON DELETE CASCADE
);

CREATE INDEX IF NOT EXISTS idx_todos_session_date ON todos(session_id, date);

CREATE TABLE IF NOT EXISTS daily_notes (
    session_id TEXT NOT NULL,
    date TEXT NOT NULL,
    content_markdown TEXT NOT NULL DEFAULT '',
    updated_at TEXT NOT NULL,
    PRIMARY KEY (session_id, date),
    FOREIGN KEY (session_id) REFERENCES academic_sessions(id) ON DELETE CASCADE
);
"#;

const MIGRATION_V2_SQL: &str = r#"
ALTER TABLE academic_sessions ADD COLUMN plan_id TEXT;
ALTER TABLE courses ADD COLUMN plan_course_id TEXT;
ALTER TABLE courses ADD COLUMN is_visible INTEGER NOT NULL DEFAULT 1;
ALTER TABLE study_units ADD COLUMN plan_unit_id TEXT;
ALTER TABLE todos ADD COLUMN plan_task_id TEXT;
ALTER TABLE todos ADD COLUMN planned_hours REAL;
ALTER TABLE todos ADD COLUMN kind TEXT;
ALTER TABLE app_settings ADD COLUMN plan_folder TEXT NOT NULL DEFAULT '';

CREATE UNIQUE INDEX IF NOT EXISTS idx_todos_plan_task ON todos(session_id, plan_task_id) WHERE plan_task_id IS NOT NULL;
CREATE INDEX IF NOT EXISTS idx_academic_sessions_plan_id ON academic_sessions(plan_id);
CREATE INDEX IF NOT EXISTS idx_courses_plan_course ON courses(plan_course_id);
CREATE INDEX IF NOT EXISTS idx_study_units_plan_unit ON study_units(plan_unit_id);

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
"#;

pub fn configure_connection_pragmas(conn: &Connection) -> Result<()> {
    conn.execute_batch(
        r#"
        PRAGMA journal_mode = WAL;
        PRAGMA synchronous = NORMAL;
        PRAGMA foreign_keys = ON;
        PRAGMA temp_store = MEMORY;
        PRAGMA busy_timeout = 5000;
        "#,
    )?;
    Ok(())
}

pub fn run_migrations(conn: &mut Connection) -> Result<()> {
    configure_connection_pragmas(conn)?;

    // Crea la tabella delle migrazioni se assente
    conn.execute(
        r#"
        CREATE TABLE IF NOT EXISTS schema_migrations (
            version INTEGER PRIMARY KEY,
            applied_at TEXT NOT NULL
        );
        "#,
        [],
    )?;

    let current_version: i32 = conn
        .query_row(
            "SELECT COALESCE(MAX(version), 0) FROM schema_migrations",
            [],
            |row| row.get(0),
        )
        .unwrap_or(0);

    let now_iso = Utc::now().to_rfc3339();

    if current_version < 1 {
        let tx = conn.transaction()?;
        tx.execute_batch(MIGRATION_V1_SQL)?;
        tx.execute(
            "INSERT OR IGNORE INTO schema_migrations (version, applied_at) VALUES (?1, ?2)",
            params![1, &now_iso],
        )?;
        tx.commit()?;
    }

    if current_version < 2 {
        let tx = conn.transaction()?;
        tx.execute_batch(MIGRATION_V2_SQL)?;
        tx.execute(
            "INSERT OR REPLACE INTO schema_migrations (version, applied_at) VALUES (?1, ?2)",
            params![2, &now_iso],
        )?;
        tx.commit()?;
    }

    // Assicura il singleton in user_profile (id = 1) e app_settings (id = 1)
    conn.execute(
        r#"
        INSERT OR IGNORE INTO user_profile (
            id, student_name, university, faculty, degree_course,
            enrollment_year, onboarding_completed, created_at, updated_at
        ) VALUES (1, '', '', '', '', NULL, 0, ?1, ?1)
        "#,
        params![&now_iso],
    )?;

    conn.execute(
        "INSERT OR IGNORE INTO app_settings (id) VALUES (1)",
        [],
    )?;

    Ok(())
}
