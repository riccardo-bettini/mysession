use serde::{Deserialize, Serialize};

pub const SUPPORTED_SCHEMA_VERSION: i64 = 1;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct PlanPayload {
    pub schema_version: i64,
    pub plan_id: String,
    pub revision: i64,
    pub generated_at: String,
    pub session: PlanSession,
    pub constraints: PlanConstraints,
    pub courses: Vec<PlanCourse>,
    pub exams: Vec<PlanExam>,
    pub units: Vec<PlanUnit>,
    pub days: Vec<PlanDay>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct PlanSession {
    pub name: String,
    #[serde(default)]
    pub subtitle: String,
    pub session_type: String,
    pub academic_year: String,
    pub start_date: String,
    pub end_date: String,
    #[serde(default)]
    pub notes: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct PlanConstraints {
    pub max_hours_per_day: f64,
    pub study_weekdays: Vec<i32>,
    #[serde(default)]
    pub off_dates: Vec<String>,
    pub review_days_before_exam: i32,
    pub timezone: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct PlanCourse {
    pub id: String,
    pub sigla: String,
    pub name: String,
    #[serde(default)]
    pub code: String,
    pub cfu: f64,
    #[serde(default)]
    pub professor: String,
    pub color_hex: String,
    #[serde(default = "default_difficulty")]
    pub difficulty_weight: f64,
    pub target_hours: Option<f64>,
    pub active: bool,
    #[serde(default)]
    pub notes: String,
}

fn default_difficulty() -> f64 {
    1.0
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct PlanExam {
    pub id: String,
    pub course_id: String,
    pub date: String,
    #[serde(default)]
    pub time_slot: String,
    #[serde(default)]
    pub duration_minutes: i32,
    pub round: String,
    #[serde(default)]
    pub exam_type: String,
    #[serde(default)]
    pub classroom: String,
    #[serde(default)]
    pub title: String,
    pub status: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct PlanUnit {
    pub id: String,
    pub course_id: String,
    pub unit_number: i32,
    pub title: String,
    #[serde(default)]
    pub description: String,
    pub estimated_hours: f64,
    #[serde(default)]
    pub priority: Option<i32>,
    #[serde(default)]
    pub source: Option<String>,
    #[serde(default)]
    pub aliases: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct PlanDay {
    pub date: String,
    pub tasks: Vec<PlanTask>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct PlanTask {
    pub id: String,
    pub course_id: String,
    #[serde(default)]
    pub unit_id: Option<String>,
    pub text: String,
    pub planned_hours: f64,
    pub kind: String,
    #[serde(default)]
    pub priority: Option<i32>,
    #[serde(default)]
    pub source: Option<String>,
    #[serde(default)]
    pub aliases: Vec<String>,
}
