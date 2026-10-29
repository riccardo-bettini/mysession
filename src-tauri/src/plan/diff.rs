use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use crate::plan::model::{PlanPayload, PlanTask};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PlanDiffSummary {
    pub is_initial_import: bool,
    pub plan_id: String,
    pub old_revision: i64,
    pub new_revision: i64,
    pub added_tasks: Vec<TaskChangeItem>,
    pub removed_tasks: Vec<TaskChangeItem>,
    pub moved_tasks: Vec<TaskMoveItem>,
    pub conflicts: Vec<TaskConflictItem>,
    pub exams_changed: Vec<ExamChangeItem>,
    pub courses_changed: Vec<CourseChangeItem>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TaskChangeItem {
    pub task_id: String,
    pub course_id: String,
    pub date: String,
    pub text: String,
    pub planned_hours: f64,
    pub kind: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TaskMoveItem {
    pub task_id: String,
    pub course_id: String,
    pub text: String,
    pub old_date: String,
    pub new_date: String,
    pub planned_hours: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TaskConflictItem {
    pub task_id: String,
    pub course_id: String,
    pub base_date: String,
    pub my_date: String,
    pub their_date: String,
    pub base_text: String,
    pub my_text: String,
    pub their_text: String,
    pub is_completed: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExamChangeItem {
    pub exam_id: String,
    pub course_id: String,
    pub title: String,
    pub old_date: Option<String>,
    pub new_date: String,
    pub old_status: Option<String>,
    pub new_status: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CourseChangeItem {
    pub course_id: String,
    pub sigla: String,
    pub name: String,
    pub is_added: bool,
    pub is_active_changed: bool,
    pub active: bool,
}

#[derive(Debug, Clone)]
pub struct DbTaskState {
    pub plan_task_id: String,
    pub date: String,
    pub text: String,
    pub done: bool,
    pub planned_hours: Option<f64>,
}

pub fn compute_plan_diff(
    base: Option<&PlanPayload>,
    theirs: &PlanPayload,
    mine_tasks: &[DbTaskState],
) -> PlanDiffSummary {
    let mut added_tasks = Vec::new();
    let mut removed_tasks = Vec::new();
    let mut moved_tasks = Vec::new();
    let mut conflicts = Vec::new();
    let mut exams_changed = Vec::new();
    let mut courses_changed = Vec::new();

    let is_initial = base.is_none();
    let old_rev = base.map(|b| b.revision).unwrap_or(0);
    let new_rev = theirs.revision;

    // Map tasks in `theirs`
    let mut their_tasks_by_id: HashMap<String, (&PlanTask, &str)> = HashMap::new();
    let mut their_alias_map: HashMap<String, String> = HashMap::new(); // alias -> canonical id
    for day in &theirs.days {
        for task in &day.tasks {
            their_tasks_by_id.insert(task.id.clone(), (task, &day.date));
            for alias in &task.aliases {
                their_alias_map.insert(alias.clone(), task.id.clone());
            }
        }
    }

    // Map tasks in `mine`
    let mut my_tasks_by_id: HashMap<String, &DbTaskState> = HashMap::new();
    for t in mine_tasks {
        my_tasks_by_id.insert(t.plan_task_id.clone(), t);
    }

    if let Some(base_plan) = base {
        // Map tasks in `base`
        let mut base_tasks_by_id: HashMap<String, (&PlanTask, &str)> = HashMap::new();
        for day in &base_plan.days {
            for task in &day.tasks {
                base_tasks_by_id.insert(task.id.clone(), (task, &day.date));
            }
        }

        // 1. Identify removed tasks (in base, not in theirs and not aliased)
        for (bid, (btask, bdate)) in &base_tasks_by_id {
            let still_exists = their_tasks_by_id.contains_key(bid) || their_alias_map.contains_key(bid);
            if !still_exists {
                removed_tasks.push(TaskChangeItem {
                    task_id: (*bid).clone(),
                    course_id: btask.course_id.clone(),
                    date: (*bdate).to_string(),
                    text: btask.text.clone(),
                    planned_hours: btask.planned_hours,
                    kind: btask.kind.clone(),
                });
            }
        }

        // 2. Compare tasks in theirs with base and mine
        for (tid, (ttask, tdate)) in &their_tasks_by_id {
            // Find in base (directly or via alias)
            let base_entry = base_tasks_by_id.get(tid).or_else(|| {
                ttask.aliases.iter().find_map(|a| base_tasks_by_id.get(a))
            });

            match base_entry {
                None => {
                    // Added task
                    added_tasks.push(TaskChangeItem {
                        task_id: (*tid).clone(),
                        course_id: ttask.course_id.clone(),
                        date: (*tdate).to_string(),
                        text: ttask.text.clone(),
                        planned_hours: ttask.planned_hours,
                        kind: ttask.kind.clone(),
                    });
                }
                Some((btask, bdate)) => {
                    // Check if mine has local changes
                    let mine_entry = my_tasks_by_id.get(tid).or_else(|| {
                        ttask.aliases.iter().find_map(|a| my_tasks_by_id.get(a))
                    });

                    let t_date_changed = *tdate != *bdate;
                    let t_text_changed = ttask.text != btask.text;

                    if let Some(mtask) = mine_entry {
                        let m_date_changed = mtask.date != *bdate;
                        let m_text_changed = mtask.text != btask.text;

                        if (m_date_changed && t_date_changed && mtask.date != *tdate)
                            || (m_text_changed && t_text_changed && mtask.text != ttask.text)
                        {
                            // Conflict!
                            conflicts.push(TaskConflictItem {
                                task_id: (*tid).clone(),
                                course_id: ttask.course_id.clone(),
                                base_date: (*bdate).to_string(),
                                my_date: mtask.date.clone(),
                                their_date: (*tdate).to_string(),
                                base_text: btask.text.clone(),
                                my_text: mtask.text.clone(),
                                their_text: ttask.text.clone(),
                                is_completed: mtask.done,
                            });
                        } else if t_date_changed && !m_date_changed {
                            // Clean move
                            moved_tasks.push(TaskMoveItem {
                                task_id: (*tid).clone(),
                                course_id: ttask.course_id.clone(),
                                text: ttask.text.clone(),
                                old_date: (*bdate).to_string(),
                                new_date: (*tdate).to_string(),
                                planned_hours: ttask.planned_hours,
                            });
                        }
                    } else if t_date_changed {
                        // Not in mine, moved in theirs
                        moved_tasks.push(TaskMoveItem {
                            task_id: (*tid).clone(),
                            course_id: ttask.course_id.clone(),
                            text: ttask.text.clone(),
                            old_date: (*bdate).to_string(),
                            new_date: (*tdate).to_string(),
                            planned_hours: ttask.planned_hours,
                        });
                    }
                }
            }
        }

        // Compare exams
        let base_exams: HashMap<String, &crate::plan::model::PlanExam> =
            base_plan.exams.iter().map(|e| (e.id.clone(), e)).collect();
        for te in &theirs.exams {
            if let Some(be) = base_exams.get(&te.id) {
                if te.date != be.date || te.status != be.status {
                    exams_changed.push(ExamChangeItem {
                        exam_id: te.id.clone(),
                        course_id: te.course_id.clone(),
                        title: te.title.clone(),
                        old_date: Some(be.date.clone()),
                        new_date: te.date.clone(),
                        old_status: Some(be.status.clone()),
                        new_status: te.status.clone(),
                    });
                }
            } else {
                exams_changed.push(ExamChangeItem {
                    exam_id: te.id.clone(),
                    course_id: te.course_id.clone(),
                    title: te.title.clone(),
                    old_date: None,
                    new_date: te.date.clone(),
                    old_status: None,
                    new_status: te.status.clone(),
                });
            }
        }

        // Compare courses
        let base_courses: HashMap<String, &crate::plan::model::PlanCourse> =
            base_plan.courses.iter().map(|c| (c.id.clone(), c)).collect();
        for tc in &theirs.courses {
            if let Some(bc) = base_courses.get(&tc.id) {
                if tc.active != bc.active {
                    courses_changed.push(CourseChangeItem {
                        course_id: tc.id.clone(),
                        sigla: tc.sigla.clone(),
                        name: tc.name.clone(),
                        is_added: false,
                        is_active_changed: true,
                        active: tc.active,
                    });
                }
            } else {
                courses_changed.push(CourseChangeItem {
                    course_id: tc.id.clone(),
                    sigla: tc.sigla.clone(),
                    name: tc.name.clone(),
                    is_added: true,
                    is_active_changed: false,
                    active: tc.active,
                });
            }
        }
    } else {
        // Initial import: all tasks are added
        for day in &theirs.days {
            for task in &day.tasks {
                added_tasks.push(TaskChangeItem {
                    task_id: task.id.clone(),
                    course_id: task.course_id.clone(),
                    date: day.date.clone(),
                    text: task.text.clone(),
                    planned_hours: task.planned_hours,
                    kind: task.kind.clone(),
                });
            }
        }
        for te in &theirs.exams {
            exams_changed.push(ExamChangeItem {
                exam_id: te.id.clone(),
                course_id: te.course_id.clone(),
                title: te.title.clone(),
                old_date: None,
                new_date: te.date.clone(),
                old_status: None,
                new_status: te.status.clone(),
            });
        }
        for tc in &theirs.courses {
            courses_changed.push(CourseChangeItem {
                course_id: tc.id.clone(),
                sigla: tc.sigla.clone(),
                name: tc.name.clone(),
                is_added: true,
                is_active_changed: false,
                active: tc.active,
            });
        }
    }

    PlanDiffSummary {
        is_initial_import: is_initial,
        plan_id: theirs.plan_id.clone(),
        old_revision: old_rev,
        new_revision: new_rev,
        added_tasks,
        removed_tasks,
        moved_tasks,
        conflicts,
        exams_changed,
        courses_changed,
    }
}
