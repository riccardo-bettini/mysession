use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::Duration;
use rusqlite::Connection;
use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Emitter};
use crate::plan::model::PlanPayload;
use crate::plan::validate::validate_plan;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PendingPlanNotification {
    pub plan_id: String,
    pub plan_name: String,
    pub revision: i64,
    pub file_path: String,
    pub is_new_session: bool,
}

pub fn scan_folder_for_pending_plans(
    conn: &Connection,
    folder_path: &str,
) -> Vec<PendingPlanNotification> {
    let mut results = Vec::new();
    let folder = Path::new(folder_path);
    if !folder.is_dir() {
        return results;
    }

    let entries = match fs::read_dir(folder) {
        Ok(e) => e,
        Err(_) => return results,
    };

    for entry in entries.flatten() {
        let path = entry.path();
        if path.extension().and_then(|ext| ext.to_str()) != Some("json") {
            continue;
        }

        // Limit file size to 5MB
        if let Ok(meta) = entry.metadata() {
            if meta.len() == 0 || meta.len() > 5 * 1024 * 1024 {
                continue;
            }
        }

        let content = match fs::read_to_string(&path) {
            Ok(c) => c,
            Err(_) => continue,
        };

        let plan: PlanPayload = match serde_json::from_str(&content) {
            Ok(p) => p,
            Err(_) => continue,
        };

        if validate_plan(&plan).is_err() {
            continue;
        }

        // Query current applied revision from plan_state
        let applied_rev: i64 = conn
            .query_row(
                "SELECT applied_revision FROM plan_state WHERE plan_id = ?1",
                rusqlite::params![&plan.plan_id],
                |row| row.get(0),
            )
            .unwrap_or(0);

        if plan.revision > applied_rev {
            let session_exists: bool = conn
                .query_row(
                    "SELECT 1 FROM academic_sessions WHERE plan_id = ?1 LIMIT 1",
                    rusqlite::params![&plan.plan_id],
                    |_| Ok(true),
                )
                .unwrap_or(false);

            results.push(PendingPlanNotification {
                plan_id: plan.plan_id.clone(),
                plan_name: plan.session.name.clone(),
                revision: plan.revision,
                file_path: path.to_string_lossy().to_string(),
                is_new_session: !session_exists,
            });
        }
    }

    results
}

pub struct PlanWatcherHandle {
    stop_signal: Arc<AtomicBool>,
}

impl PlanWatcherHandle {
    pub fn stop(&self) {
        self.stop_signal.store(true, Ordering::SeqCst);
    }
}

pub fn start_plan_watcher(app_handle: AppHandle, db_path: PathBuf) -> PlanWatcherHandle {
    let stop_signal = Arc::new(AtomicBool::new(false));
    let stop_clone = Arc::clone(&stop_signal);

    thread::spawn(move || {
        while !stop_clone.load(Ordering::SeqCst) {
            thread::sleep(Duration::from_secs(3));

            let conn = match Connection::open(&db_path) {
                Ok(c) => c,
                Err(_) => continue,
            };

            let folder_opt: Option<String> = conn
                .query_row(
                    "SELECT plan_folder FROM app_settings WHERE id = 1",
                    [],
                    |row| row.get(0),
                )
                .ok();

            if let Some(folder_path) = folder_opt {
                if !folder_path.trim().is_empty() {
                    let plans = scan_folder_for_pending_plans(&conn, &folder_path);
                    if !plans.is_empty() {
                        let _ = app_handle.emit("plan://pending", &plans);
                    }
                }
            }
        }
    });

    PlanWatcherHandle { stop_signal }
}
