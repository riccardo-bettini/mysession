use crate::plan::model::{PlanPayload, SUPPORTED_SCHEMA_VERSION};
use chrono::NaiveDate;
use std::collections::{HashMap, HashSet};

const VALID_TASK_CATEGORIES: [&str; 6] = [
    "teoria",
    "esercizi",
    "temi_esame",
    "dimostrazioni",
    "domande_orale",
    "ripasso",
];

pub fn validate_plan(plan: &PlanPayload) -> Result<(), Vec<String>> {
    let mut errors = Vec::new();

    if plan.schema_version != SUPPORTED_SCHEMA_VERSION {
        errors.push(format!(
            "Versione schema non supportata: {}. MySession v2.00 supporta schemaVersion {}.",
            plan.schema_version, SUPPORTED_SCHEMA_VERSION
        ));
    }

    if plan.plan_id.trim().is_empty() {
        errors.push("planId non può essere vuoto".to_string());
    }

    if plan.revision < 1 {
        errors.push("revision deve essere maggiore o uguale a 1".to_string());
    }

    let start_date = NaiveDate::parse_from_str(plan.session.start_date.trim(), "%Y-%m-%d");
    let end_date = NaiveDate::parse_from_str(plan.session.end_date.trim(), "%Y-%m-%d");

    let (s_dt, e_dt) = match (start_date, end_date) {
        (Ok(s), Ok(e)) => {
            if s > e {
                errors.push("session.startDate è successiva a endDate".to_string());
            }
            (Some(s), Some(e))
        }
        (Err(_), _) => {
            errors.push(format!("session.startDate non valida: {}", plan.session.start_date));
            (None, None)
        }
        (_, Err(_)) => {
            errors.push(format!("session.endDate non valida: {}", plan.session.end_date));
            (None, None)
        }
    };

    let mut course_ids = HashSet::new();
    for course in &plan.courses {
        if !course_ids.insert(course.id.clone()) {
            errors.push(format!("ID corso duplicato: {}", course.id));
        }
        if course.sigla.trim().is_empty() {
            errors.push(format!("Sigla vuota per il corso {}", course.id));
        }
    }

    let mut unit_ids = HashSet::new();
    for unit in &plan.units {
        if !unit_ids.insert(unit.id.clone()) {
            errors.push(format!("ID unità duplicato: {}", unit.id));
        }
        if !course_ids.contains(&unit.course_id) {
            errors.push(format!(
                "Unità {} fa riferimento a un corso inesistente: {}",
                unit.id, unit.course_id
            ));
        }
    }

    let mut exam_dates_by_course: HashMap<String, Vec<NaiveDate>> = HashMap::new();
    for exam in &plan.exams {
        if !course_ids.contains(&exam.course_id) {
            errors.push(format!(
                "Esame {} fa riferimento a un corso inesistente: {}",
                exam.id, exam.course_id
            ));
        }
        if let Ok(ed) = NaiveDate::parse_from_str(exam.date.trim(), "%Y-%m-%d") {
            if exam.status == "planned" {
                exam_dates_by_course
                    .entry(exam.course_id.clone())
                    .or_default()
                    .push(ed);
            }
        } else {
            errors.push(format!("Data esame {} non valida: {}", exam.id, exam.date));
        }
    }

    let off_dates_set: HashSet<String> = plan.constraints.off_dates.iter().cloned().collect();
    let weekdays_set: HashSet<u32> = plan
        .constraints
        .study_weekdays
        .iter()
        .map(|&w| w as u32)
        .collect();

    let mut seen_dates = HashSet::new();
    let mut last_date: Option<NaiveDate> = None;
    let mut task_ids = HashSet::new();
    let mut quiz_ids = HashSet::new();
    let mut question_ids = HashSet::new();

    for day in &plan.days {
        let current_date = match NaiveDate::parse_from_str(day.date.trim(), "%Y-%m-%d") {
            Ok(d) => d,
            Err(_) => {
                errors.push(format!("Data giorno non valida: {}", day.date));
                continue;
            }
        };

        if !seen_dates.insert(day.date.clone()) {
            errors.push(format!("Data duplicata in days[]: {}", day.date));
        }

        if let Some(prev) = last_date {
            if current_date <= prev {
                errors.push(format!(
                    "days[] non è ordinato cronologicamente: {} segue {}",
                    day.date, prev
                ));
            }
        }
        last_date = Some(current_date);

        if let (Some(s), Some(e)) = (s_dt, e_dt) {
            if current_date < s || current_date > e {
                errors.push(format!(
                    "Data {} fuori dal periodo della sessione ({} - {})",
                    day.date, s, e
                ));
            }
        }

        if off_dates_set.contains(&day.date) {
            errors.push(format!("Pianificate attività nel giorno off: {}", day.date));
        }

        let isoweekday = current_date
            .format("%u")
            .to_string()
            .parse::<u32>()
            .unwrap_or(1);
        if !weekdays_set.is_empty() && !weekdays_set.contains(&isoweekday) {
            errors.push(format!(
                "Data {} cade in un giorno della settimana non di studio",
                day.date
            ));
        }

        if day.operational_plan.daily_goal.trim().is_empty() {
            errors.push(format!("{}: operationalPlan.dailyGoal vuoto", day.date));
        }

        let mut daily_hours = 0.0;
        let mut task_courses = HashSet::new();
        for task in &day.tasks {
            daily_hours += task.planned_hours;
            task_courses.insert(task.course_id.clone());

            if !task_ids.insert(task.id.clone()) {
                errors.push(format!("ID attività duplicato: {}", task.id));
            }
            if !VALID_TASK_CATEGORIES.contains(&task.category.as_str()) {
                errors.push(format!(
                    "Attività {} ha categoria non valida '{}'",
                    task.id, task.category
                ));
            }
            if !course_ids.contains(&task.course_id) {
                errors.push(format!(
                    "Attività {} fa riferimento a un corso inesistente: {}",
                    task.id, task.course_id
                ));
            }
            if let Some(ref uid) = task.unit_id {
                if !unit_ids.contains(uid) {
                    errors.push(format!(
                        "Attività {} fa riferimento a un'unità inesistente: {}",
                        task.id, uid
                    ));
                }
            }
            if let Some(exams) = exam_dates_by_course.get(&task.course_id) {
                if let Some(last_exam) = exams.iter().max() {
                    if current_date > *last_exam {
                        errors.push(format!(
                            "Attività {} pianificata il {}, successiva all'ultimo esame ({})",
                            task.id, current_date, last_exam
                        ));
                    }
                }
            }
        }

        if daily_hours > plan.constraints.max_hours_per_day + 1e-4 {
            errors.push(format!(
                "Il giorno {} ha {:.1}h > maxHoursPerDay ({:.1}h)",
                day.date, daily_hours, plan.constraints.max_hours_per_day
            ));
        }

        let mut quiz_courses = HashSet::new();
        for quiz in &day.quizzes {
            if !quiz_ids.insert(quiz.id.clone()) {
                errors.push(format!("ID quiz duplicato: {}", quiz.id));
            }
            if !quiz_courses.insert(quiz.course_id.clone()) {
                errors.push(format!(
                    "{}: più di un quiz giornaliero per la materia {}",
                    day.date, quiz.course_id
                ));
            }
            if quiz.questions.len() < 5 || quiz.questions.len() > 8 {
                errors.push(format!(
                    "Quiz {} ({}): attese 5-8 domande, trovate {}",
                    quiz.id,
                    day.date,
                    quiz.questions.len()
                ));
            }
            for q in &quiz.questions {
                if !question_ids.insert(q.id.clone()) {
                    errors.push(format!("ID domanda duplicato: {}", q.id));
                }
                if q.question_type == "multiple_choice" && q.options.len() != 4 {
                    errors.push(format!(
                        "Domanda {}: multiple_choice richiede esattamente 4 opzioni",
                        q.id
                    ));
                }
            }
        }

        for cid in &task_courses {
            if !quiz_courses.contains(cid) {
                errors.push(format!(
                    "{}: manca il quiz giornaliero per la materia {}",
                    day.date, cid
                ));
            }
        }
    }

    if errors.is_empty() {
        Ok(())
    } else {
        Err(errors)
    }
}
