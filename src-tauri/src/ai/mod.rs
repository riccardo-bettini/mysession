pub mod catalog;
pub mod cli_bridge;
pub mod downloader;
pub mod hardware;
pub mod parser;
pub mod runtime;

use crate::db::DatabaseManager;
use crate::models::{
    AppSettings, ExtractedLessonItem, SessionSnapshot, UpsertCourseInput, UpsertExamInput,
    UpsertTaskInput,
};
use chrono::{Duration, Local, NaiveDate};
use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use std::sync::OnceLock;
use uuid::Uuid;

static GLOBAL_RUNTIME: OnceLock<runtime::AiRuntimeManager> = OnceLock::new();
static GLOBAL_DOWNLOADER: OnceLock<downloader::ModelDownloadManager> = OnceLock::new();

pub fn get_runtime_manager() -> &'static runtime::AiRuntimeManager {
    GLOBAL_RUNTIME.get_or_init(runtime::AiRuntimeManager::new)
}

pub fn get_download_manager() -> &'static downloader::ModelDownloadManager {
    GLOBAL_DOWNLOADER.get_or_init(downloader::ModelDownloadManager::new)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AiToolCallRecord {
    pub tool_name: String,
    pub summary: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AiAssistantResponse {
    pub reply_markdown: String,
    pub executed_tools: Vec<AiToolCallRecord>,
    pub undo_snapshot_id: Option<String>,
    pub navigate_to_date: Option<String>,
    pub navigate_to_section: Option<String>,
    pub updated_settings: Option<AppSettings>,
    pub updated_snapshot: Option<SessionSnapshot>,
    pub provider_used: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct LlmActionPayload {
    reply_markdown: String,
    #[serde(default)]
    actions: Vec<LlmToolAction>,
    #[serde(default)]
    navigate_to_date: Option<String>,
    #[serde(default)]
    navigate_to_section: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct LlmToolAction {
    tool: String,
    #[serde(default)]
    course_sigla: Option<String>,
    #[serde(default)]
    course_name: Option<String>,
    #[serde(default)]
    course_cfu: Option<f64>,
    #[serde(default)]
    date: Option<String>,
    #[serde(default)]
    from_date: Option<String>,
    #[serde(default)]
    to_date: Option<String>,
    #[serde(default)]
    category: Option<String>,
    #[serde(default)]
    title: Option<String>,
    #[serde(default)]
    chapter: Option<String>,
    #[serde(default)]
    topic: Option<String>,
    #[serde(default)]
    pages: Option<String>,
    #[serde(default)]
    planned_hours: Option<f64>,
    #[serde(default)]
    done: Option<bool>,
    #[serde(default)]
    round: Option<String>,
    #[serde(default)]
    time_slot: Option<String>,
    #[serde(default)]
    quiz_questions: Option<Vec<LlmQuizQuestionDraft>>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct LlmQuizQuestionDraft {
    #[serde(default)]
    question_type: Option<String>,
    #[serde(default)]
    category: Option<String>,
    #[serde(default)]
    chapter: Option<String>,
    #[serde(default)]
    topic: Option<String>,
    prompt: String,
    #[serde(default)]
    options: Vec<String>,
    #[serde(default)]
    correct_index: Option<i32>,
    #[serde(default)]
    explanation: String,
}

fn decode_base64_simple(input: &str) -> Result<Vec<u8>, String> {
    let clean: Vec<u8> = input
        .bytes()
        .filter(|b| !b.is_ascii_whitespace())
        .collect();
    if clean.len() % 4 != 0 {
        return Err("Base64 non valido".to_string());
    }
    let val = |b: u8| -> Result<u8, String> {
        match b {
            b'A'..=b'Z' => Ok(b - b'A'),
            b'a'..=b'z' => Ok(b - b'a' + 26),
            b'0'..=b'9' => Ok(b - b'0' + 52),
            b'+' => Ok(62),
            b'/' => Ok(63),
            _ => Err("Carattere base64 non valido".to_string()),
        }
    };
    let mut out = Vec::with_capacity((clean.len() / 4) * 3);
    for chunk in clean.chunks(4) {
        let a = val(chunk[0])?;
        let b = val(chunk[1])?;
        out.push((a << 2) | (b >> 4));
        if chunk[2] != b'=' {
            let c = val(chunk[2])?;
            out.push((b << 4) | (c >> 2));
            if chunk[3] != b'=' {
                let d = val(chunk[3])?;
                out.push((c << 6) | d);
            }
        }
    }
    Ok(out)
}

fn write_temp_image_if_present(
    image_base64: Option<&str>,
    mime_type: Option<&str>,
) -> Result<Option<PathBuf>, String> {
    let Some(raw) = image_base64.filter(|s| !s.trim().is_empty()) else {
        return Ok(None);
    };
    let clean_b64 = raw.split(";base64,").last().unwrap_or(raw);
    let bytes = decode_base64_simple(clean_b64)?;
    let ext = match mime_type.unwrap_or("image/png") {
        "image/jpeg" | "image/jpg" => "jpg",
        "image/webp" => "webp",
        _ => "png",
    };
    let path = std::env::temp_dir().join(format!("mysession_ai_{}.{}", Uuid::new_v4().simple(), ext));
    fs::write(&path, bytes).map_err(|e| format!("Impossibile scrivere immagine temporanea: {}", e))?;
    Ok(Some(path))
}

fn build_session_context_summary(snapshot: &SessionSnapshot) -> String {
    let today = &snapshot.telemetry.reference_today;
    let courses: Vec<String> = snapshot
        .courses
        .iter()
        .map(|c| format!("{} ({}, {} CFU)", c.sigla, c.name, c.cfu))
        .collect();
    let exams: Vec<String> = snapshot
        .exams
        .iter()
        .map(|e| format!("{} [{}]", e.title, e.date))
        .collect();
    let today_tasks: Vec<String> = snapshot
        .tasks
        .iter()
        .filter(|t| t.date == *today)
        .map(|t| {
            format!(
                "[{}] {} ({} · {} · {}h · done={})",
                t.category, t.title, t.chapter, t.topic, t.planned_hours, t.done
            )
        })
        .collect();

    format!(
        "Sessione attiva: {} ({} -> {}). Data oggi: {}.\n\
        Corsi: {:?}\nAppelli: {:?}\nTask di oggi: {:?}",
        snapshot.session.name,
        snapshot.session.start_date,
        snapshot.session.end_date,
        today,
        courses,
        exams,
        today_tasks
    )
}

pub fn process_natural_command(
    conn: &mut Connection,
    session_id: &str,
    prompt: &str,
    image_base64: Option<&str>,
    mime_type: Option<&str>,
) -> Result<AiAssistantResponse, String> {
    let clean_prompt = prompt.trim();
    if clean_prompt.is_empty() && image_base64.is_none() {
        return Err("Inserisci una domanda o un comando per l'Assistente AI.".to_string());
    }

    let settings = DatabaseManager::get_settings(conn).map_err(|e| e.to_string())?;
    let snapshot = DatabaseManager::build_session_snapshot(conn, session_id, &settings)
        .map_err(|e| e.to_string())?;

    let provider = settings.ai_provider.trim().to_lowercase();
    let context = build_session_context_summary(&snapshot);

    let system_prompt = format!(
        "Sei l'assistente AI accademico integrato in MySession v2.00.\n\
        Rispondi in italiano chiaro e conciso, restituendo ESCLUSIVAMENTE un JSON valido con la struttura:\n\
        {{\n\
          \"replyMarkdown\": \"risposta per lo studente in markdown\",\n\
          \"navigateToDate\": \"YYYY-MM-DD oppure null\",\n\
          \"navigateToSection\": \"home|calendar|today|courses|quiz|progress|ai oppure null\",\n\
          \"actions\": [\n\
            {{ \"tool\": \"upsert_task\", \"courseSigla\": \"MAM\", \"date\": \"YYYY-MM-DD\", \"category\": \"teoria|esercizi|temi_esame|dimostrazioni|domande_orale\", \"title\": \"...\", \"chapter\": \"...\", \"topic\": \"...\", \"pages\": \"...\", \"plannedHours\": 1.5 }},\n\
            {{ \"tool\": \"move_tasks_by_date\", \"fromDate\": \"YYYY-MM-DD\", \"toDate\": \"YYYY-MM-DD\", \"done\": false }},\n\
            {{ \"tool\": \"toggle_task\", \"title\": \"parte del titolo\", \"date\": \"YYYY-MM-DD\", \"done\": true }},\n\
            {{ \"tool\": \"upsert_exam\", \"courseSigla\": \"MAM\", \"date\": \"YYYY-MM-DD\", \"round\": \"1\", \"timeSlot\": \"09:00\" }},\n\
            {{ \"tool\": \"upsert_course\", \"courseSigla\": \"MAM\", \"courseName\": \"...\", \"courseCfu\": 10 }},\n\
            {{ \"tool\": \"create_quiz\", \"courseSigla\": \"MAM\", \"date\": \"YYYY-MM-DD\", \"title\": \"Quiz extra ...\", \"topic\": \"...\", \"quizQuestions\": [ {{ \"questionType\": \"multiple_choice\", \"category\": \"teoria\", \"chapter\": \"...\", \"topic\": \"...\", \"prompt\": \"...\", \"options\": [\"A\",\"B\",\"C\",\"D\"], \"correctIndex\": 0, \"explanation\": \"...\" }} ] }}\n\
          ]\n\
        }}\n\
        Se l'utente fa solo una domanda senza chiedere modifiche, lascia \"actions\": [].\n\n\
        CONTESTO SESSIONE:\n{}",
        context
    );

    if provider.starts_with("cli_") {
        let temp_img = write_temp_image_if_present(image_base64, mime_type)?;
        let full_prompt = format!("{}\n\nRICHIESTA UTENTE:\n{}", system_prompt, clean_prompt);
        let raw_res = cli_bridge::run_cli_agent(
            &provider,
            &settings.ai_custom_cli_command,
            &full_prompt,
            temp_img.as_deref(),
        );
        if let Some(ref p) = temp_img {
            let _ = fs::remove_file(p);
        }
        let raw_output = raw_res?;
        return parse_and_apply_llm_response(conn, session_id, &settings, &raw_output, &provider);
    }

    if provider == "local" {
        if let Ok(raw_output) =
            call_local_llama_server(&settings, &system_prompt, clean_prompt, image_base64, mime_type)
        {
            if let Ok(res) =
                parse_and_apply_llm_response(conn, session_id, &settings, &raw_output, "local")
            {
                return Ok(res);
            }
        }
    }

    execute_deterministic_fallback(conn, session_id, &settings, &snapshot, clean_prompt)
}

fn call_local_llama_server(
    settings: &AppSettings,
    system_prompt: &str,
    user_prompt: &str,
    image_base64: Option<&str>,
    mime_type: Option<&str>,
) -> Result<String, String> {
    let app_data_dir = DatabaseManager::resolve_app_data_dir();
    let runtime = get_runtime_manager();
    let port = runtime.ensure_started(
        &app_data_dir,
        &settings.ai_local_model_id,
        &settings.ai_hardware_preference_or_auto(),
        settings.ai_idle_timeout_minutes,
    )?;
    runtime.touch_activity();

    let user_content = if let Some(b64) = image_base64.filter(|s| !s.trim().is_empty()) {
        let clean_b64 = b64.split(";base64,").last().unwrap_or(b64);
        let mime = mime_type.unwrap_or("image/png");
        serde_json::json!([
            { "type": "text", "text": user_prompt },
            { "type": "image_url", "image_url": { "url": format!("data:{};base64,{}", mime, clean_b64) } }
        ])
    } else {
        serde_json::json!(user_prompt)
    };

    let body = serde_json::json!({
        "model": settings.ai_local_model_id,
        "temperature": 0.2,
        "response_format": { "type": "json_object" },
        "messages": [
            { "role": "system", "content": system_prompt },
            { "role": "user", "content": user_content }
        ]
    });

    let resp: serde_json::Value = downloader::get_http_agent()
        .post(&format!("http://127.0.0.1:{}/v1/chat/completions", port))
        .send_json(body)
        .map_err(|e| e.to_string())?
        .into_json()
        .map_err(|e| e.to_string())?;

    resp["choices"][0]["message"]["content"]
        .as_str()
        .map(|s| s.to_string())
        .ok_or_else(|| "Risposta modello locale vuota".to_string())
}

impl AppSettings {
    pub fn ai_hardware_preference_or_auto(&self) -> &str {
        "auto"
    }
}

fn parse_and_apply_llm_response(
    conn: &mut Connection,
    session_id: &str,
    settings: &AppSettings,
    raw_output: &str,
    provider_used: &str,
) -> Result<AiAssistantResponse, String> {
    let json_block = parser::extract_json_block(raw_output);
    let payload: LlmActionPayload = match serde_json::from_str(&json_block) {
        Ok(p) => p,
        Err(_) => {
            return Ok(AiAssistantResponse {
                reply_markdown: raw_output.to_string(),
                executed_tools: Vec::new(),
                undo_snapshot_id: None,
                navigate_to_date: None,
                navigate_to_section: None,
                updated_settings: None,
                updated_snapshot: None,
                provider_used: provider_used.to_string(),
            });
        }
    };

    let mut executed_tools = Vec::new();
    let mut undo_snapshot_id = None;

    if !payload.actions.is_empty() {
        undo_snapshot_id = DatabaseManager::save_ai_undo_snapshot(
            conn,
            session_id,
            &format!("Azione AI ({})", provider_used),
        )
        .ok();
    }

    for act in &payload.actions {
        if let Some(rec) = execute_single_tool_action(conn, session_id, act) {
            executed_tools.push(rec);
        }
    }

    let updated_snapshot = if !executed_tools.is_empty() {
        DatabaseManager::build_session_snapshot(conn, session_id, settings).ok()
    } else {
        None
    };

    Ok(AiAssistantResponse {
        reply_markdown: payload.reply_markdown,
        executed_tools,
        undo_snapshot_id,
        navigate_to_date: payload.navigate_to_date,
        navigate_to_section: payload.navigate_to_section,
        updated_settings: None,
        updated_snapshot,
        provider_used: provider_used.to_string(),
    })
}

fn find_course_id_by_sigla(conn: &Connection, session_id: &str, sigla: Option<&str>) -> Option<String> {
    let target = sigla?.trim().to_uppercase();
    if target.is_empty() {
        return None;
    }
    DatabaseManager::list_courses(conn, session_id)
        .ok()?
        .into_iter()
        .find(|c| c.sigla.to_uppercase() == target || c.name.to_uppercase().contains(&target))
        .map(|c| c.id)
}

fn execute_single_tool_action(
    conn: &Connection,
    session_id: &str,
    act: &LlmToolAction,
) -> Option<AiToolCallRecord> {
    let today = Local::now().date_naive().format("%Y-%m-%d").to_string();
    match act.tool.as_str() {
        "upsert_course" => {
            let sigla = act.course_sigla.as_deref().unwrap_or("NEW").to_uppercase();
            let name = act.course_name.clone().unwrap_or_else(|| sigla.clone());
            let cfu = act.course_cfu.unwrap_or(6.0);
            if DatabaseManager::upsert_course(
                conn,
                &UpsertCourseInput {
                    id: None,
                    session_id: session_id.to_string(),
                    sigla: sigla.clone(),
                    name: name.clone(),
                    code: None,
                    cfu,
                    professor: None,
                    color_hex: "#7C5CFC".to_string(),
                    difficulty_weight: Some(1.0),
                    target_hours: None,
                    is_active: Some(true),
                    notes: None,
                },
            )
            .is_ok()
            {
                return Some(AiToolCallRecord {
                    tool_name: "upsert_course".to_string(),
                    summary: format!("Aggiunto corso {} ({} CFU)", sigla, cfu),
                });
            }
        }
        "upsert_exam" => {
            let cid = find_course_id_by_sigla(conn, session_id, act.course_sigla.as_deref())?;
            let date = act.date.clone().unwrap_or_else(|| today.clone());
            let round = act.round.clone().unwrap_or_else(|| "1".to_string());
            if DatabaseManager::upsert_exam(
                conn,
                &UpsertExamInput {
                    id: None,
                    session_id: session_id.to_string(),
                    course_id: cid,
                    date: date.clone(),
                    time_slot: act.time_slot.clone(),
                    duration_minutes: Some(120),
                    round: round.clone(),
                    exam_type: Some("scritto_orale".to_string()),
                    classroom: None,
                    title: act.title.clone(),
                    details: None,
                    status: Some("planned".to_string()),
                    grade: None,
                },
            )
            .is_ok()
            {
                return Some(AiToolCallRecord {
                    tool_name: "upsert_exam".to_string(),
                    summary: format!("Appello salvato per il {}", date),
                });
            }
        }
        "upsert_task" => {
            let cid = find_course_id_by_sigla(conn, session_id, act.course_sigla.as_deref());
            let date = act.date.clone().unwrap_or_else(|| today.clone());
            let category = act.category.clone().unwrap_or_else(|| "teoria".to_string());
            let title = act.title.clone().unwrap_or_else(|| "Nuova attività di studio".to_string());
            if DatabaseManager::upsert_task(
                conn,
                &UpsertTaskInput {
                    id: None,
                    session_id: session_id.to_string(),
                    course_id: cid,
                    unit_id: None,
                    date: date.clone(),
                    category: category.clone(),
                    title: title.clone(),
                    chapter: act.chapter.clone(),
                    topic: act.topic.clone(),
                    pages: act.pages.clone(),
                    source: None,
                    planned_hours: act.planned_hours.or(Some(1.5)),
                    priority: Some(2),
                },
            )
            .is_ok()
            {
                return Some(AiToolCallRecord {
                    tool_name: "upsert_task".to_string(),
                    summary: format!("Task [{}] '{}' aggiunta il {}", category, title, date),
                });
            }
        }
        "move_tasks_by_date" => {
            let from_date = act.from_date.as_deref().unwrap_or(&today);
            let to_date = act.to_date.clone().unwrap_or_else(|| {
                NaiveDate::parse_from_str(from_date, "%Y-%m-%d")
                    .map(|d| (d + Duration::days(1)).format("%Y-%m-%d").to_string())
                    .unwrap_or_else(|_| today.clone())
            });
            let only_undone = act.done == Some(false);
            let sql = if only_undone {
                "UPDATE tasks SET date = ?1 WHERE session_id = ?2 AND date = ?3 AND done = 0"
            } else {
                "UPDATE tasks SET date = ?1 WHERE session_id = ?2 AND date = ?3"
            };
            if let Ok(changed) = conn.execute(sql, params![&to_date, session_id, from_date]) {
                return Some(AiToolCallRecord {
                    tool_name: "move_tasks_by_date".to_string(),
                    summary: format!("Spostate {} task dal {} al {}", changed, from_date, to_date),
                });
            }
        }
        "toggle_task" => {
            let needle = act.title.as_deref().unwrap_or("").to_lowercase();
            let target_done = act.done.unwrap_or(true);
            if let Ok(tasks) = DatabaseManager::list_tasks(conn, session_id) {
                if let Some(found) = tasks.into_iter().find(|t| {
                    (needle.is_empty() || t.title.to_lowercase().contains(&needle))
                        && act.date.as_deref().map(|d| d == t.date).unwrap_or(true)
                }) {
                    if DatabaseManager::toggle_task(conn, &found.id, target_done).is_ok() {
                        return Some(AiToolCallRecord {
                            tool_name: "toggle_task".to_string(),
                            summary: format!(
                                "Task '{}' segnata come {}",
                                found.title,
                                if target_done { "completata" } else { "da fare" }
                            ),
                        });
                    }
                }
            }
        }
        "create_quiz" => {
            let cid = find_course_id_by_sigla(conn, session_id, act.course_sigla.as_deref())?;
            let date = act.date.clone().unwrap_or_else(|| today.clone());
            let title = act.title.clone().unwrap_or_else(|| "Quiz AI".to_string());
            let topic = act.topic.clone().unwrap_or_default();
            let qid = format!("quiz_{}", Uuid::new_v4().simple());
            if conn
                .execute(
                    "INSERT INTO daily_quizzes (id, session_id, course_id, date, title, topic_summary, plan_quiz_id) VALUES (?1, ?2, ?3, ?4, ?5, ?6, NULL)",
                    params![&qid, session_id, &cid, &date, &title, &topic],
                )
                .is_ok()
            {
                if let Some(ref questions) = act.quiz_questions {
                    for (idx, q) in questions.iter().enumerate() {
                        let qqid = format!("qq_{}", Uuid::new_v4().simple());
                        let opts_json = serde_json::to_string(&q.options).unwrap_or_else(|_| "[]".into());
                        let _ = conn.execute(
                            r#"
                            INSERT INTO quiz_questions (
                                id, quiz_id, question_order, question_type, category, chapter,
                                topic, prompt, options_json, correct_index, explanation, plan_question_id
                            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, NULL)
                            "#,
                            params![
                                &qqid,
                                &qid,
                                (idx + 1) as i32,
                                q.question_type.as_deref().unwrap_or("multiple_choice"),
                                q.category.as_deref().unwrap_or("teoria"),
                                q.chapter.as_deref().unwrap_or(""),
                                q.topic.as_deref().unwrap_or(""),
                                &q.prompt,
                                &opts_json,
                                q.correct_index,
                                &q.explanation
                            ],
                        );
                    }
                }
                return Some(AiToolCallRecord {
                    tool_name: "create_quiz".to_string(),
                    summary: format!("Creato quiz '{}' per il {}", title, date),
                });
            }
        }
        _ => {}
    }
    None
}

fn execute_deterministic_fallback(
    conn: &mut Connection,
    session_id: &str,
    settings: &AppSettings,
    snapshot: &SessionSnapshot,
    prompt: &str,
) -> Result<AiAssistantResponse, String> {
    let lower = prompt.to_lowercase();
    let today = snapshot.telemetry.reference_today.clone();

    if lower.contains("sposta") && lower.contains("domani") {
        let tomorrow = NaiveDate::parse_from_str(&today, "%Y-%m-%d")
            .map(|d| (d + Duration::days(1)).format("%Y-%m-%d").to_string())
            .unwrap_or_else(|_| today.clone());
        let snap_id = DatabaseManager::save_ai_undo_snapshot(conn, session_id, "Sposta task non completate a domani").ok();
        let changed = conn
            .execute(
                "UPDATE tasks SET date = ?1 WHERE session_id = ?2 AND date = ?3 AND done = 0",
                params![&tomorrow, session_id, &today],
            )
            .map_err(|e| e.to_string())?;
        let updated = DatabaseManager::build_session_snapshot(conn, session_id, settings).ok();
        return Ok(AiAssistantResponse {
            reply_markdown: format!("Ho spostato **{} attività** non completate da oggi ({}) a domani ({}).", changed, today, tomorrow),
            executed_tools: vec![AiToolCallRecord {
                tool_name: "move_tasks_by_date".to_string(),
                summary: format!("Spostate {} task a {}", changed, tomorrow),
            }],
            undo_snapshot_id: snap_id,
            navigate_to_date: Some(tomorrow),
            navigate_to_section: Some("today".to_string()),
            updated_settings: None,
            updated_snapshot: updated,
            provider_used: "deterministic".to_string(),
        });
    }

    if lower.contains("oggi") || lower.contains("priorit") || lower.contains("fare") {
        let today_tasks: Vec<_> = snapshot.tasks.iter().filter(|t| t.date == today).collect();
        let undone: Vec<_> = today_tasks.iter().filter(|t| !t.done).collect();
        let mut msg = format!("### Riepilogo per Oggi ({})\n", today);
        if undone.is_empty() {
            msg.push_str("Nessuna task residua per oggi!");
        } else {
            for t in undone {
                msg.push_str(&format!("- **[{}]** {} ({} · {}h)\n", t.category, t.title, t.chapter, t.planned_hours));
            }
        }
        return Ok(AiAssistantResponse {
            reply_markdown: msg,
            executed_tools: Vec::new(),
            undo_snapshot_id: None,
            navigate_to_date: Some(today),
            navigate_to_section: Some("today".to_string()),
            updated_settings: None,
            updated_snapshot: None,
            provider_used: "deterministic".to_string(),
        });
    }

    let hint = if settings.ai_provider == "local" {
        "Il modello locale Qwen2.5-VL 3B non è ancora scaricato o avviato. Puoi scaricarlo nelle **Impostazioni -> AI** oppure collegare **Antigravity CLI / Claude Code / Codex**."
    } else {
        "Impossibile contattare il provider AI configurato. Verifica lo stato nelle Impostazioni."
    };

    Ok(AiAssistantResponse {
        reply_markdown: format!(
            "Sessione attiva **{}** · {} corsi · {}% completato ({}/{} task).\n\n{}",
            snapshot.session.name,
            snapshot.courses.len(),
            snapshot.telemetry.overall_progress_pct,
            snapshot.telemetry.tasks_completed,
            snapshot.telemetry.tasks_total,
            hint
        ),
        executed_tools: Vec::new(),
        undo_snapshot_id: None,
        navigate_to_date: None,
        navigate_to_section: None,
        updated_settings: None,
        updated_snapshot: None,
        provider_used: "deterministic".to_string(),
    })
}

pub fn parse_timetable_image_with_ai(
    settings: &AppSettings,
    image_base64: &str,
    mime_type: &str,
) -> Result<Vec<ExtractedLessonItem>, String> {
    let provider = settings.ai_provider.trim().to_lowercase();
    let clean_mime = if mime_type.is_empty() { "image/png" } else { mime_type };
    let clean_base64 = image_base64.split(";base64,").last().unwrap_or(image_base64);

    let prompt_text = "Sei un analizzatore OCR esperto per orari universitari.\n\
        Estrai TUTTE le lezioni settimanali presenti nell'immagine e rispondi ESCLUSIVAMENTE con JSON:\n\
        { \"lessons\": [ { \"dayOfWeek\": 1..7, \"startTime\": \"HH:MM\", \"endTime\": \"HH:MM\", \"courseName\": \"...\", \"classroom\": \"...\", \"professor\": \"...\" } ] }";

    if provider.starts_with("cli_") {
        let temp_img = write_temp_image_if_present(Some(clean_base64), Some(clean_mime))?
            .ok_or_else(|| "Immagine mancante".to_string())?;
        let res = cli_bridge::run_cli_agent(
            &provider,
            &settings.ai_custom_cli_command,
            prompt_text,
            Some(&temp_img),
        );
        let _ = fs::remove_file(&temp_img);
        let text = res?;
        return parser::clean_and_parse_lessons_json(&text);
    }

    if provider == "local" {
        let app_data_dir = DatabaseManager::resolve_app_data_dir();
        let runtime = get_runtime_manager();
        let port = runtime.ensure_started(
            &app_data_dir,
            &settings.ai_local_model_id,
            "auto",
            settings.ai_idle_timeout_minutes,
        )?;
        runtime.touch_activity();

        let data_url = format!("data:{};base64,{}", clean_mime, clean_base64);
        let body = serde_json::json!({
            "model": settings.ai_local_model_id,
            "temperature": 0.1,
            "max_tokens": 2048,
            "response_format": { "type": "json_object" },
            "messages": [
                { "role": "system", "content": "Estrai tutte le lezioni in JSON RFC 8259 con chiave 'lessons'." },
                { "role": "user", "content": [
                    { "type": "text", "text": prompt_text },
                    { "type": "image_url", "image_url": { "url": data_url } }
                ]}
            ]
        });

        let resp: serde_json::Value = downloader::get_http_agent()
            .post(&format!("http://127.0.0.1:{}/v1/chat/completions", port))
            .send_json(body)
            .map_err(|e| format!("Errore AI locale: {}", e))?
            .into_json()
            .map_err(|e| format!("Errore risposta AI locale: {}", e))?;

        let text = resp["choices"][0]["message"]["content"]
            .as_str()
            .ok_or_else(|| "Risposta AI locale vuota".to_string())?;

        return parser::clean_and_parse_lessons_json(text);
    }

    Err("Configura il modello locale base o un Agente CLI nelle Impostazioni AI.".to_string())
}

pub fn parse_timetable_prompt_with_ai(
    settings: &AppSettings,
    user_prompt: &str,
) -> Result<Vec<ExtractedLessonItem>, String> {
    let provider = settings.ai_provider.trim().to_lowercase();
    let prompt_text = format!(
        "Estrai tutte le lezioni settimanali da questo testo e rispondi ESCLUSIVAMENTE in JSON:\n\
        \"{}\"\n\n\
        Formato: {{ \"lessons\": [ {{ \"dayOfWeek\": 1..7, \"startTime\": \"HH:MM\", \"endTime\": \"HH:MM\", \"courseName\": \"...\", \"classroom\": \"...\", \"professor\": \"...\" }} ] }}",
        user_prompt.trim()
    );

    if provider.starts_with("cli_") {
        let text = cli_bridge::run_cli_agent(&provider, &settings.ai_custom_cli_command, &prompt_text, None)?;
        return parser::clean_and_parse_lessons_json(&text);
    }

    let app_data_dir = DatabaseManager::resolve_app_data_dir();
    let runtime = get_runtime_manager();
    let port = runtime.ensure_started(
        &app_data_dir,
        &settings.ai_local_model_id,
        "auto",
        settings.ai_idle_timeout_minutes,
    )?;
    runtime.touch_activity();

    let body = serde_json::json!({
        "model": settings.ai_local_model_id,
        "temperature": 0.1,
        "response_format": { "type": "json_object" },
        "messages": [
            { "role": "system", "content": "Estrai lezioni universitarie in JSON RFC 8259 con chiave 'lessons'." },
            { "role": "user", "content": prompt_text }
        ]
    });

    let resp: serde_json::Value = downloader::get_http_agent()
        .post(&format!("http://127.0.0.1:{}/v1/chat/completions", port))
        .send_json(body)
        .map_err(|e| format!("Errore AI locale: {}", e))?
        .into_json()
        .map_err(|e| format!("Errore risposta AI locale: {}", e))?;

    let text = resp["choices"][0]["message"]["content"]
        .as_str()
        .ok_or_else(|| "Risposta AI locale vuota".to_string())?;
    parser::clean_and_parse_lessons_json(text)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::UpsertSessionInput;

    #[test]
    fn test_clean_and_parse_lessons_json_resilience() {
        let ai_markdown = r#"
        ```json
        {
          "lessons": [
            { "giorno": "Lunedì", "ora_inizio": "8:30", "ora_fine": "10.30", "materia": "MAM", "aula": "BL28" },
            { "dayOfWeek": 3, "startTime": "14:00", "endTime": "16:00", "courseName": "TM", "classroom": "B8" }
          ]
        }
        ```
        "#;
        let parsed = parser::clean_and_parse_lessons_json(ai_markdown).unwrap();
        assert_eq!(parsed.len(), 2);
        assert_eq!(parsed[0].day_of_week, 1);
        assert_eq!(parsed[0].start_time, "08:30");
        assert_eq!(parsed[1].day_of_week, 3);
    }

    #[test]
    fn test_deterministic_move_and_undo_snapshot() {
        let db = DatabaseManager::init_in_memory().unwrap();
        let sess = db
            .with_conn(|conn| {
                DatabaseManager::upsert_session(
                    conn,
                    &UpsertSessionInput {
                        id: None,
                        name: "Sessione Test".into(),
                        subtitle: "".into(),
                        session_type: "invernale".into(),
                        academic_year: "2026/2027".into(),
                        start_date: "2026-11-01".into(),
                        end_date: "2027-02-28".into(),
                        max_hours_per_day: Some(6.0),
                        study_weekdays: None,
                        off_dates: None,
                        review_days_before_exam: Some(5),
                        notes: None,
                    },
                )
            })
            .unwrap();

        let today = Local::now().date_naive().format("%Y-%m-%d").to_string();
        db.with_conn(|conn| {
            DatabaseManager::upsert_task(
                conn,
                &UpsertTaskInput {
                    id: None,
                    session_id: sess.id.clone(),
                    course_id: None,
                    unit_id: None,
                    date: today.clone(),
                    category: "teoria".into(),
                    title: "Ripasso Capitolo 1".into(),
                    chapter: Some("Cap. 1".into()),
                    topic: Some("Cinematica".into()),
                    pages: Some("pp. 1-10".into()),
                    source: None,
                    planned_hours: Some(2.0),
                    priority: Some(1),
                },
            )?;
            Ok(())
        })
        .unwrap();

        let res = db
            .with_conn(|conn| {
                process_natural_command(conn, &sess.id, "Sposta le task non finite a domani", None, None)
                    .map_err(crate::db::DbError::Logic)
            })
            .unwrap();
        assert!(res.undo_snapshot_id.is_some());
        let tasks_after_move = db.with_conn(|c| DatabaseManager::list_tasks(c, &sess.id)).unwrap();
        assert_ne!(tasks_after_move[0].date, today);

        let snap_id = res.undo_snapshot_id.unwrap();
        db.with_conn(|conn| {
            DatabaseManager::restore_ai_undo_snapshot(conn, &snap_id)?;
            Ok(())
        })
        .unwrap();
        let tasks_restored = db.with_conn(|c| DatabaseManager::list_tasks(c, &sess.id)).unwrap();
        assert_eq!(tasks_restored[0].date, today);
    }
}
