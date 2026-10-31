use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UserProfile {
    pub id: i64,
    pub student_name: String,
    pub university: String,
    pub faculty: String,
    pub degree_course: String,
    pub enrollment_year: Option<i32>,
    pub onboarding_completed: bool,
    pub created_at: String,
    pub updated_at: String,
}

impl Default for UserProfile {
    fn default() -> Self {
        Self {
            id: 1,
            student_name: String::new(),
            university: String::new(),
            faculty: String::new(),
            degree_course: String::new(),
            enrollment_year: None,
            onboarding_completed: false,
            created_at: String::new(),
            updated_at: String::new(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AppSettings {
    pub theme: String,                 // "light" | "dark" | "auto"
    pub backdrop_material: String,     // "mica" | "acrylic" | "tabbed"
    pub window_controls_style: String, // "macos-left" | "win11-right"
    pub window_frame_style: String,    // "custom" | "standard"
    pub ui_scale: i32,                 // 80..=150
    pub sidebar_state: String,         // "expanded" | "compact"
    pub default_view: String,          // "continuous" | "month" | "week" | "agenda"
    pub start_day_of_week: i32,        // 1 = Monday, 0 = Sunday, 6 = Saturday
    pub weekly_holiday_day: i32,       // 0 = Sunday, -1 = None
    pub study_hours_per_cfu: f64,      // Default 15.0 autonomous hours per CFU
    pub accent_color: String,          // Default "#007AFF"
    pub holiday_color: String,         // Default "#FF3B30"
    pub sunday_color: String,          // Default "#FF9500"
    pub cat_studio_color: String,      // Default "#007AFF"
    pub cat_scadenza_color: String,    // Default "#FF9500"
    pub cat_lezione_color: String,     // Default "#AF52DE"
    pub cat_personale_color: String,   // Default "#34C759"
    pub ai_provider: String,           // "local" | "gemini" | "openai" | "ollama"
    pub ai_api_key: String,            // Stored locally in SQLite (%APPDATA%), never hardcoded
    pub ai_endpoint_url: String,       // Default "http://127.0.0.1:11434"
    pub ai_model: String,              // Default "gemini-3.8-flash"
    pub ai_bar_width: i32,             // Default 760
    pub ai_local_model_id: String,     // Default "qwen2.5-vl-3b"
    pub ai_idle_timeout_minutes: i32,  // Default 5 (0 = mai spegnere)
    pub ai_boot_strategy: String,      // "on_demand" | "on_app_start" | "contextual"
    pub ai_hardware_preference: String,// "auto" | "gpu" | "cpu"
    pub active_session_id: Option<String>,
    pub plan_folder: String,
}

fn default_true() -> bool {
    true
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            theme: "auto".to_string(),
            backdrop_material: "mica".to_string(),
            window_controls_style: "win11-right".to_string(),
            window_frame_style: "custom".to_string(),
            ui_scale: 100,
            sidebar_state: "expanded".to_string(),
            default_view: "continuous".to_string(),
            start_day_of_week: 1,
            weekly_holiday_day: 0,
            study_hours_per_cfu: 15.0,
            accent_color: "#007AFF".to_string(),
            holiday_color: "#FF3B30".to_string(),
            sunday_color: "#FF9500".to_string(),
            cat_studio_color: "#007AFF".to_string(),
            cat_scadenza_color: "#FF9500".to_string(),
            cat_lezione_color: "#AF52DE".to_string(),
            cat_personale_color: "#34C759".to_string(),
            ai_provider: "local".to_string(),
            ai_api_key: String::new(),
            ai_endpoint_url: "http://127.0.0.1:11434".to_string(),
            ai_model: "gemini-3.8-flash".to_string(),
            ai_bar_width: 760,
            ai_local_model_id: "qwen2.5-vl-3b".to_string(),
            ai_idle_timeout_minutes: 5,
            ai_boot_strategy: "on_demand".to_string(),
            ai_hardware_preference: "auto".to_string(),
            active_session_id: None,
            plan_folder: String::new(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AcademicSession {
    pub id: String,
    pub name: String,
    pub subtitle: String,
    pub session_type: String, // "invernale" | "estiva" | "autunnale" | "custom"
    pub academic_year: String,
    pub start_date: String,   // YYYY-MM-DD
    pub end_date: String,     // YYYY-MM-DD
    pub notes: String,
    pub created_at: String,
    pub plan_id: Option<String>,
    #[serde(default)]
    pub is_system: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WeeklyLesson {
    pub id: String,
    pub session_id: String,
    pub course_id: Option<String>,
    pub course_name: String,
    pub day_of_week: i32, // 1 = Lunedi .. 7 = Domenica
    pub start_time: String, // HH:MM
    pub end_time: String,   // HH:MM
    pub classroom: String,
    pub professor: String,
    pub color_hex: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Course {
    pub id: String,
    pub session_id: String,
    pub sigla: String,
    pub name: String,
    pub code: String,
    pub cfu: i32,
    pub professor: String,
    pub color_hex: String,
    pub difficulty_weight: f64, // 0.5..=2.0 (default 1.0)
    pub target_hours_override: Option<f64>,
    pub notes: String,
    pub sort_order: i32,
    pub plan_course_id: Option<String>,
    #[serde(default = "default_true")]
    pub is_visible: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StudyUnit {
    pub id: String,
    pub course_id: String,
    pub session_id: String,
    pub unit_number: i32,
    pub title: String,
    pub description: String,
    pub estimated_hours: f64,
    pub is_completed: bool,
    pub assigned_date: Option<String>, // YYYY-MM-DD
    pub completed_at: Option<String>,
    pub plan_unit_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Exam {
    pub id: String,
    pub session_id: String,
    pub course_id: String,
    pub date: String,      // YYYY-MM-DD
    pub time_slot: String, // HH:MM or ""
    pub duration_minutes: i32,
    pub round: String,     // "1" | "2" | "3" | "itinere_1" | "itinere_2" | "extra"
    pub exam_type: String, // "scritto_orale" | "scritto" | "orale" | "itinere" | "progetto"
    pub classroom: String,
    pub weight_fraction: f64,
    pub pair_id: Option<String>,
    pub title: String,
    pub details: String,
    pub status: String, // "planned" | "passed" | "failed" | "withdrawn"
    pub grade: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GenericEvent {
    pub id: String,
    pub session_id: String,
    pub course_id: Option<String>,
    pub date: String, // YYYY-MM-DD
    pub time_slot: String,
    pub title: String,
    pub category: String, // "studio" | "scadenza" | "lezione" | "personale"
    pub details: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TodoItem {
    pub id: String,
    pub session_id: String,
    pub course_id: Option<String>,
    pub date: String, // YYYY-MM-DD
    pub text: String,
    pub done: bool,
    pub priority: i32, // 0 = normal, 1 = high
    pub created_at: String,
    pub plan_task_id: Option<String>,
    pub planned_hours: Option<f64>,
    pub kind: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PlanState {
    pub plan_id: String,
    pub session_id: String,
    pub applied_revision: i64,
    pub base_json: String,
    pub pending_revision: Option<i64>,
    pub source_path: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DailyNote {
    pub session_id: String,
    pub date: String, // YYYY-MM-DD
    pub content_markdown: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HolidayInfo {
    pub date: String, // YYYY-MM-DD
    pub name: String,
    pub is_national: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ConflictItem {
    pub id: String,
    pub kind: String,     // "direct_date_overlap" | "direct_time_overlap" | "insufficient_cfu_buffer"
    pub severity: String, // "critical" | "warning"
    pub date: String,
    pub secondary_date: Option<String>,
    pub exam_ids: Vec<String>,
    pub course_siglas: Vec<String>,
    pub days_gap: i32,
    pub recommended_min_days: i32,
    pub title: String,
    pub description: String,
    pub suggestion: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CourseWorkloadMetric {
    pub course_id: String,
    pub sigla: String,
    pub name: String,
    pub color_hex: String,
    pub cfu: i32,
    pub total_units: usize,
    pub completed_units: usize,
    pub completion_pct: f64,
    pub remaining_study_hours: f64,
    pub next_exam_date: Option<String>,
    pub days_until_next_exam: Option<i64>,
    pub recommended_daily_hours: f64,
    pub criticality_score: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HorizonPin {
    pub exam_id: String,
    pub course_id: String,
    pub sigla: String,
    pub color_hex: String,
    pub date: String,
    pub round: String,
    pub title: String,
    pub position_pct: f64,
    pub has_conflict: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HorizonTick {
    pub year: i32,
    pub month: u32, // 1..=12
    pub label: String,
    pub position_pct: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TelemetrySummary {
    pub reference_today: String,
    pub next_target_exam: Option<Exam>,
    pub next_target_course: Option<Course>,
    pub days_to_next_target: Option<i64>,
    pub courses_count: usize,
    pub exams_count: usize,
    pub total_cfu: i32,
    pub completed_units_total: usize,
    pub units_total: usize,
    pub overall_progress_pct: f64,
    pub most_critical_course_sigla: Option<String>,
    pub recommended_hours_today: f64,
    pub course_workloads: Vec<CourseWorkloadMetric>,
    pub daily_workload_map: HashMap<String, f64>,
    pub horizon_start_date: String,
    pub horizon_end_date: String,
    pub horizon_today_pct: Option<f64>,
    pub horizon_pins: Vec<HorizonPin>,
    pub horizon_ticks: Vec<HorizonTick>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SessionSnapshot {
    pub session: AcademicSession,
    pub courses: Vec<Course>,
    pub study_units: Vec<StudyUnit>,
    pub exams: Vec<Exam>,
    pub generic_events: Vec<GenericEvent>,
    pub todos: Vec<TodoItem>,
    pub daily_notes: Vec<DailyNote>,
    pub weekly_lessons: Vec<WeeklyLesson>,
    pub conflicts: Vec<ConflictItem>,
    pub telemetry: TelemetrySummary,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BootstrapPayload {
    pub app_version: String,
    pub db_path: String,
    pub profile: UserProfile,
    pub settings: AppSettings,
    pub sessions: Vec<AcademicSession>,
    pub active_snapshot: Option<SessionSnapshot>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FullWorkspaceBackup {
    pub schema_version: i32,
    pub app_version: String,
    pub exported_at: String,
    pub profile: UserProfile,
    pub settings: AppSettings,
    pub sessions: Vec<AcademicSession>,
    pub courses: Vec<Course>,
    pub study_units: Vec<StudyUnit>,
    pub exams: Vec<Exam>,
    pub generic_events: Vec<GenericEvent>,
    pub todos: Vec<TodoItem>,
    pub daily_notes: Vec<DailyNote>,
    #[serde(default)]
    pub weekly_lessons: Vec<WeeklyLesson>,
}

// ============================================================================
// Input DTOs for IPC Commands
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpsertProfileInput {
    pub student_name: String,
    pub university: String,
    pub faculty: String,
    pub degree_course: String,
    pub enrollment_year: Option<i32>,
    pub onboarding_completed: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpsertSessionInput {
    pub id: Option<String>,
    pub name: String,
    pub subtitle: String,
    pub session_type: String,
    pub academic_year: String,
    pub start_date: String,
    pub end_date: String,
    pub notes: Option<String>,
    pub plan_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StudyUnitDraft {
    pub id: Option<String>,
    pub unit_number: i32,
    pub title: String,
    pub description: Option<String>,
    pub estimated_hours: Option<f64>,
    pub is_completed: Option<bool>,
    pub assigned_date: Option<String>,
    pub plan_unit_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpsertCourseInput {
    pub id: Option<String>,
    pub session_id: String,
    pub sigla: String,
    pub name: String,
    pub code: String,
    pub cfu: i32,
    pub professor: String,
    pub color_hex: String,
    pub difficulty_weight: Option<f64>,
    pub target_hours_override: Option<f64>,
    pub notes: Option<String>,
    pub units: Option<Vec<StudyUnitDraft>>,
    pub plan_course_id: Option<String>,
    pub is_visible: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpsertExamInput {
    pub id: Option<String>,
    pub session_id: String,
    pub course_id: String,
    pub date: String,
    pub time_slot: Option<String>,
    pub duration_minutes: Option<i32>,
    pub round: String,
    pub exam_type: Option<String>,
    pub classroom: Option<String>,
    pub weight_fraction: Option<f64>,
    pub pair_id: Option<String>,
    pub title: Option<String>,
    pub details: Option<String>,
    pub status: Option<String>,
    pub grade: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateItinerePairInput {
    pub session_id: String,
    pub course_id: String,
    pub date1: String,
    pub date2: String,
    pub time_slot1: Option<String>,
    pub time_slot2: Option<String>,
    pub classroom: Option<String>,
    pub details: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpsertGenericEventInput {
    pub id: Option<String>,
    pub session_id: String,
    pub course_id: Option<String>,
    pub date: String,
    pub time_slot: Option<String>,
    pub title: String,
    pub category: String,
    pub details: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpsertTodoInput {
    pub id: Option<String>,
    pub session_id: String,
    pub course_id: Option<String>,
    pub date: String,
    pub text: String,
    pub priority: Option<i32>,
    pub plan_task_id: Option<String>,
    pub planned_hours: Option<f64>,
    pub kind: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpsertWeeklyLessonInput {
    pub id: Option<String>,
    pub session_id: String,
    pub course_id: Option<String>,
    pub course_name: String,
    pub day_of_week: i32,
    pub start_time: String,
    pub end_time: String,
    pub classroom: Option<String>,
    pub professor: Option<String>,
    pub color_hex: Option<String>,
}
