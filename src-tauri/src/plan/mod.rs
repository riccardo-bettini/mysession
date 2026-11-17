pub mod apply;
pub mod diff;
pub mod model;
pub mod validate;
pub mod watcher;

pub use apply::{apply_study_plan, ApplyResult};
pub use diff::{
    compute_plan_diff, DbTaskState, PlanDiffSummary, TaskChangeItem, TaskConflictItem, TaskMoveItem,
};
pub use model::{PlanPayload, SUPPORTED_SCHEMA_VERSION};
pub use validate::validate_plan;
pub use watcher::{
    scan_folder_for_pending_plans, start_plan_watcher, PlanWatcherHandle,
};

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::DatabaseManager;
    use crate::models::SubmitQuizAttemptInput;
    use std::collections::HashMap;

    const SAMPLE_PLAN_V3_JSON: &str = r##"{
      "schemaVersion": 3,
      "planId": "sessione-test-v3",
      "revision": 1,
      "generatedAt": "2027-01-05T10:00:00+01:00",
      "session": {
        "name": "Sessione Test Invernale v3",
        "subtitle": "Ingegneria",
        "sessionType": "invernale",
        "academicYear": "2026/2027",
        "startDate": "2027-01-11",
        "endDate": "2027-02-12",
        "notes": ""
      },
      "constraints": {
        "maxHoursPerDay": 6.0,
        "studyWeekdays": [1, 2, 3, 4, 5],
        "offDates": ["2027-01-18"],
        "reviewDaysBeforeExam": 2,
        "timezone": "Europe/Rome"
      },
      "courses": [
        { "id": "test/ana", "sigla": "ANA", "name": "Analisi", "code": "01", "cfu": 6.0, "colorHex": "#7C5CFC", "active": true }
      ],
      "exams": [
        { "id": "test/ana/appello-1", "courseId": "test/ana", "date": "2027-01-22", "round": "1", "title": "ANA - 1 Appello", "status": "planned" }
      ],
      "units": [
        { "id": "test/ana/limiti", "courseId": "test/ana", "unitNumber": 1, "chapter": "Cap. 1", "title": "Limiti", "estimatedHours": 3.0 }
      ],
      "days": [
        {
          "date": "2027-01-12",
          "operationalPlan": {
            "dailyGoal": "Completare teoria sui limiti e verificare la comprensione.",
            "strategyNote": "Alterna 2h teoria e 1h dimostrazioni.",
            "blocks": [
              {
                "id": "test/op/2027-01-12/1",
                "order": 1,
                "courseId": "test/ana",
                "category": "teoria",
                "title": "Limiti e continuità",
                "chapter": "Cap. 1",
                "topic": "Definizione epsilon-delta",
                "pages": "pp. 1-24",
                "durationHours": 2.0,
                "instructions": "Leggi e schematizza."
              }
            ]
          },
          "tasks": [
            {
              "id": "test/ana/limiti/t1",
              "courseId": "test/ana",
              "unitId": "test/ana/limiti",
              "category": "teoria",
              "title": "Teoria sui limiti",
              "chapter": "Cap. 1",
              "topic": "Definizione epsilon-delta",
              "pages": "pp. 1-24",
              "plannedHours": 3.0,
              "priority": 1
            }
          ],
          "quizzes": [
            {
              "id": "test/ana/quiz/2027-01-12",
              "courseId": "test/ana",
              "title": "Quiz Giornaliero ANA",
              "topicSummary": "Limiti",
              "questions": [
                { "id": "q1", "type": "multiple_choice", "category": "teoria", "chapter": "Cap. 1", "topic": "Limiti", "prompt": "Domanda 1", "options": ["A", "B", "C", "D"], "correctIndex": 0, "explanation": "Spiegazione 1" },
                { "id": "q2", "type": "multiple_choice", "category": "esercizi", "chapter": "Cap. 1", "topic": "Limiti", "prompt": "Domanda 2", "options": ["A", "B", "C", "D"], "correctIndex": 1, "explanation": "Spiegazione 2" },
                { "id": "q3", "type": "multiple_choice", "category": "teoria", "chapter": "Cap. 1", "topic": "Limiti", "prompt": "Domanda 3", "options": ["A", "B", "C", "D"], "correctIndex": 2, "explanation": "Spiegazione 3" },
                { "id": "q4", "type": "multiple_choice", "category": "temi_esame", "chapter": "Cap. 1", "topic": "Limiti", "prompt": "Domanda 4", "options": ["A", "B", "C", "D"], "correctIndex": 3, "explanation": "Spiegazione 4" },
                { "id": "q5", "type": "open_flashcard", "category": "dimostrazioni", "chapter": "Cap. 1", "topic": "Carabinieri", "prompt": "Dimostra", "explanation": "Passaggi completi" }
              ]
            }
          ]
        }
      ]
    }"##;

    #[test]
    fn test_plan_v3_validation_and_apply_with_quizzes_and_progress() {
        let db = DatabaseManager::init_in_memory().unwrap();
        let plan_v1: PlanPayload = serde_json::from_str(SAMPLE_PLAN_V3_JSON).unwrap();
        assert!(validate_plan(&plan_v1).is_ok());

        let diff = compute_plan_diff(None, &plan_v1, &[]);
        assert!(diff.is_initial_import);
        assert_eq!(diff.added_tasks.len(), 1);
        assert_eq!(diff.quizzes_count, 1);

        let res = db
            .with_conn(|conn| {
                apply_study_plan(
                    conn,
                    &plan_v1,
                    SAMPLE_PLAN_V3_JSON,
                    "dummy_v3.json",
                    &HashMap::new(),
                    None,
                )
            })
            .unwrap();

        assert_eq!(res.applied_revision, 1);
        assert_eq!(res.quizzes_synced, 1);
        let session_id = res.session_id;

        db.with_conn(|conn| {
            let settings = DatabaseManager::get_settings(conn)?;
            let snap = DatabaseManager::build_session_snapshot(conn, &session_id, &settings)?;
            assert_eq!(snap.operational_plans.len(), 1);
            assert_eq!(snap.tasks.len(), 1);
            assert_eq!(snap.tasks[0].category, "teoria");
            assert_eq!(snap.tasks[0].chapter, "Cap. 1");
            assert_eq!(snap.quizzes.len(), 1);
            assert_eq!(snap.quizzes[0].questions.len(), 5);

            // Completa la task e registra un tentativo quiz
            DatabaseManager::toggle_task(conn, &snap.tasks[0].id, true)?;
            DatabaseManager::submit_quiz_attempt(
                conn,
                &SubmitQuizAttemptInput {
                    quiz_id: snap.quizzes[0].id.clone(),
                    session_id: session_id.clone(),
                    course_id: snap.courses[0].id.clone(),
                    score_pct: 80.0,
                    correct_count: 4,
                    total_count: 5,
                    answers_json: "[]".into(),
                    wrong_question_ids: vec![snap.quizzes[0].questions[4].id.clone()],
                },
            )?;

            let snap2 = DatabaseManager::build_session_snapshot(conn, &session_id, &settings)?;
            assert_eq!(snap2.telemetry.tasks_completed, 1);
            assert_eq!(snap2.telemetry.quizzes_completed, 1);
            assert_eq!(snap2.telemetry.quiz_overall_avg_score, Some(80.0));
            assert_eq!(snap2.telemetry.overall_progress_pct, 100.0);
            Ok(())
        })
        .unwrap();
    }
}
