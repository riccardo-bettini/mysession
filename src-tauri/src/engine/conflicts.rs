use crate::models::{ConflictItem, Course, Exam, StudyUnit};
use chrono::NaiveDate;
use std::collections::{BTreeMap, HashMap};

/// Real-time Conflict Detection Engine:
/// 1. Direct Date / Time Slot Overlap (`direct_date_overlap`) — 2 or more planned exams on the exact same date.
/// 2. Insufficient Preparation Buffer (`insufficient_cfu_buffer`) — consecutive exams within the same round
///    (or across rounds) where the gap in days `delta_days` is smaller than the minimum recommended preparation
///    buffer `B_min = max(2, ceil(0.45 * CFU * weight * (1.0 - 0.6 * completion_ratio)))`.
pub fn detect_session_conflicts(
    courses: &[Course],
    study_units: &[StudyUnit],
    exams: &[Exam],
) -> Vec<ConflictItem> {
    let mut conflicts = Vec::new();

    let course_map: HashMap<&str, &Course> = courses.iter().map(|c| (c.id.as_str(), c)).collect();

    // Precompute unit completion ratio per course: p_c in [0.0, 1.0]
    let mut course_completion: HashMap<&str, f64> = HashMap::new();
    for c in courses {
        let units: Vec<&StudyUnit> = study_units
            .iter()
            .filter(|u| u.course_id == c.id)
            .collect();
        let ratio = if units.is_empty() {
            0.0
        } else {
            let done = units.iter().filter(|u| u.is_completed).count();
            (done as f64) / (units.len() as f64)
        };
        course_completion.insert(c.id.as_str(), ratio);
    }

    // Filter active/planned exams
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

            conflicts.push(ConflictItem {
                id: format!("conf_direct_{}", date_str),
                kind: "direct_date_overlap".to_string(),
                severity: "critical".to_string(),
                date: (*date_str).to_string(),
                secondary_date: None,
                exam_ids,
                course_siglas: siglas.clone(),
                days_gap: 0,
                recommended_min_days: 4,
                title: format!(
                    "Sovrapposizione Diretta Appelli ({})",
                    siglas.join(" + ")
                ),
                description: format!(
                    "Sono programmati {} appelli nella stessa data ({}): {}.",
                    day_exams.len(),
                    date_str,
                    siglas.join(", ")
                ),
                suggestion:
                    "Pianifica il superamento di almeno una materia al giro precedente o sposta uno degli appelli allo slot di recupero."
                        .to_string(),
            });
        }
    }

    // 2. Proximity / Insufficient CFU Buffer Conflicts
    // Compare consecutive exams on distinct dates within the same round (or general chronological sequence)
    for i in 0..active_exams.len() {
        let ex_a = active_exams[i];
        let Some(date_a) = NaiveDate::parse_from_str(&ex_a.date, "%Y-%m-%d").ok() else {
            continue;
        };

        for ex_b in active_exams.iter().skip(i + 1) {
            if ex_b.date == ex_a.date || ex_b.course_id == ex_a.course_id {
                continue;
            }
            // Only evaluate proximity within the same round or between primary exams
            let same_round_group = ex_a.round == ex_b.round
                || (ex_a.round.starts_with("itinere") && ex_b.round.starts_with("itinere"));
            if !same_round_group {
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
            let cfu_b = course_b.map(|c| c.cfu).unwrap_or(6);
            let weight_b = course_b.map(|c| c.difficulty_weight).unwrap_or(1.0);
            let completion_b = course_completion
                .get(ex_b.course_id.as_str())
                .copied()
                .unwrap_or(0.0);

            // Minimum safe buffer formula: B_min = max(2, ceil(0.45 * CFU * weight * (1.0 - 0.6 * completion)))
            let raw_min = 0.45 * (cfu_b as f64) * weight_b * (1.0 - 0.6 * completion_b) * ex_b.weight_fraction;
            let recommended_min_days = (raw_min.ceil() as i32).max(2);

            if delta_days < recommended_min_days {
                let sigla_a = course_map
                    .get(ex_a.course_id.as_str())
                    .map(|c| c.sigla.clone())
                    .unwrap_or_else(|| "Esame A".to_string());
                let sigla_b = course_b
                    .map(|c| c.sigla.clone())
                    .unwrap_or_else(|| "Esame B".to_string());

                let severity = if delta_days <= 2 && cfu_b >= 6 {
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
                    title: format!(
                        "Buffer Critico: {} → {} (+{} gg per {} CFU)",
                        sigla_a, sigla_b, delta_days, cfu_b
                    ),
                    description: format!(
                        "Solo {} giorni di intervallo tra {} ({}) e {} ({}, {} CFU). Buffer consigliato: almeno {} giorni.",
                        delta_days, sigla_a, ex_a.date, sigla_b, ex_b.date, cfu_b, recommended_min_days
                    ),
                    suggestion: format!(
                        "Completa in anticipo le unità didattiche di {} prima del {} oppure valuta lo split su un altro giro.",
                        sigla_b, ex_a.date
                    ),
                });
            }

            // Only check the immediately following exam in the same round
            break;
        }
    }

    conflicts
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_conflict_detection_direct_and_buffer() {
        let courses = vec![
            Course {
                id: "mam".to_string(),
                session_id: "s1".to_string(),
                sigla: "MAM".to_string(),
                name: "Meccanica Applicata".to_string(),
                code: "054828".to_string(),
                cfu: 10,
                professor: "Rocchi".to_string(),
                color_hex: "#0284C7".to_string(),
                difficulty_weight: 1.0,
                target_hours_override: None,
                notes: String::new(),
                sort_order: 0,
                plan_course_id: None,
                is_visible: true,
            },
            Course {
                id: "mse".to_string(),
                session_id: "s1".to_string(),
                sigla: "MSE".to_string(),
                name: "Macchine e Sistemi Energetici".to_string(),
                code: "MSE".to_string(),
                cfu: 9,
                professor: "Docente".to_string(),
                color_hex: "#059669".to_string(),
                difficulty_weight: 1.0,
                target_hours_override: None,
                notes: String::new(),
                sort_order: 1,
                plan_course_id: None,
                is_visible: true,
            },
            Course {
                id: "dcm".to_string(),
                session_id: "s1".to_string(),
                sigla: "DCM".to_string(),
                name: "Dinamica e Controllo".to_string(),
                code: "DCM".to_string(),
                cfu: 9,
                professor: "Docente".to_string(),
                color_hex: "#7C3AED".to_string(),
                difficulty_weight: 1.0,
                target_hours_override: None,
                notes: String::new(),
                sort_order: 2,
                plan_course_id: None,
                is_visible: true,
            },
        ];

        let exams = vec![
            Exam {
                id: "mse_1".to_string(),
                session_id: "s1".to_string(),
                course_id: "mse".to_string(),
                date: "2027-01-18".to_string(),
                time_slot: "09:00".to_string(),
                duration_minutes: 120,
                round: "1".to_string(),
                exam_type: "scritto".to_string(),
                classroom: String::new(),
                weight_fraction: 1.0,
                pair_id: None,
                title: "1° Appello MSE".to_string(),
                details: String::new(),
                status: "planned".to_string(),
                grade: None,
            },
            Exam {
                id: "dcm_1".to_string(),
                session_id: "s1".to_string(),
                course_id: "dcm".to_string(),
                date: "2027-01-20".to_string(), // Only 2 days after MSE (9 CFU) -> Buffer conflict!
                time_slot: "09:00".to_string(),
                duration_minutes: 120,
                round: "1".to_string(),
                exam_type: "scritto".to_string(),
                classroom: String::new(),
                weight_fraction: 1.0,
                pair_id: None,
                title: "1° Appello DCM".to_string(),
                details: String::new(),
                status: "planned".to_string(),
                grade: None,
            },
            Exam {
                id: "mam_2".to_string(),
                session_id: "s1".to_string(),
                course_id: "mam".to_string(),
                date: "2027-02-08".to_string(), // Same day as dcm_2 -> Direct overlap!
                time_slot: "09:00".to_string(),
                duration_minutes: 120,
                round: "2".to_string(),
                exam_type: "scritto".to_string(),
                classroom: String::new(),
                weight_fraction: 1.0,
                pair_id: None,
                title: "2° Appello MAM".to_string(),
                details: String::new(),
                status: "planned".to_string(),
                grade: None,
            },
            Exam {
                id: "dcm_2".to_string(),
                session_id: "s1".to_string(),
                course_id: "dcm".to_string(),
                date: "2027-02-08".to_string(),
                time_slot: "14:00".to_string(),
                duration_minutes: 120,
                round: "2".to_string(),
                exam_type: "scritto".to_string(),
                classroom: String::new(),
                weight_fraction: 1.0,
                pair_id: None,
                title: "2° Appello DCM".to_string(),
                details: String::new(),
                status: "planned".to_string(),
                grade: None,
            },
        ];

        let conflicts = detect_session_conflicts(&courses, &[], &exams);
        assert!(conflicts.iter().any(|c| c.kind == "direct_date_overlap" && c.date == "2027-02-08"));
        assert!(conflicts.iter().any(|c| c.kind == "insufficient_cfu_buffer" && c.date == "2027-01-20"));
    }
}
