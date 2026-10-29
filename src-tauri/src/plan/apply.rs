use std::collections::{HashMap, HashSet};
use chrono::Utc;
use rusqlite::{params, Connection, OptionalExtension};
use uuid::Uuid;
use crate::db::DbResult;
use crate::plan::model::PlanPayload;

pub struct ApplyResult {
    pub session_id: String,
    pub applied_revision: i64,
    pub courses_synced: usize,
    pub units_synced: usize,
    pub exams_synced: usize,
    pub tasks_synced: usize,
}

pub fn apply_study_plan(
    conn: &mut Connection,
    theirs: &PlanPayload,
    theirs_raw_json: &str,
    source_path: &str,
    conflict_resolutions: &HashMap<String, String>, // task_id -> "keep_mine" | "use_theirs"
    target_session_id: Option<&str>,
) -> DbResult<ApplyResult> {
    let tx = conn.transaction()?;
    let now_iso = Utc::now().to_rfc3339();

    // 1. Resolve or create academic session
    let session_id: String = if let Some(target_id) = target_session_id {
        tx.execute(
            "UPDATE academic_sessions SET plan_id = ?1 WHERE id = ?2",
            params![&theirs.plan_id, target_id],
        )?;
        target_id.to_string()
    } else {
        let existing_id: Option<String> = tx
            .query_row(
                "SELECT id FROM academic_sessions WHERE plan_id = ?1 LIMIT 1",
                params![&theirs.plan_id],
                |row| row.get(0),
            )
            .optional()?;

        match existing_id {
            Some(id) => {
                // Update session dates/name if needed
                tx.execute(
                    r#"
                    UPDATE academic_sessions
                    SET name = ?1,
                        subtitle = ?2,
                        session_type = ?3,
                        academic_year = ?4,
                        start_date = ?5,
                        end_date = ?6,
                        notes = ?7
                    WHERE id = ?8
                    "#,
                    params![
                        theirs.session.name.trim(),
                        theirs.session.subtitle.trim(),
                        theirs.session.session_type.trim(),
                        theirs.session.academic_year.trim(),
                        theirs.session.start_date.trim(),
                        theirs.session.end_date.trim(),
                        theirs.session.notes.trim(),
                        &id
                    ],
                )?;
                id
            }
            None => {
                let new_id = format!("sess_{}", Uuid::new_v4().simple());
                tx.execute(
                    r#"
                    INSERT INTO academic_sessions (
                        id, name, subtitle, session_type, academic_year,
                        start_date, end_date, notes, created_at, plan_id
                    ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)
                    "#,
                    params![
                        &new_id,
                        theirs.session.name.trim(),
                        theirs.session.subtitle.trim(),
                        theirs.session.session_type.trim(),
                        theirs.session.academic_year.trim(),
                        theirs.session.start_date.trim(),
                        theirs.session.end_date.trim(),
                        theirs.session.notes.trim(),
                        &now_iso,
                        &theirs.plan_id
                    ],
                )?;
                // Set as active session
                tx.execute(
                    "UPDATE app_settings SET active_session_id = ?1 WHERE id = 1",
                    params![&new_id],
                )?;
                new_id
            }
        }
    };

    // 2. Sync Courses
    let mut plan_course_to_db_id: HashMap<String, String> = HashMap::new();
    for (idx, course) in theirs.courses.iter().enumerate() {
        let existing_course_id: Option<String> = tx
            .query_row(
                "SELECT id FROM courses WHERE session_id = ?1 AND (plan_course_id = ?2 OR sigla = ?3) LIMIT 1",
                params![&session_id, &course.id, &course.sigla.to_uppercase()],
                |row| row.get(0),
            )
            .optional()?;

        let db_course_id = match existing_course_id {
            Some(id) => {
                tx.execute(
                    r#"
                    UPDATE courses
                    SET name = ?1,
                        code = ?2,
                        cfu = ?3,
                        professor = ?4,
                        color_hex = ?5,
                        difficulty_weight = ?6,
                        target_hours_override = ?7,
                        notes = ?8,
                        plan_course_id = ?9,
                        sort_order = ?10
                    WHERE id = ?11
                    "#,
                    params![
                        course.name.trim(),
                        course.code.trim(),
                        course.cfu.round().max(1.0) as i32,
                        course.professor.trim(),
                        course.color_hex.trim(),
                        course.difficulty_weight.clamp(0.5, 2.0),
                        course.target_hours,
                        course.notes.trim(),
                        &course.id,
                        idx as i32,
                        &id
                    ],
                )?;
                id
            }
            None => {
                let new_cid = format!("crs_{}", Uuid::new_v4().simple());
                tx.execute(
                    r#"
                    INSERT INTO courses (
                        id, session_id, sigla, name, code, cfu, professor,
                        color_hex, difficulty_weight, target_hours_override, notes,
                        sort_order, plan_course_id, is_visible
                    ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, 1)
                    "#,
                    params![
                        &new_cid,
                        &session_id,
                        course.sigla.trim().to_uppercase(),
                        course.name.trim(),
                        course.code.trim(),
                        course.cfu.round().max(1.0) as i32,
                        course.professor.trim(),
                        course.color_hex.trim(),
                        course.difficulty_weight.clamp(0.5, 2.0),
                        course.target_hours,
                        course.notes.trim(),
                        idx as i32,
                        &course.id
                    ],
                )?;
                new_cid
            }
        };
        plan_course_to_db_id.insert(course.id.clone(), db_course_id);
    }

    // 3. Sync Study Units
    let mut plan_unit_to_db_id: HashMap<String, String> = HashMap::new();
    for unit in &theirs.units {
        let db_course_id = match plan_course_to_db_id.get(&unit.course_id) {
            Some(id) => id,
            None => continue,
        };

        let existing_unit_id: Option<String> = tx
            .query_row(
                "SELECT id FROM study_units WHERE course_id = ?1 AND plan_unit_id = ?2 LIMIT 1",
                params![db_course_id, &unit.id],
                |row| row.get(0),
            )
            .optional()?;

        let db_unit_id = match existing_unit_id {
            Some(uid) => {
                tx.execute(
                    r#"
                    UPDATE study_units
                    SET unit_number = ?1,
                        title = ?2,
                        description = ?3,
                        estimated_hours = ?4
                    WHERE id = ?5
                    "#,
                    params![
                        unit.unit_number,
                        unit.title.trim(),
                        unit.description.trim(),
                        unit.estimated_hours.max(0.5),
                        &uid
                    ],
                )?;
                uid
            }
            None => {
                let new_uid = format!("unit_{}", Uuid::new_v4().simple());
                tx.execute(
                    r#"
                    INSERT INTO study_units (
                        id, course_id, session_id, unit_number, title, description,
                        estimated_hours, is_completed, assigned_date, completed_at, plan_unit_id
                    ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, 0, NULL, NULL, ?8)
                    "#,
                    params![
                        &new_uid,
                        db_course_id,
                        &session_id,
                        unit.unit_number,
                        unit.title.trim(),
                        unit.description.trim(),
                        unit.estimated_hours.max(0.5),
                        &unit.id
                    ],
                )?;
                new_uid
            }
        };
        plan_unit_to_db_id.insert(unit.id.clone(), db_unit_id);
    }

    // 4. Sync Exams
    for exam in &theirs.exams {
        let db_course_id = match plan_course_to_db_id.get(&exam.course_id) {
            Some(id) => id,
            None => continue,
        };

        let existing_exam_id: Option<String> = tx
            .query_row(
                "SELECT id FROM exams WHERE session_id = ?1 AND course_id = ?2 AND round = ?3 LIMIT 1",
                params![&session_id, db_course_id, &exam.round],
                |row| row.get(0),
            )
            .optional()?;

        match existing_exam_id {
            Some(eid) => {
                tx.execute(
                    r#"
                    UPDATE exams
                    SET date = ?1,
                        time_slot = ?2,
                        duration_minutes = ?3,
                        exam_type = ?4,
                        classroom = ?5,
                        title = ?6,
                        status = ?7
                    WHERE id = ?8
                    "#,
                    params![
                        exam.date.trim(),
                        exam.time_slot.trim(),
                        exam.duration_minutes.max(30),
                        exam.exam_type.trim(),
                        exam.classroom.trim(),
                        exam.title.trim(),
                        exam.status.trim(),
                        &eid
                    ],
                )?;
            }
            None => {
                let new_eid = format!("exam_{}", Uuid::new_v4().simple());
                tx.execute(
                    r#"
                    INSERT INTO exams (
                        id, session_id, course_id, date, time_slot, duration_minutes,
                        round, exam_type, classroom, weight_fraction, pair_id,
                        title, details, status, grade
                    ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, 1.0, NULL, ?10, '', ?11, NULL)
                    "#,
                    params![
                        &new_eid,
                        &session_id,
                        db_course_id,
                        exam.date.trim(),
                        exam.time_slot.trim(),
                        exam.duration_minutes.max(30),
                        exam.round.trim(),
                        exam.exam_type.trim(),
                        exam.classroom.trim(),
                        exam.title.trim(),
                        exam.status.trim()
                    ],
                )?;
            }
        }
    }

    // 5. Sync Tasks into `todos`
    // Map current DB todos with `plan_task_id`
    struct ExistingTodo {
        id: String,
        date: String,
        text: String,
        done: bool,
    }

    let mut existing_todos: HashMap<String, ExistingTodo> = HashMap::new();
    {
        let mut stmt = tx.prepare(
            "SELECT id, plan_task_id, date, text, done FROM todos WHERE session_id = ?1 AND plan_task_id IS NOT NULL",
        )?;
        let rows = stmt.query_map(params![&session_id], |row| {
            Ok((
                row.get::<_, String>(1)?,
                ExistingTodo {
                    id: row.get(0)?,
                    date: row.get(2)?,
                    text: row.get(3)?,
                    done: row.get::<_, i32>(4)? != 0,
                },
            ))
        })?;
        for r in rows {
            let (ptid, item) = r?;
            existing_todos.insert(ptid, item);
        }
    }

    let mut touched_plan_task_ids: HashSet<String> = HashSet::new();

    for day in &theirs.days {
        for task in &day.tasks {
            touched_plan_task_ids.insert(task.id.clone());
            let db_course_id = plan_course_to_db_id.get(&task.course_id);

            // Find existing todo either by id or by alias
            let existing = existing_todos.get(&task.id).or_else(|| {
                task.aliases.iter().find_map(|a| existing_todos.get(a))
            });

            let resolution = conflict_resolutions
                .get(&task.id)
                .map(String::as_str)
                .unwrap_or("use_theirs");

            let (target_date, target_text, target_done) = match existing {
                Some(ext) => {
                    if resolution == "keep_mine" {
                        (ext.date.clone(), ext.text.clone(), ext.done)
                    } else {
                        (day.date.clone(), task.text.clone(), ext.done)
                    }
                }
                None => (day.date.clone(), task.text.clone(), false),
            };

            match existing {
                Some(ext) => {
                    tx.execute(
                        r#"
                        UPDATE todos
                        SET plan_task_id = ?1,
                            course_id = ?2,
                            date = ?3,
                            text = ?4,
                            done = ?5,
                            priority = ?6,
                            planned_hours = ?7,
                            kind = ?8
                        WHERE id = ?9
                        "#,
                        params![
                            &task.id,
                            db_course_id,
                            &target_date,
                            &target_text,
                            if target_done { 1 } else { 0 },
                            task.priority.unwrap_or(0),
                            task.planned_hours,
                            &task.kind,
                            &ext.id
                        ],
                    )?;
                }
                None => {
                    let new_todo_id = format!("todo_{}", Uuid::new_v4().simple());
                    tx.execute(
                        r#"
                        INSERT INTO todos (
                            id, session_id, course_id, date, text, done,
                            priority, created_at, plan_task_id, planned_hours, kind
                        ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)
                        "#,
                        params![
                            &new_todo_id,
                            &session_id,
                            db_course_id,
                            &target_date,
                            &target_text,
                            if target_done { 1 } else { 0 },
                            task.priority.unwrap_or(0),
                            &now_iso,
                            &task.id,
                            task.planned_hours,
                            &task.kind
                        ],
                    )?;
                }
            }
        }
    }

    // Handle removed tasks: if not done, delete. If done, unlink plan_task_id.
    for (ptid, ext) in existing_todos {
        if !touched_plan_task_ids.contains(&ptid) {
            if ext.done {
                tx.execute(
                    "UPDATE todos SET plan_task_id = NULL WHERE id = ?1",
                    params![&ext.id],
                )?;
            } else {
                tx.execute("DELETE FROM todos WHERE id = ?1", params![&ext.id])?;
            }
        }
    }

    // 6. Record in `plan_state`
    tx.execute(
        r#"
        INSERT INTO plan_state (
            plan_id, session_id, applied_revision, base_json,
            pending_revision, source_path, updated_at
        ) VALUES (?1, ?2, ?3, ?4, NULL, ?5, ?6)
        ON CONFLICT(plan_id) DO UPDATE SET
            session_id = excluded.session_id,
            applied_revision = excluded.applied_revision,
            base_json = excluded.base_json,
            pending_revision = NULL,
            source_path = excluded.source_path,
            updated_at = excluded.updated_at
        "#,
        params![
            &theirs.plan_id,
            &session_id,
            theirs.revision,
            theirs_raw_json,
            source_path,
            &now_iso
        ],
    )?;

    // 7. Auto-completion of study units if all linked tasks are done
    for unit in &theirs.units {
        if let Some(db_unit_id) = plan_unit_to_db_id.get(&unit.id) {
            // Find all tasks linked to this unit in the plan
            let unit_task_ids: Vec<&str> = theirs
                .days
                .iter()
                .flat_map(|d| &d.tasks)
                .filter(|t| t.unit_id.as_deref() == Some(&unit.id))
                .map(|t| t.id.as_str())
                .collect();

            if !unit_task_ids.is_empty() {
                let mut all_done = true;
                for tid in &unit_task_ids {
                    let is_done: bool = tx
                        .query_row(
                            "SELECT done FROM todos WHERE plan_task_id = ?1 LIMIT 1",
                            params![tid],
                            |row| Ok(row.get::<_, i32>(0)? != 0),
                        )
                        .unwrap_or(false);
                    if !is_done {
                        all_done = false;
                        break;
                    }
                }
                if all_done {
                    tx.execute(
                        "UPDATE study_units SET is_completed = 1, completed_at = COALESCE(completed_at, ?1) WHERE id = ?2",
                        params![&now_iso, db_unit_id],
                    )?;
                }
            }
        }
    }

    tx.commit()?;

    Ok(ApplyResult {
        session_id,
        applied_revision: theirs.revision,
        courses_synced: theirs.courses.len(),
        units_synced: theirs.units.len(),
        exams_synced: theirs.exams.len(),
        tasks_synced: touched_plan_task_ids.len(),
    })
}
