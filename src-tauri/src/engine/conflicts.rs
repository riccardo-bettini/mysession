use crate::models::{ConflictItem, Course, Exam, StudyUnit};
use chrono::NaiveDate;
use std::collections::{BTreeMap, HashMap};

pub fn detect_session_conflicts(
    courses: &[Course],
    study_units: &[StudyUnit],
    exams: &[Exam],
) -> Vec<ConflictItem> {
    let mut conflicts = Vec::new();
    let course_map: HashMap<&str, &Course> = courses.iter().map(|c| (c.id.as_str(), c)).collect();

    let mut course_completion: HashMap<&str, f64> = HashMap::new();
    for c in courses {
        let units: Vec<&StudyUnit> = study_units.iter().filter(|u| u.course_id == c.id).collect();
        let ratio = if units.is_empty() {
            0.0
        } else {
            let done = units.iter().filter(|u| u.is_completed).count();
            (done as f64) / (units.len() as f64)
        };
        course_completion.insert(c.id.as_str(), ratio);
    }

    let mut active_exams: Vec<&Exam> = exams
        .iter()
        .filter(|e| e.status != "withdrawn" && e.status != "passed")
        .collect();
    active_exams.sort_by(|a, b| a.date.cmp(&b.date).then_with(|| a.time_slot.cmp(&b.time_slot)));

    // 1. Direct Date Collisions
    let mut by_date: BTreeMap<&str, Vec<&Exam>> = BTreeMap::new();
    for ex in &active_exams {
        by_date.entry(ex.date.as_str()).or_default().push(*ex);
    }

    for (date_str, day_exams) in &by_date {
        if day_exams.len() > 1 {
            let exam_ids: Vec<String> = day_exams.iter().map(|e| e.id.clone()).collect();
            let siglas: Vec<String> = day_exams
                .iter()
                .map(|e| {
                    course_map
                        .get(e.course_id.as_str())
                        .map(|c| c.sigla.clone())
                        .unwrap_or_else(|| "ESAME".to_string())
                })
                .collect();

            let mut sorted_exam_ids = exam_ids.clone();
            sorted_exam_ids.sort();
            let conflict_id = format!("conf_direct_{}_{}", date_str, sorted_exam_ids.join("_"));

            conflicts.push(ConflictItem {
                id: conflict_id,
                kind: "direct_date_overlap".to_string(),
                severity: "critical".to_string(),
                date: (*date_str).to_string(),
                secondary_date: None,
                exam_ids,
                course_siglas: siglas.clone(),
                days_gap: 0,
                recommended_min_days: 4,
                title: format!("Sovrapposizione Appelli ({})", siglas.join(" + ")),
                description: format!(
                    "Sono programmati {} appelli nella stessa data ({}): {}.",
                    day_exams.len(),
                    date_str,
                    siglas.join(", ")
                ),
                suggestion:
                    "Pianifica il superamento di una materia in un appello precedente o sposta uno degli appelli."
                        .to_string(),
            });
        }
    }

    // 2. Proximity / Insufficient CFU Buffer Conflicts
    for i in 0..active_exams.len() {
        let ex_a = active_exams[i];
        let Some(date_a) = NaiveDate::parse_from_str(&ex_a.date, "%Y-%m-%d").ok() else {
            continue;
        };

        for ex_b in active_exams.iter().skip(i + 1) {
            if ex_b.date == ex_a.date || ex_b.course_id == ex_a.course_id {
                continue;
            }
            if ex_a.round != ex_b.round {
                continue;
            }

            let Some(date_b) = NaiveDate::parse_from_str(&ex_b.date, "%Y-%m-%d").ok() else {
                continue;
            };

            let delta_days = (date_b - date_a).num_days() as i32;
            if delta_days <= 0 {
                continue;
            }

            let course_b = course_map.get(ex_b.course_id.as_str()).copied();
            let cfu_b = course_b.map(|c| c.cfu).unwrap_or(6.0);
            let weight_b = course_b.map(|c| c.difficulty_weight).unwrap_or(1.0);
            let completion_b = course_completion.get(ex_b.course_id.as_str()).copied().unwrap_or(0.0);

            let raw_min = 0.45 * cfu_b * weight_b * (1.0 - 0.6 * completion_b);
            let recommended_min_days = (raw_min.ceil() as i32).max(2);

            if delta_days < recommended_min_days {
                let sigla_a = course_map
                    .get(ex_a.course_id.as_str())
                    .map(|c| c.sigla.clone())
                    .unwrap_or_else(|| "Esame A".to_string());
                let sigla_b = course_b
                    .map(|c| c.sigla.clone())
                    .unwrap_or_else(|| "Esame B".to_string());

                let severity = if delta_days <= 2 && cfu_b >= 6.0 {
                    "critical"
                } else {
                    "warning"
                };

                conflicts.push(ConflictItem {
                    id: format!("conf_buffer_{}_{}", ex_a.id, ex_b.id),
                    kind: "insufficient_cfu_buffer".to_string(),
                    severity: severity.to_string(),
                    date: ex_b.date.clone(),
                    secondary_date: Some(ex_a.date.clone()),
                    exam_ids: vec![ex_a.id.clone(), ex_b.id.clone()],
                    course_siglas: vec![sigla_a.clone(), sigla_b.clone()],
                    days_gap: delta_days,
                    recommended_min_days,
                    title: format!("Buffer stretto: {} -> {} (+{} gg)", sigla_a, sigla_b, delta_days),
                    description: format!(
                        "Solo {} giorni tra {} ({}) e {} ({}, {} CFU). Consigliati almeno {} giorni.",
                        delta_days, sigla_a, ex_a.date, sigla_b, ex_b.date, cfu_b, recommended_min_days
                    ),
                    suggestion: format!(
                        "Anticipa la preparazione di {} prima del {}.",
                        sigla_b, ex_a.date
                    ),
                });
            }
            break;
        }
    }

    conflicts
}
