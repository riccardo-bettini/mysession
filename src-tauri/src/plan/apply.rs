use crate::db::DbResult;
use crate::plan::model::PlanPayload;
use chrono::Utc;
use rusqlite::{params, Connection, OptionalExtension};
use std::collections::{HashMap, HashSet};
use uuid::Uuid;

pub struct ApplyResult {
    pub session_id: String,
    pub applied_revision: i64,
    pub courses_synced: usize,
    pub units_synced: usize,
    pub exams_synced: usize,
    pub tasks_synced: usize,
    pub quizzes_synced: usize,
}

pub fn apply_study_plan(
    conn: &mut Connection,
    theirs: &PlanPayload,
    theirs_raw_json: &str,
    source_path: &str,
    conflict_resolutions: &HashMap<String, String>,
    target_session_id: Option<&str>,
) -> DbResult<ApplyResult> {
    let tx = conn.transaction()?;
    let now_iso = Utc::now().to_rfc3339();

    let weekdays_json =
        serde_json::to_string(&theirs.constraints.study_weekdays).unwrap_or_else(|_| "[1,2,3,4,5]".into());
    let off_dates_json =
        serde_json::to_string(&theirs.constraints.off_dates).unwrap_or_else(|_| "[]".into());

    // 1. Resolve or create Academic Session
    let session_id: String = if let Some(target_id) = target_session_id {
        tx.execute(
            r#"
            UPDATE academic_sessions
            SET plan_id = ?1,
                name = ?2,
                subtitle = ?3,
                session_type = ?4,
                academic_year = ?5,
                start_date = ?6,
                end_date = ?7,
                max_hours_per_day = ?8,
                study_weekdays_json = ?9,
                off_dates_json = ?10,
                review_days_before_exam = ?11,
                notes = ?12
            WHERE id = ?13
            "#,
            params![
                &theirs.plan_id,
                theirs.session.name.trim(),
                theirs.session.subtitle.trim(),
                theirs.session.session_type.trim(),
                theirs.session.academic_year.trim(),
                theirs.session.start_date.trim(),
                theirs.session.end_date.trim(),
                theirs.constraints.max_hours_per_day,
                &weekdays_json,
                &off_dates_json,
                theirs.constraints.review_days_before_exam,
                theirs.session.notes.trim(),
                target_id
            ],
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
                tx.execute(
                    r#"
                    UPDATE academic_sessions
                    SET name = ?1,
                        subtitle = ?2,
                        session_type = ?3,
                        academic_year = ?4,
                        start_date = ?5,
                        end_date = ?6,
                        max_hours_per_day = ?7,
                        study_weekdays_json = ?8,
                        off_dates_json = ?9,
                        review_days_before_exam = ?10,
                        notes = ?11
                    WHERE id = ?12
                    "#,
                    params![
                        theirs.session.name.trim(),
                        theirs.session.subtitle.trim(),
                        theirs.session.session_type.trim(),
                        theirs.session.academic_year.trim(),
                        theirs.session.start_date.trim(),
                        theirs.session.end_date.trim(),
                        theirs.constraints.max_hours_per_day,
                        &weekdays_json,
                        &off_dates_json,
                        theirs.constraints.review_days_before_exam,
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
                        id, plan_id, name, subtitle, session_type, academic_year,
                        start_date, end_date, max_hours_per_day, study_weekdays_json,
                        off_dates_json, review_days_before_exam, notes, created_at
                    ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14)
                    "#,
                    params![
                        &new_id,
                        &theirs.plan_id,
                        theirs.session.name.trim(),
                        theirs.session.subtitle.trim(),
                        theirs.session.session_type.trim(),
                        theirs.session.academic_year.trim(),
                        theirs.session.start_date.trim(),
                        theirs.session.end_date.trim(),
                        theirs.constraints.max_hours_per_day,
                        &weekdays_json,
                        &off_dates_json,
                        theirs.constraints.review_days_before_exam,
                        theirs.session.notes.trim(),
                        &now_iso
                    ],
                )?;
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
                        target_hours = ?7,
                        is_active = ?8,
                        notes = ?9,
                        plan_course_id = ?10,
                        sort_order = ?11
                    WHERE id = ?12
                    "#,
                    params![
                        course.name.trim(),
                        course.code.trim(),
                        course.cfu.max(0.0),
                        course.professor.trim(),
                        course.color_hex.trim(),
                        course.difficulty_weight.clamp(0.5, 2.5),
                        course.target_hours,
                        if course.active { 1 } else { 0 },
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
                        id, session_id, plan_course_id, sigla, name, code, cfu, professor,
                        color_hex, difficulty_weight, target_hours, is_active, notes, sort_order
                    ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14)
                    "#,
                    params![
                        &new_cid,
                        &session_id,
                        &course.id,
                        course.sigla.trim().to_uppercase(),
                        course.name.trim(),
                        course.code.trim(),
                        course.cfu.max(0.0),
                        course.professor.trim(),
                        course.color_hex.trim(),
                        course.difficulty_weight.clamp(0.5, 2.5),
                        course.target_hours,
                        if course.active { 1 } else { 0 },
                        course.notes.trim(),
                        idx as i32
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
        let Some(db_course_id) = plan_course_to_db_id.get(&unit.course_id) else {
            continue;
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
                        chapter = ?2,
                        title = ?3,
                        description = ?4,
                        source = ?5,
                        estimated_hours = ?6,
                        priority = ?7
                    WHERE id = ?8
                    "#,
                    params![
                        unit.unit_number,
                        unit.chapter.trim(),
                        unit.title.trim(),
                        unit.description.trim(),
                        unit.source.as_deref().unwrap_or("").trim(),
                        unit.estimated_hours.max(0.0),
                        unit.priority.unwrap_or(2),
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
                        id, course_id, session_id, plan_unit_id, unit_number, chapter,
                        title, description, source, estimated_hours, priority, is_completed, completed_at
                    ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, 0, NULL)
                    "#,
                    params![
                        &new_uid,
                        db_course_id,
                        &session_id,
                        &unit.id,
                        unit.unit_number,
                        unit.chapter.trim(),
                        unit.title.trim(),
                        unit.description.trim(),
                        unit.source.as_deref().unwrap_or("").trim(),
                        unit.estimated_hours.max(0.0),
                        unit.priority.unwrap_or(2),
                    ],
                )?;
                new_uid
            }
        };
        plan_unit_to_db_id.insert(unit.id.clone(), db_unit_id);
    }

    // 4. Sync Exams
    for exam in &theirs.exams {
        let Some(db_course_id) = plan_course_to_db_id.get(&exam.course_id) else {
            continue;
        };

        let existing_exam_id: Option<String> = tx
            .query_row(
                "SELECT id FROM exams WHERE session_id = ?1 AND (plan_exam_id = ?2 OR (course_id = ?3 AND round = ?4)) LIMIT 1",
                params![&session_id, &exam.id, db_course_id, &exam.round],
                |row| row.get(0),
            )
            .optional()?;

        match existing_exam_id {
            Some(eid) => {
                tx.execute(
                    r#"
                    UPDATE exams
                    SET plan_exam_id = ?1,
                        date = ?2,
                        time_slot = ?3,
                        duration_minutes = ?4,
                        round = ?5,
                        exam_type = ?6,
                        classroom = ?7,
                        title = ?8,
                        status = ?9
                    WHERE id = ?10
                    "#,
                    params![
                        &exam.id,
                        exam.date.trim(),
                        exam.time_slot.trim(),
                        exam.duration_minutes.max(30),
                        exam.round.trim(),
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
                        id, session_id, course_id, plan_exam_id, date, time_slot,
                        duration_minutes, round, exam_type, classroom, title, details, status, grade
                    ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, '', ?12, NULL)
                    "#,
                    params![
                        &new_eid,
                        &session_id,
                        db_course_id,
                        &exam.id,
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

    // 5. Sync Operational Plans & Blocks
    let mut existing_blocks_done: HashMap<String, bool> = HashMap::new();
    {
        let mut stmt = tx.prepare(
            "SELECT plan_block_id, done FROM operational_blocks WHERE session_id = ?1 AND plan_block_id IS NOT NULL",
        )?;
        let rows = stmt.query_map(params![&session_id], |row| {
            Ok((row.get::<_, String>(0)?, row.get::<_, i32>(1)? != 0))
        })?;
        for r in rows {
            let (bid, done) = r?;
            existing_blocks_done.insert(bid, done);
        }
    }

    tx.execute(
        "DELETE FROM daily_operational_plans WHERE session_id = ?1",
        params![&session_id],
    )?;
    tx.execute(
        "DELETE FROM operational_blocks WHERE session_id = ?1 AND plan_block_id IS NOT NULL",
        params![&session_id],
    )?;

    for day in &theirs.days {
        tx.execute(
            r#"
            INSERT INTO daily_operational_plans (session_id, date, daily_goal, strategy_note)
            VALUES (?1, ?2, ?3, ?4)
            "#,
            params![
                &session_id,
                &day.date,
                day.operational_plan.daily_goal.trim(),
                day.operational_plan.strategy_note.trim()
            ],
        )?;

        for block in &day.operational_plan.blocks {
            let db_course_id = plan_course_to_db_id.get(&block.course_id).cloned();
            let done = existing_blocks_done.get(&block.id).copied().unwrap_or(false);
            let new_bid = format!("opb_{}", Uuid::new_v4().simple());
            tx.execute(
                r#"
                INSERT INTO operational_blocks (
                    id, session_id, date, course_id, block_order, category, title,
                    chapter, topic, pages, source, duration_hours, instructions, done, plan_block_id
                ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15)
                "#,
                params![
                    &new_bid,
                    &session_id,
                    &day.date,
                    db_course_id,
                    block.order,
                    &block.category,
                    block.title.trim(),
                    block.chapter.trim(),
                    block.topic.trim(),
                    block.pages.trim(),
                    block.source.trim(),
                    block.duration_hours,
                    block.instructions.trim(),
                    if done { 1 } else { 0 },
                    &block.id
                ],
            )?;
        }
    }

    // 6. Sync Tasks & Subtasks
    struct ExistingTask {
        id: String,
        date: String,
        title: String,
        done: bool,
    }

    let mut existing_tasks: HashMap<String, ExistingTask> = HashMap::new();
    {
        let mut stmt = tx.prepare(
            "SELECT id, plan_task_id, date, title, done FROM tasks WHERE session_id = ?1 AND plan_task_id IS NOT NULL",
        )?;
        let rows = stmt.query_map(params![&session_id], |row| {
            Ok((
                row.get::<_, String>(1)?,
                ExistingTask {
                    id: row.get(0)?,
                    date: row.get(2)?,
                    title: row.get(3)?,
                    done: row.get::<_, i32>(4)? != 0,
                },
            ))
        })?;
        for r in rows {
            let (ptid, item) = r?;
            existing_tasks.insert(ptid, item);
        }
    }

    let mut touched_plan_task_ids: HashSet<String> = HashSet::new();

    for day in &theirs.days {
        for task in &day.tasks {
            touched_plan_task_ids.insert(task.id.clone());
            let db_course_id = plan_course_to_db_id.get(&task.course_id);
            let db_unit_id = task
                .unit_id
                .as_ref()
                .and_then(|uid| plan_unit_to_db_id.get(uid));

            let existing = existing_tasks
                .get(&task.id)
                .or_else(|| task.aliases.iter().find_map(|a| existing_tasks.get(a)));

            let resolution = conflict_resolutions
                .get(&task.id)
                .map(String::as_str)
                .unwrap_or("use_theirs");

            let (target_date, target_title, target_done) = match existing {
                Some(ext) => {
                    if resolution == "keep_mine" {
                        (ext.date.clone(), ext.title.clone(), ext.done)
                    } else {
                        (day.date.clone(), task.title.clone(), ext.done)
                    }
                }
                None => (day.date.clone(), task.title.clone(), false),
            };

            let effective_task_id = match existing {
                Some(ext) => {
                    tx.execute(
                        r#"
                        UPDATE tasks
                        SET plan_task_id = ?1,
                            course_id = ?2,
                            unit_id = ?3,
                            date = ?4,
                            category = ?5,
                            title = ?6,
                            chapter = ?7,
                            topic = ?8,
                            pages = ?9,
                            source = ?10,
                            planned_hours = ?11,
                            done = ?12,
                            priority = ?13
                        WHERE id = ?14
                        "#,
                        params![
                            &task.id,
                            db_course_id,
                            db_unit_id,
                            &target_date,
                            &task.category,
                            &target_title,
                            task.chapter.trim(),
                            task.topic.trim(),
                            task.pages.trim(),
                            task.source.as_deref().unwrap_or("").trim(),
                            task.planned_hours,
                            if target_done { 1 } else { 0 },
                            task.priority.unwrap_or(2),
                            &ext.id
                        ],
                    )?;
                    ext.id.clone()
                }
                None => {
                    let new_tid = format!("tsk_{}", Uuid::new_v4().simple());
                    tx.execute(
                        r#"
                        INSERT INTO tasks (
                            id, session_id, course_id, unit_id, date, category, title,
                            chapter, topic, pages, source, planned_hours, done, priority,
                            plan_task_id, created_at
                        ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16)
                        "#,
                        params![
                            &new_tid,
                            &session_id,
                            db_course_id,
                            db_unit_id,
                            &target_date,
                            &task.category,
                            &target_title,
                            task.chapter.trim(),
                            task.topic.trim(),
                            task.pages.trim(),
                            task.source.as_deref().unwrap_or("").trim(),
                            task.planned_hours,
                            if target_done { 1 } else { 0 },
                            task.priority.unwrap_or(2),
                            &task.id,
                            &now_iso
                        ],
                    )?;
                    new_tid
                }
            };

            // Sync subtasks
            let mut existing_subs: HashMap<String, (String, bool)> = HashMap::new();
            {
                let mut sstmt = tx.prepare(
                    "SELECT id, plan_subtask_id, done FROM task_subitems WHERE task_id = ?1 AND plan_subtask_id IS NOT NULL",
                )?;
                let srows = sstmt.query_map(params![&effective_task_id], |row| {
                    Ok((
                        row.get::<_, String>(1)?,
                        (row.get::<_, String>(0)?, row.get::<_, i32>(2)? != 0),
                    ))
                })?;
                for sr in srows {
                    let (psid, pair) = sr?;
                    existing_subs.insert(psid, pair);
                }
            }

            let mut touched_subs = HashSet::new();
            for (idx, sub) in task.subtasks.iter().enumerate() {
                touched_subs.insert(sub.id.clone());
                let ext_sub = existing_subs
                    .get(&sub.id)
                    .or_else(|| sub.aliases.iter().find_map(|a| existing_subs.get(a)));
                match ext_sub {
                    Some((sub_db_id, _)) => {
                        tx.execute(
                            r#"
                            UPDATE task_subitems
                            SET plan_subtask_id = ?1, text = ?2, category = ?3, pages = ?4, sort_order = ?5
                            WHERE id = ?6
                            "#,
                            params![
                                &sub.id,
                                &sub.text,
                                sub.category.trim(),
                                sub.pages.trim(),
                                idx as i32,
                                sub_db_id
                            ],
                        )?;
                    }
                    None => {
                        let new_sub_id = format!("sub_{}", Uuid::new_v4().simple());
                        tx.execute(
                            r#"
                            INSERT INTO task_subitems (id, task_id, text, category, pages, done, sort_order, plan_subtask_id)
                            VALUES (?1, ?2, ?3, ?4, ?5, 0, ?6, ?7)
                            "#,
                            params![
                                &new_sub_id,
                                &effective_task_id,
                                &sub.text,
                                sub.category.trim(),
                                sub.pages.trim(),
                                idx as i32,
                                &sub.id
                            ],
                        )?;
                    }
                }
            }

            for (psid, (sub_db_id, done)) in existing_subs {
                if !touched_subs.contains(&psid) {
                    if done {
                        tx.execute(
                            "UPDATE task_subitems SET plan_subtask_id = NULL WHERE id = ?1",
                            params![sub_db_id],
                        )?;
                    } else {
                        tx.execute("DELETE FROM task_subitems WHERE id = ?1", params![sub_db_id])?;
                    }
                }
            }
        }
    }

    for (ptid, ext) in existing_tasks {
        if !touched_plan_task_ids.contains(&ptid) {
            if ext.done {
                tx.execute(
                    "UPDATE tasks SET plan_task_id = NULL WHERE id = ?1",
                    params![&ext.id],
                )?;
            } else {
                tx.execute("DELETE FROM tasks WHERE id = ?1", params![&ext.id])?;
            }
        }
    }

    // 7. Sync Daily Quizzes & Questions (preserving quiz_attempts!)
    let mut existing_quizzes: HashMap<String, String> = HashMap::new();
    {
        let mut qstmt = tx.prepare(
            "SELECT id, plan_quiz_id FROM daily_quizzes WHERE session_id = ?1 AND plan_quiz_id IS NOT NULL",
        )?;
        let qrows = qstmt.query_map(params![&session_id], |row| {
            Ok((row.get::<_, String>(1)?, row.get::<_, String>(0)?))
        })?;
        for qr in qrows {
            let (pqid, db_qid) = qr?;
            existing_quizzes.insert(pqid, db_qid);
        }
    }

    let mut touched_quizzes = HashSet::new();
    let mut quizzes_synced = 0usize;

    for day in &theirs.days {
        for quiz in &day.quizzes {
            let Some(db_course_id) = plan_course_to_db_id.get(&quiz.course_id) else {
                continue;
            };
            touched_quizzes.insert(quiz.id.clone());
            quizzes_synced += 1;

            let existing_qid = existing_quizzes
                .get(&quiz.id)
                .or_else(|| quiz.aliases.iter().find_map(|a| existing_quizzes.get(a)));

            let db_quiz_id = match existing_qid {
                Some(qid) => {
                    tx.execute(
                        r#"
                        UPDATE daily_quizzes
                        SET plan_quiz_id = ?1, course_id = ?2, date = ?3, title = ?4, topic_summary = ?5
                        WHERE id = ?6
                        "#,
                        params![
                            &quiz.id,
                            db_course_id,
                            &day.date,
                            quiz.title.trim(),
                            quiz.topic_summary.trim(),
                            qid
                        ],
                    )?;
                    qid.clone()
                }
                None => {
                    let new_qid = format!("quiz_{}", Uuid::new_v4().simple());
                    tx.execute(
                        r#"
                        INSERT INTO daily_quizzes (id, session_id, course_id, date, title, topic_summary, plan_quiz_id)
                        VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)
                        "#,
                        params![
                            &new_qid,
                            &session_id,
                            db_course_id,
                            &day.date,
                            quiz.title.trim(),
                            quiz.topic_summary.trim(),
                            &quiz.id
                        ],
                    )?;
                    new_qid
                }
            };

            // Replace questions for this quiz cleanly
            tx.execute(
                "DELETE FROM quiz_questions WHERE quiz_id = ?1",
                params![&db_quiz_id],
            )?;

            for (qidx, q) in quiz.questions.iter().enumerate() {
                let new_qqid = format!("qq_{}", Uuid::new_v4().simple());
                let options_json = serde_json::to_string(&q.options).unwrap_or_else(|_| "[]".into());
                tx.execute(
                    r#"
                    INSERT INTO quiz_questions (
                        id, quiz_id, question_order, question_type, category, chapter,
                        topic, prompt, options_json, correct_index, explanation, plan_question_id
                    ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12)
                    "#,
                    params![
                        &new_qqid,
                        &db_quiz_id,
                        (qidx + 1) as i32,
                        &q.question_type,
                        &q.category,
                        q.chapter.trim(),
                        q.topic.trim(),
                        q.prompt.trim(),
                        &options_json,
                        q.correct_index,
                        q.explanation.trim(),
                        &q.id
                    ],
                )?;
            }
        }
    }

    for (pqid, db_qid) in existing_quizzes {
        if !touched_quizzes.contains(&pqid) {
            let has_attempts: bool = tx
                .query_row(
                    "SELECT 1 FROM quiz_attempts WHERE quiz_id = ?1 LIMIT 1",
                    params![&db_qid],
                    |_| Ok(true),
                )
                .unwrap_or(false);
            if has_attempts {
                tx.execute(
                    "UPDATE daily_quizzes SET plan_quiz_id = NULL WHERE id = ?1",
                    params![&db_qid],
                )?;
            } else {
                tx.execute("DELETE FROM daily_quizzes WHERE id = ?1", params![&db_qid])?;
            }
        }
    }

    // 8. Record in plan_state
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

    // 9. Auto-complete study units whose tasks are all done
    for unit in &theirs.units {
        if let Some(db_unit_id) = plan_unit_to_db_id.get(&unit.id) {
            let counts: (i32, i32) = tx
                .query_row(
                    "SELECT COUNT(*), SUM(CASE WHEN done = 1 THEN 1 ELSE 0 END) FROM tasks WHERE unit_id = ?1",
                    params![db_unit_id],
                    |r| Ok((r.get(0)?, r.get::<_, Option<i32>>(1)?.unwrap_or(0))),
                )
                .unwrap_or((0, 0));
            if counts.0 > 0 && counts.0 == counts.1 {
                tx.execute(
                    "UPDATE study_units SET is_completed = 1, completed_at = COALESCE(completed_at, ?1) WHERE id = ?2",
                    params![&now_iso, db_unit_id],
                )?;
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
        quizzes_synced,
    })
}
