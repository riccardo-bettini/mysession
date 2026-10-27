use crate::engine::holidays::get_italian_holiday_name;
use crate::models::{
    AcademicSession, ConflictItem, Course, CourseWorkloadMetric, Exam, HorizonPin, HorizonTick,
    StudyUnit, TelemetrySummary,
};
use chrono::{Datelike, Duration, Local, NaiveDate};
use std::collections::{HashMap, HashSet};

const ITALIAN_MONTHS_SHORT: [&str; 12] = [
    "GEN", "FEB", "MAR", "APR", "MAG", "GIU", "LUG", "AGO", "SET", "OTT", "NOV", "DIC",
];

/// Deterministic Workload & Telemetry Engine:
/// - Computes remaining autonomous study hours per course from CFU, difficulty weight, and uncompleted StudyUnits.
/// - Distributes daily study workload across available days up to each course's next upcoming exam.
/// - Computes the next target exam countdown, criticality index, and Horizon Ribbon percentages.
#[allow(clippy::too_many_arguments)]
pub fn compute_session_telemetry(
    session: &AcademicSession,
    courses: &[Course],
    study_units: &[StudyUnit],
    exams: &[Exam],
    conflicts: &[ConflictItem],
    study_hours_per_cfu: f64,
    weekly_holiday_day: i32,
    override_today: Option<NaiveDate>,
) -> TelemetrySummary {
    let today = override_today.unwrap_or_else(|| Local::now().date_naive());
    let today_str = today.format("%Y-%m-%d").to_string();

    let course_map: HashMap<&str, &Course> = courses.iter().map(|c| (c.id.as_str(), c)).collect();

    // Sort planned exams chronologically
    let mut planned_exams: Vec<&Exam> = exams
        .iter()
        .filter(|e| e.status != "withdrawn" && e.status != "passed")
        .collect();
    planned_exams.sort_by(|a, b| a.date.cmp(&b.date));

    // Find next upcoming target exam (>= today, or fallback to first exam)
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

    let total_cfu: i32 = courses.iter().map(|c| c.cfu).sum();
    let units_total = study_units.len();
    let completed_units_total = study_units.iter().filter(|u| u.is_completed).count();
    let overall_progress_pct = if units_total > 0 {
        ((completed_units_total as f64) / (units_total as f64) * 100.0).round()
    } else {
        0.0
    };

    let mut daily_workload_map: HashMap<String, f64> = HashMap::new();
    let mut course_workloads: Vec<CourseWorkloadMetric> = Vec::new();

    for c in courses {
        let c_units: Vec<&StudyUnit> = study_units
            .iter()
            .filter(|u| u.course_id == c.id)
            .collect();
        let total_u = c_units.len();
        let comp_u = c_units.iter().filter(|u| u.is_completed).count();
        let completion_pct = if total_u > 0 {
            (comp_u as f64 / total_u as f64) * 100.0
        } else {
            0.0
        };

        let base_hours = c
            .target_hours_override
            .unwrap_or_else(|| (c.cfu as f64) * study_hours_per_cfu.max(5.0) * c.difficulty_weight);

        let remaining_ratio = if total_u > 0 {
            ((total_u - comp_u) as f64) / (total_u as f64)
        } else {
            1.0
        };
        let remaining_study_hours = (base_hours * remaining_ratio * 10.0).round() / 10.0;

        // Next exam for this specific course (>= today, or last exam of course)
        let course_next_exam = planned_exams
            .iter()
            .find(|e| {
                e.course_id == c.id
                    && NaiveDate::parse_from_str(&e.date, "%Y-%m-%d")
                        .map(|d| d >= today)
                        .unwrap_or(false)
            })
            .copied();

        let (next_exam_date, days_until_next_exam, recommended_daily_hours, criticality_score) =
            if let Some(ex) = course_next_exam {
                if let Ok(ex_date) = NaiveDate::parse_from_str(&ex.date, "%Y-%m-%d") {
                    let raw_days = (ex_date - today).num_days().max(1);

                    // Calculate weighted available study days in [today, ex_date)
                    let mut effective_days = 0.0_f64;
                    let mut day_weights: Vec<(NaiveDate, f64)> = Vec::new();
                    let mut cursor = today;
                    let max_horizon = today + Duration::days(120);
                    let end_cursor = ex_date.min(max_horizon);

                    while cursor < end_cursor {
                        let weekday_num = cursor.weekday().num_days_from_sunday() as i32;
                        let is_weekly_rest =
                            weekly_holiday_day >= 0 && weekday_num == weekly_holiday_day;
                        let is_nat_holiday = get_italian_holiday_name(cursor).is_some();

                        let w = if is_nat_holiday || is_weekly_rest {
                            0.35
                        } else {
                            1.0
                        };
                        effective_days += w;
                        day_weights.push((cursor, w));
                        cursor += Duration::days(1);
                    }

                    let eff = effective_days.max(1.0);
                    let daily_rec = ((remaining_study_hours / eff) * 10.0).round() / 10.0;

                    for (d, w) in day_weights {
                        let key = d.format("%Y-%m-%d").to_string();
                        let entry = daily_workload_map.entry(key).or_insert(0.0);
                        *entry = ((*entry + daily_rec * w) * 10.0).round() / 10.0;
                    }

                    let crit = ((remaining_study_hours * c.difficulty_weight) / (raw_days as f64)
                        * 100.0)
                        .round()
                        / 100.0;
                    (Some(ex.date.clone()), Some(raw_days), daily_rec, crit)
                } else {
                    (Some(ex.date.clone()), None, 0.0, 0.0)
                }
            } else {
                (None, None, 0.0, 0.0)
            };

        course_workloads.push(CourseWorkloadMetric {
            course_id: c.id.clone(),
            sigla: c.sigla.clone(),
            name: c.name.clone(),
            color_hex: c.color_hex.clone(),
            cfu: c.cfu,
            total_units: total_u,
            completed_units: comp_u,
            completion_pct: (completion_pct * 10.0).round() / 10.0,
            remaining_study_hours,
            next_exam_date,
            days_until_next_exam,
            recommended_daily_hours,
            criticality_score,
        });
    }

    course_workloads.sort_by(|a, b| {
        b.criticality_score
            .partial_cmp(&a.criticality_score)
            .unwrap_or(std::cmp::Ordering::Equal)
    });

    let most_critical_course_sigla = course_workloads
        .first()
        .filter(|cw| cw.criticality_score > 0.0)
        .map(|cw| cw.sigla.clone());

    let recommended_hours_today = daily_workload_map.get(&today_str).copied().unwrap_or(0.0);

    // Horizon Ribbon Coordinates
    let mut start_date = NaiveDate::parse_from_str(&session.start_date, "%Y-%m-%d")
        .unwrap_or_else(|_| NaiveDate::from_ymd_opt(today.year(), 1, 1).unwrap());
    let mut end_date = NaiveDate::parse_from_str(&session.end_date, "%Y-%m-%d")
        .unwrap_or_else(|_| NaiveDate::from_ymd_opt(today.year(), 12, 31).unwrap());

    for ex in exams {
        if let Ok(d) = NaiveDate::parse_from_str(&ex.date, "%Y-%m-%d") {
            let padded_start = d - Duration::days(5);
            let padded_end = d + Duration::days(5);
            if padded_start < start_date {
                start_date = padded_start;
            }
            if padded_end > end_date {
                end_date = padded_end;
            }
        }
    }

    let total_span_days = (end_date - start_date).num_days().max(15) as f64;
    let pct_for_date = |d: NaiveDate| -> f64 {
        let clamped = d.max(start_date).min(end_date);
        let offset = (clamped - start_date).num_days() as f64;
        ((offset / total_span_days) * 1000.0).round() / 10.0
    };

    let horizon_today_pct = if today >= start_date && today <= end_date {
        Some(pct_for_date(today))
    } else {
        None
    };

    let conflict_exam_ids: HashSet<&str> = conflicts
        .iter()
        .flat_map(|c| c.exam_ids.iter().map(|s| s.as_str()))
        .collect();

    let mut horizon_pins = Vec::new();
    for ex in exams {
        if let Ok(d) = NaiveDate::parse_from_str(&ex.date, "%Y-%m-%d") {
            let course = course_map.get(ex.course_id.as_str()).copied();
            horizon_pins.push(HorizonPin {
                exam_id: ex.id.clone(),
                course_id: ex.course_id.clone(),
                sigla: course
                    .map(|c| c.sigla.clone())
                    .unwrap_or_else(|| "ESAME".to_string()),
                color_hex: course
                    .map(|c| c.color_hex.clone())
                    .unwrap_or_else(|| "#007AFF".to_string()),
                date: ex.date.clone(),
                round: ex.round.clone(),
                title: ex.title.clone(),
                position_pct: pct_for_date(d),
                has_conflict: conflict_exam_ids.contains(ex.id.as_str()),
            });
        }
    }

    let mut horizon_ticks = Vec::new();
    let mut tick_y = start_date.year();
    let mut tick_m = start_date.month();
    while let Some(mid_month) = NaiveDate::from_ymd_opt(tick_y, tick_m, 15) {
        if mid_month > end_date + Duration::days(15) {
            break;
        }
        if mid_month >= start_date - Duration::days(15) {
            horizon_ticks.push(HorizonTick {
                year: tick_y,
                month: tick_m,
                label: ITALIAN_MONTHS_SHORT[(tick_m - 1) as usize].to_string(),
                position_pct: pct_for_date(mid_month),
            });
        }
        if tick_m == 12 {
            tick_m = 1;
            tick_y += 1;
        } else {
            tick_m += 1;
        }
    }

    TelemetrySummary {
        reference_today: today_str,
        next_target_exam,
        next_target_course,
        days_to_next_target,
        courses_count: courses.len(),
        exams_count: exams.len(),
        total_cfu,
        completed_units_total,
        units_total,
        overall_progress_pct,
        most_critical_course_sigla,
        recommended_hours_today,
        course_workloads,
        daily_workload_map,
        horizon_start_date: start_date.format("%Y-%m-%d").to_string(),
        horizon_end_date: end_date.format("%Y-%m-%d").to_string(),
        horizon_today_pct,
        horizon_pins,
        horizon_ticks,
    }
}
