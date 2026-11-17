use serde::{Deserialize, Serialize};
use std::collections::HashMap;

pub const DEFAULT_SIDEBAR_SECTIONS: [&str; 7] = [
    "home",
    "calendar",
    "today",
    "courses",
    "quiz",
    "progress",
    "ai",
];

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AppSettings {
    pub theme: String, // "light" | "dark" | "auto"
    pub backdrop_material: String, // "mica" | "acrylic" | "tabbed" | "solid"
    pub ui_scale: i32, // 80..=140
    pub accent_color: String, // Default "#7C5CFC"
    pub cat_teoria_color: String,
    pub cat_esercizi_color: String,
    pub cat_temi_esame_color: String,
    pub cat_dimostrazioni_color: String,
    pub cat_domande_orale_color: String,
    pub sidebar_sections: Vec<String>,
    pub hidden_sections: Vec<String>,
    pub plan_folder: String,
    pub active_session_id: Option<String>,
    pub study_hours_per_cfu: f64,
    pub start_day_of_week: i32, // 1 = Monday .. 7 = Sunday (or 0 = Sunday)
    pub ai_provider: String,    // "local" | "cli_agy" | "cli_claude" | "cli_codex" | "cli_custom"
    pub ai_local_model_id: String,
    pub ai_custom_cli_command: String,
    pub ai_idle_timeout_minutes: i32,
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            theme: "light".to_string(),
            backdrop_material: "mica".to_string(),
            ui_scale: 100,
            accent_color: "#7C5CFC".to_string(),
            cat_teoria_color: "#3B82F6".to_string(),
            cat_esercizi_color: "#10B981".to_string(),
            cat_temi_esame_color: "#F59E0B".to_string(),
            cat_dimostrazioni_color: "#8B5CF6".to_string(),
            cat_domande_orale_color: "#EC4899".to_string(),
            sidebar_sections: DEFAULT_SIDEBAR_SECTIONS
                .iter()
                .map(|s| s.to_string())
                .collect(),
            hidden_sections: Vec::new(),
            plan_folder: String::new(),
            active_session_id: None,
            study_hours_per_cfu: 15.0,
            start_day_of_week: 1,
            ai_provider: "local".to_string(),
            ai_local_model_id: "qwen2.5-vl-3b".to_string(),
            ai_custom_cli_command: String::new(),
            ai_idle_timeout_minutes: 5,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AcademicSession {
    pub id: String,
    pub plan_id: Option<String>,
    pub name: String,
    pub subtitle: String,
    pub session_type: String,
    pub academic_year: String,
    pub start_date: String,
    pub end_date: String,
    pub max_hours_per_day: f64,
    pub study_weekdays: Vec<i32>,
    pub off_dates: Vec<String>,
    pub review_days_before_exam: i32,
    pub notes: String,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Course {
    pub id: String,
    pub session_id: String,
    pub plan_course_id: Option<String>,
    pub sigla: String,
    pub name: String,
    pub code: String,
    pub cfu: f64,
    pub professor: String,
    pub color_hex: String,
    pub difficulty_weight: f64,
    pub target_hours: Option<f64>,
    pub is_active: bool,
    pub notes: String,
    pub sort_order: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StudyUnit {
    pub id: String,
    pub course_id: String,
    pub session_id: String,
    pub plan_unit_id: Option<String>,
    pub unit_number: i32,
    pub chapter: String,
    pub title: String,
    pub description: String,
    pub source: String,
    pub estimated_hours: f64,
    pub priority: i32,
    pub is_completed: bool,
    pub completed_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Exam {
    pub id: String,
    pub session_id: String,
    pub course_id: String,
    pub plan_exam_id: Option<String>,
    pub date: String,
    pub time_slot: String,
    pub duration_minutes: i32,
    pub round: String,
    pub exam_type: String,
    pub classroom: String,
    pub title: String,
    pub details: String,
    pub status: String, // "planned" | "passed" | "failed" | "withdrawn"
    pub grade: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OperationalBlock {
    pub id: String,
    pub session_id: String,
    pub date: String,
    pub course_id: Option<String>,
    pub block_order: i32,
    pub category: String,
    pub title: String,
    pub chapter: String,
    pub topic: String,
    pub pages: String,
    pub source: String,
    pub duration_hours: f64,
    pub instructions: String,
    pub done: bool,
    pub plan_block_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DailyOperationalPlan {
    pub session_id: String,
    pub date: String,
    pub daily_goal: String,
    pub strategy_note: String,
    pub blocks: Vec<OperationalBlock>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TaskSubItem {
    pub id: String,
    pub task_id: String,
    pub text: String,
    pub category: String,
    pub pages: String,
    pub done: bool,
    pub sort_order: i32,
    pub plan_subtask_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TaskItem {
    pub id: String,
    pub session_id: String,
    pub course_id: Option<String>,
    pub unit_id: Option<String>,
    pub date: String,
    pub category: String, // "teoria" | "esercizi" | "temi_esame" | "dimostrazioni" | "domande_orale" | "ripasso"
    pub title: String,
    pub chapter: String,
    pub topic: String,
    pub pages: String,
    pub source: String,
    pub planned_hours: f64,
    pub done: bool,
    pub priority: i32,
    pub plan_task_id: Option<String>,
    pub created_at: String,
    #[serde(default)]
    pub subtasks: Vec<TaskSubItem>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct QuizQuestion {
    pub id: String,
    pub quiz_id: String,
    pub question_order: i32,
    pub question_type: String, // "multiple_choice" | "open_flashcard"
    pub category: String,
    pub chapter: String,
    pub topic: String,
    pub prompt: String,
    pub options: Vec<String>,
    pub correct_index: Option<i32>,
    pub explanation: String,
    pub plan_question_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct QuizAttempt {
    pub id: String,
    pub quiz_id: String,
    pub session_id: String,
    pub course_id: String,
    pub score_pct: f64,
    pub correct_count: i32,
    pub total_count: i32,
    pub answers_json: String,
    pub wrong_question_ids: Vec<String>,
    pub completed_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DailyQuiz {
    pub id: String,
    pub session_id: String,
    pub course_id: String,
    pub date: String,
    pub title: String,
    pub topic_summary: String,
    pub plan_quiz_id: Option<String>,
    pub questions: Vec<QuizQuestion>,
    pub latest_attempt: Option<QuizAttempt>,
    pub attempts_count: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WeeklyLesson {
    pub id: String,
    pub session_id: String,
    pub course_id: Option<String>,
    pub course_name: String,
    pub day_of_week: i32, // 1=Lun .. 7=Dom
    pub start_time: String,
    pub end_time: String,
    pub classroom: String,
    pub professor: String,
    pub color_hex: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CalendarEvent {
    pub id: String,
    pub session_id: String,
    pub course_id: Option<String>,
    pub date: String,
    pub time_slot: String,
    pub title: String,
    pub category: String,
    pub details: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DailyNote {
    pub session_id: String,
    pub date: String,
    pub content_markdown: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HolidayInfo {
    pub date: String,
    pub name: String,
    pub is_national: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ConflictItem {
    pub id: String,
    pub kind: String,
    pub severity: String,
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
pub struct CategoryProgress {
    pub category: String,
    pub total_tasks: usize,
    pub completed_tasks: usize,
    pub planned_hours: f64,
    pub completed_hours: f64,
    pub completion_pct: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CourseProgressMetric {
    pub course_id: String,
    pub sigla: String,
    pub name: String,
    pub color_hex: String,
    pub cfu: f64,
    pub professor: String,
    pub next_exam_date: Option<String>,
    pub days_until_exam: Option<i64>,
    pub total_tasks: usize,
    pub completed_tasks: usize,
    pub total_units: usize,
    pub completed_units: usize,
    pub completion_pct: f64,
    pub planned_hours_total: f64,
    pub completed_hours_total: f64,
    pub quizzes_total: usize,
    pub quizzes_completed: usize,
    pub quiz_average_score: Option<f64>,
    pub recommended_daily_hours: f64,
    pub criticality_score: f64,
    pub category_breakdown: Vec<CategoryProgress>,
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
    pub total_cfu: f64,
    pub tasks_total: usize,
    pub tasks_completed: usize,
    pub quizzes_total: usize,
    pub quizzes_completed: usize,
    pub quiz_overall_avg_score: Option<f64>,
    pub planned_hours_total: f64,
    pub completed_hours_total: f64,
    pub overall_progress_pct: f64,
    pub hours_last_7_days: Vec<(String, f64)>,
    pub category_totals: Vec<CategoryProgress>,
    pub course_progress: Vec<CourseProgressMetric>,
    pub daily_workload_map: HashMap<String, f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SessionSnapshot {
    pub session: AcademicSession,
    pub courses: Vec<Course>,
    pub study_units: Vec<StudyUnit>,
    pub exams: Vec<Exam>,
    pub operational_plans: Vec<DailyOperationalPlan>,
    pub tasks: Vec<TaskItem>,
    pub quizzes: Vec<DailyQuiz>,
    pub quiz_attempts: Vec<QuizAttempt>,
    pub weekly_lessons: Vec<WeeklyLesson>,
    pub calendar_events: Vec<CalendarEvent>,
    pub daily_notes: Vec<DailyNote>,
    pub conflicts: Vec<ConflictItem>,
    pub telemetry: TelemetrySummary,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PendingPlanNotification {
    pub plan_id: String,
    pub plan_name: String,
    pub revision: i64,
    pub file_path: String,
    pub is_new_session: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BootstrapPayload {
    pub app_version: String,
    pub db_path: String,
    pub settings: AppSettings,
    pub sessions: Vec<AcademicSession>,
    pub active_snapshot: Option<SessionSnapshot>,
    pub pending_plans: Vec<PendingPlanNotification>,
}

// ============================================================================
// Input DTOs for IPC Commands
// ============================================================================

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
    pub max_hours_per_day: Option<f64>,
    pub study_weekdays: Option<Vec<i32>>,
    pub off_dates: Option<Vec<String>>,
    pub review_days_before_exam: Option<i32>,
    pub notes: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpsertCourseInput {
    pub id: Option<String>,
    pub session_id: String,
    pub sigla: String,
    pub name: String,
    pub code: Option<String>,
    pub cfu: f64,
    pub professor: Option<String>,
    pub color_hex: String,
    pub difficulty_weight: Option<f64>,
    pub target_hours: Option<f64>,
    pub is_active: Option<bool>,
    pub notes: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpsertStudyUnitInput {
    pub id: Option<String>,
    pub session_id: String,
    pub course_id: String,
    pub unit_number: i32,
    pub chapter: Option<String>,
    pub title: String,
    pub description: Option<String>,
    pub source: Option<String>,
    pub estimated_hours: f64,
    pub priority: Option<i32>,
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
    pub title: Option<String>,
    pub details: Option<String>,
    pub status: Option<String>,
    pub grade: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpsertTaskInput {
    pub id: Option<String>,
    pub session_id: String,
    pub course_id: Option<String>,
    pub unit_id: Option<String>,
    pub date: String,
    pub category: String,
    pub title: String,
    pub chapter: Option<String>,
    pub topic: Option<String>,
    pub pages: Option<String>,
    pub source: Option<String>,
    pub planned_hours: Option<f64>,
    pub priority: Option<i32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpsertTaskSubItemInput {
    pub id: Option<String>,
    pub task_id: String,
    pub text: String,
    pub category: Option<String>,
    pub pages: Option<String>,
    pub done: Option<bool>,
    pub sort_order: Option<i32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SubmitQuizAttemptInput {
    pub quiz_id: String,
    pub session_id: String,
    pub course_id: String,
    pub score_pct: f64,
    pub correct_count: i32,
    pub total_count: i32,
    pub answers_json: String,
    pub wrong_question_ids: Vec<String>,
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

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpsertCalendarEventInput {
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
pub struct ExtractedLessonItem {
    pub day_of_week: i32,
    pub start_time: String,
    pub end_time: String,
    pub course_name: String,
    pub classroom: String,
    pub professor: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CliAgentInfo {
    pub id: String,       // "cli_agy" | "cli_claude" | "cli_codex" | "cli_custom"
    pub name: String,
    pub command: String,
    pub installed: bool,
    pub resolved_path: Option<String>,
}
