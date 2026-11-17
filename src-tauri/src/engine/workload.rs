use crate::models::{
    AcademicSession, CategoryProgress, Course, CourseProgressMetric, DailyQuiz, Exam, QuizAttempt,
    StudyUnit, TaskItem, TelemetrySummary,
};
use chrono::{Duration, Local, NaiveDate};
use std::collections::{HashMap, HashSet};

pub const FIVE_CATEGORIES: [&str; 5] = [
    "teoria",
    "esercizi",
    "temi_esame",
    "dimostrazioni",
    "domande_orale",
];

fn normalize_category(cat: &str) -> &str {
    match cat.trim().to_lowercase().as_str() {
        "teoria" | "study" => "teoria",
        "esercizi" | "exercise" => "esercizi",
        "temi_esame" | "past-exam" => "temi_esame",
        "dimostrazioni" => "dimostrazioni",
        "domande_orale" | "ripasso" | "review" => "domande_orale",
        _ => "teoria",
    }
}

#[allow(clippy::too_many_arguments)]
pub fn compute_session_telemetry(
    _session: &AcademicSession,
    courses: &[Course],
    study_units: &[StudyUnit],
    exams: &[Exam],
    tasks: &[TaskItem],
    quizzes: &[DailyQuiz],
    quiz_attempts: &[QuizAttempt],
    study_hours_per_cfu: f64,
    override_today: Option<NaiveDate>,
) -> TelemetrySummary {
    let today = override_today.unwrap_or_else(|| Local::now().date_naive());
    let today_str = today.format("%Y-%m-%d").to_string();

    let course_map: HashMap<&str, &Course> = courses.iter().map(|c| (c.id.as_str(), c)).collect();

    let mut planned_exams: Vec<&Exam> = exams
        .iter()
        .filter(|e| e.status != "withdrawn" && e.status != "passed")
        .collect();
    planned_exams.sort_by(|a, b| a.date.cmp(&b.date));

    let next_target_exam = planned_exams
        .iter()
        .find(|e| {
            NaiveDate::parse_from_str(&e.date, "%Y-%m-%d")
                .map(|d| d >= today)
                .unwrap_or(false)
        })
        .copied()
        .or_else(|| planned_exams.first().copied())
        .cloned();

    let next_target_course = next_target_exam
        .as_ref()
        .and_then(|e| course_map.get(e.course_id.as_str()).copied())
        .cloned();

    let days_to_next_target = next_target_exam.as_ref().and_then(|e| {
        NaiveDate::parse_from_str(&e.date, "%Y-%m-%d")
            .ok()
            .map(|d| (d - today).num_days())
    });

    let total_cfu: f64 = courses.iter().filter(|c| c.is_active).map(|c| c.cfu).sum();

    // Daily workload map from tasks
    let mut daily_workload_map: HashMap<String, f64> = HashMap::new();
    let mut daily_completed_map: HashMap<String, f64> = HashMap::new();
    for t in tasks {
        *daily_workload_map.entry(t.date.clone()).or_insert(0.0) += t.planned_hours;
        if t.done {
            *daily_completed_map.entry(t.date.clone()).or_insert(0.0) += t.planned_hours;
        }
    }
    for v in daily_workload_map.values_mut() {
        *v = (*v * 10.0).round() / 10.0;
    }

    // Last 7 days hours (6 days ago .. today)
    let mut hours_last_7_days = Vec::with_capacity(7);
    for offset in (-6..=0).rev() {
        let dt = today - Duration::days(offset);
        let iso = dt.format("%Y-%m-%d").to_string();
        let hrs = daily_completed_map
            .get(&iso)
            .copied()
            .or_else(|| daily_workload_map.get(&iso).copied())
            .unwrap_or(0.0);
        hours_last_7_days.push((iso, (hrs * 10.0).round() / 10.0));
    }

    // Quiz attempts latest by quiz_id
    let mut latest_attempt_by_quiz: HashMap<&str, &QuizAttempt> = HashMap::new();
    for att in quiz_attempts {
        latest_attempt_by_quiz
            .entry(att.quiz_id.as_str())
            .and_modify(|existing| {
                if att.completed_at > existing.completed_at {
                    *existing = att;
                }
            })
            .or_insert(att);
    }

    let quizzes_total = quizzes.len();
    let mut quizzes_completed = 0usize;
    let mut score_sum = 0.0;
    for q in quizzes {
        if let Some(att) = latest_attempt_by_quiz.get(q.id.as_str()) {
            quizzes_completed += 1;
            score_sum += att.score_pct;
        }
    }
    let quiz_overall_avg_score = if quizzes_completed > 0 {
        Some(((score_sum / quizzes_completed as f64) * 10.0).round() / 10.0)
    } else {
        None
    };

    // Session category breakdown
    let mut category_totals = Vec::new();
    for cat in FIVE_CATEGORIES {
        let matching: Vec<&TaskItem> = tasks
            .iter()
            .filter(|t| normalize_category(&t.category) == cat)
            .collect();
        let total_t = matching.len();
        let comp_t = matching.iter().filter(|t| t.done).count();
        let planned_h: f64 = matching.iter().map(|t| t.planned_hours).sum();
        let comp_h: f64 = matching.iter().filter(|t| t.done).map(|t| t.planned_hours).sum();
        let pct = if planned_h > 0.0 {
            (comp_h / planned_h) * 100.0
        } else if total_t > 0 {
            (comp_t as f64 / total_t as f64) * 100.0
        } else {
            0.0
        };
        category_totals.push(CategoryProgress {
            category: cat.to_string(),
            total_tasks: total_t,
            completed_tasks: comp_t,
            planned_hours: (planned_h * 10.0).round() / 10.0,
            completed_hours: (comp_h * 10.0).round() / 10.0,
            completion_pct: pct.round(),
        });
    }

    // Course progress
    let mut course_progress = Vec::new();
    for c in courses {
        let c_tasks: Vec<&TaskItem> = tasks
            .iter()
            .filter(|t| t.course_id.as_deref() == Some(c.id.as_str()))
            .collect();
        let c_units: Vec<&StudyUnit> = study_units
            .iter()
            .filter(|u| u.course_id == c.id)
            .collect();
        let c_quizzes: Vec<&DailyQuiz> = quizzes
            .iter()
            .filter(|q| q.course_id == c.id)
            .collect();

        let total_tasks = c_tasks.len();
        let completed_tasks = c_tasks.iter().filter(|t| t.done).count();
        let total_units = c_units.len();
        let completed_units = c_units.iter().filter(|u| u.is_completed).count();

        let planned_from_tasks: f64 = c_tasks.iter().map(|t| t.planned_hours).sum();
        let completed_from_tasks: f64 = c_tasks
            .iter()
            .filter(|t| t.done)
            .map(|t| t.planned_hours)
            .sum();

        let planned_hours_total = if planned_from_tasks > 0.0 {
            planned_from_tasks
        } else {
            c.target_hours
                .unwrap_or_else(|| c.cfu * study_hours_per_cfu.max(5.0) * c.difficulty_weight)
        };
        let completed_hours_total = completed_from_tasks;

        let completion_pct = if planned_from_tasks > 0.0 {
            (completed_from_tasks / planned_from_tasks) * 100.0
        } else if total_units > 0 {
            (completed_units as f64 / total_units as f64) * 100.0
        } else {
            0.0
        };

        let mut c_quizzes_done = 0usize;
        let mut c_score_sum = 0.0;
        let mut seen_q: HashSet<&str> = HashSet::new();
        for q in &c_quizzes {
            if seen_q.insert(q.id.as_str()) {
                if let Some(att) = latest_attempt_by_quiz.get(q.id.as_str()) {
                    c_quizzes_done += 1;
                    c_score_sum += att.score_pct;
                }
            }
        }
        let quiz_average_score = if c_quizzes_done > 0 {
            Some(((c_score_sum / c_quizzes_done as f64) * 10.0).round() / 10.0)
        } else {
            None
        };

        let course_next_exam = planned_exams
            .iter()
            .find(|e| {
                e.course_id == c.id
                    && NaiveDate::parse_from_str(&e.date, "%Y-%m-%d")
                        .map(|d| d >= today)
                        .unwrap_or(false)
            })
            .copied();

        let (next_exam_date, days_until_exam, recommended_daily_hours, criticality_score) =
            if let Some(ex) = course_next_exam {
                if let Ok(ex_date) = NaiveDate::parse_from_str(&ex.date, "%Y-%m-%d") {
                    let days_left = (ex_date - today).num_days().max(1);
                    let remaining_h = (planned_hours_total - completed_hours_total).max(0.0);
                    let daily_rec = ((remaining_h / (days_left as f64)) * 10.0).round() / 10.0;
                    let crit = ((remaining_h * c.difficulty_weight) / (days_left as f64) * 10.0)
                        .round()
                        / 10.0;
                    (Some(ex.date.clone()), Some((ex_date - today).num_days()), daily_rec, crit)
                } else {
                    (Some(ex.date.clone()), None, 0.0, 0.0)
                }
            } else {
                (None, None, 0.0, 0.0)
            };

        let mut cat_breakdown = Vec::new();
        for cat in FIVE_CATEGORIES {
            let matching: Vec<&TaskItem> = c_tasks
                .iter()
                .copied()
                .filter(|t| normalize_category(&t.category) == cat)
                .collect();
            let t_cnt = matching.len();
            let c_cnt = matching.iter().filter(|t| t.done).count();
            let p_h: f64 = matching.iter().map(|t| t.planned_hours).sum();
            let c_h: f64 = matching.iter().filter(|t| t.done).map(|t| t.planned_hours).sum();
            let pct = if p_h > 0.0 {
                (c_h / p_h) * 100.0
            } else if t_cnt > 0 {
                (c_cnt as f64 / t_cnt as f64) * 100.0
            } else {
                0.0
            };
            cat_breakdown.push(CategoryProgress {
                category: cat.to_string(),
                total_tasks: t_cnt,
                completed_tasks: c_cnt,
                planned_hours: (p_h * 10.0).round() / 10.0,
                completed_hours: (c_h * 10.0).round() / 10.0,
                completion_pct: pct.round(),
            });
        }

        course_progress.push(CourseProgressMetric {
            course_id: c.id.clone(),
            sigla: c.sigla.clone(),
            name: c.name.clone(),
            color_hex: c.color_hex.clone(),
            cfu: c.cfu,
            professor: c.professor.clone(),
            next_exam_date,
            days_until_exam,
            total_tasks,
            completed_tasks,
            total_units,
            completed_units,
            completion_pct: (completion_pct * 10.0).round() / 10.0,
            planned_hours_total: (planned_hours_total * 10.0).round() / 10.0,
            completed_hours_total: (completed_hours_total * 10.0).round() / 10.0,
            quizzes_total: c_quizzes.len(),
            quizzes_completed: c_quizzes_done,
            quiz_average_score,
            recommended_daily_hours,
            criticality_score,
            category_breakdown: cat_breakdown,
        });
    }

    let tasks_total = tasks.len();
    let tasks_completed = tasks.iter().filter(|t| t.done).count();
    let planned_hours_total: f64 = tasks.iter().map(|t| t.planned_hours).sum();
    let completed_hours_total: f64 = tasks.iter().filter(|t| t.done).map(|t| t.planned_hours).sum();

    let overall_progress_pct = if planned_hours_total > 0.0 {
        ((completed_hours_total / planned_hours_total) * 100.0).round()
    } else if tasks_total > 0 {
        ((tasks_completed as f64 / tasks_total as f64) * 100.0).round()
    } else {
        0.0
    };

    TelemetrySummary {
        reference_today: today_str,
        next_target_exam,
        next_target_course,
        days_to_next_target,
        courses_count: courses.len(),
        exams_count: exams.len(),
        total_cfu,
        tasks_total,
        tasks_completed,
        quizzes_total,
        quizzes_completed,
        quiz_overall_avg_score,
        planned_hours_total: (planned_hours_total * 10.0).round() / 10.0,
        completed_hours_total: (completed_hours_total * 10.0).round() / 10.0,
        overall_progress_pct,
        hours_last_7_days,
        category_totals,
        course_progress,
        daily_workload_map,
    }
}
