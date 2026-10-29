pub mod apply;
pub mod diff;
pub mod model;
pub mod validate;
pub mod watcher;

pub use apply::{apply_study_plan, ApplyResult};
pub use diff::{compute_plan_diff, DbTaskState, PlanDiffSummary, TaskConflictItem, TaskMoveItem, TaskChangeItem};
pub use model::{PlanPayload, SUPPORTED_SCHEMA_VERSION};
pub use validate::{validate_plan, ValidationError};
pub use watcher::{scan_folder_for_pending_plans, start_plan_watcher, PendingPlanNotification, PlanWatcherHandle};

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::DatabaseManager;
    use std::collections::HashMap;

    const SAMPLE_PLAN_JSON: &str = r##"{
      "schemaVersion": 1,
      "planId": "sessione-test",
      "revision": 1,
      "generatedAt": "2027-01-05T10:00:00+01:00",
      "session": {
        "name": "Sessione Test Invernale",
        "subtitle": "Ingegneria",
        "sessionType": "invernale",
        "academicYear": "2026/2027",
        "startDate": "2027-01-11",
        "endDate": "2027-02-12",
        "notes": ""
      },
      "constraints": {
        "maxHoursPerDay": 4.0,
        "studyWeekdays": [1, 2, 3, 4, 5],
        "offDates": ["2027-01-18"],
        "reviewDaysBeforeExam": 2,
        "timezone": "Europe/Rome"
      },
      "courses": [
        { "id": "test/ana", "sigla": "ANA", "name": "Analisi", "code": "01", "cfu": 6.0, "colorHex": "#3B82F6", "active": true }
      ],
      "exams": [
        { "id": "test/ana/appello-1", "courseId": "test/ana", "date": "2027-01-22", "round": "1", "status": "planned" }
      ],
      "units": [
        { "id": "test/ana/limiti", "courseId": "test/ana", "unitNumber": 1, "title": "Limiti", "estimatedHours": 3.0 }
      ],
      "days": [
        {
          "date": "2027-01-12",
          "tasks": [
            { "id": "test/ana/limiti/t1", "courseId": "test/ana", "unitId": "test/ana/limiti", "text": "Limiti: teoria", "plannedHours": 3.0, "kind": "study" }
          ]
        }
      ]
    }"##;

    #[test]
    fn test_plan_validation_and_diff_flow() {
        let plan: PlanPayload = serde_json::from_str(SAMPLE_PLAN_JSON).expect("Parse JSON fallito");
        let valid_res = validate_plan(&plan);
        assert!(valid_res.is_ok(), "Il piano di esempio deve essere valido: {:?}", valid_res);

        // Compute diff for initial import
        let diff = compute_plan_diff(None, &plan, &[]);
        assert!(diff.is_initial_import);
        assert_eq!(diff.added_tasks.len(), 1);
        assert_eq!(diff.added_tasks[0].task_id, "test/ana/limiti/t1");
    }

    #[test]
    fn test_plan_apply_and_conflict_resolution() {
        let db = DatabaseManager::init_in_memory().unwrap();
        let plan_v1: PlanPayload = serde_json::from_str(SAMPLE_PLAN_JSON).unwrap();

        // 1. Initial Apply
        let res = db.with_conn(|conn| {
            apply_study_plan(
                conn,
                &plan_v1,
                SAMPLE_PLAN_JSON,
                "dummy_path.json",
                &HashMap::new(),
                None,
            )
        }).unwrap();

        assert_eq!(res.applied_revision, 1);
        let session_id = res.session_id;

        // 2. Mark task as completed in SQLite and manually modify date
        db.with_conn(|conn| {
            let mut stmt = conn.prepare("SELECT id FROM todos WHERE plan_task_id = 'test/ana/limiti/t1'")?;
            let todo_id: String = stmt.query_row([], |r| r.get(0))?;
            conn.execute(
                "UPDATE todos SET done = 1, date = '2027-01-14' WHERE id = ?1",
                rusqlite::params![&todo_id],
            )?;
            Ok(())
        }).unwrap();

        // 3. Create plan v2 where incoming task date is moved to 2027-01-13 (conflict!)
        let mut plan_v2 = plan_v1.clone();
        plan_v2.revision = 2;
        plan_v2.days[0].date = "2027-01-13".to_string();

        let db_tasks = db.with_conn(|conn| {
            let mut stmt = conn.prepare("SELECT plan_task_id, date, text, done, planned_hours FROM todos WHERE session_id = ?1")?;
            let rows = stmt.query_map([&session_id], |r| {
                Ok(DbTaskState {
                    plan_task_id: r.get(0)?,
                    date: r.get(1)?,
                    text: r.get(2)?,
                    done: r.get::<_, i32>(3)? != 0,
                    planned_hours: r.get(4)?,
                })
            })?;
            let mut out = Vec::new();
            for r in rows {
                out.push(r?);
            }
            Ok(out)
        }).unwrap();

        let diff = compute_plan_diff(Some(&plan_v1), &plan_v2, &db_tasks);
        assert_eq!(diff.conflicts.len(), 1, "Deve rilevare il conflitto su date divergenti");
        assert_eq!(diff.conflicts[0].my_date, "2027-01-14");
        assert_eq!(diff.conflicts[0].their_date, "2027-01-13");
        assert!(diff.conflicts[0].is_completed);

        // 4. Apply v2 with "keep_mine" resolution
        let mut resolutions = HashMap::new();
        resolutions.insert("test/ana/limiti/t1".to_string(), "keep_mine".to_string());

        let res_v2 = db.with_conn(|conn| {
            apply_study_plan(
                conn,
                &plan_v2,
                "{}",
                "dummy_path.json",
                &resolutions,
                Some(&session_id),
            )
        }).unwrap();

        assert_eq!(res_v2.applied_revision, 2);

        // Verify task preserved user date 2027-01-14 and done = 1
        db.with_conn(|conn| {
            let (final_date, final_done): (String, i32) = conn.query_row(
                "SELECT date, done FROM todos WHERE plan_task_id = 'test/ana/limiti/t1'",
                [],
                |r| Ok((r.get(0)?, r.get(1)?)),
            )?;
            assert_eq!(final_date, "2027-01-14");
            assert_eq!(final_done, 1);

            // Verify auto-completion of study unit
            let unit_done: i32 = conn.query_row(
                "SELECT is_completed FROM study_units WHERE plan_unit_id = 'test/ana/limiti'",
                [],
                |r| r.get(0),
            )?;
            assert_eq!(unit_done, 1, "L'unità deve risultare completata in quanto tutti i task associati sono completati");
            Ok(())
        }).unwrap();
    }
}
