pub mod catalog;
pub mod downloader;
pub mod hardware;
pub mod runtime;

use crate::db::DatabaseManager;
use crate::models::{
    AppSettings, SessionSnapshot, UpsertCourseInput, UpsertExamInput, UpsertTodoInput,
};
use chrono::{Datelike, Local, NaiveDate};
use rusqlite::Connection;
use serde::{Deserialize, Serialize};
use std::sync::OnceLock;

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
    pub navigate_to_date: Option<String>,
    pub updated_settings: Option<AppSettings>,
    pub updated_snapshot: Option<SessionSnapshot>,
    pub provider_used: String,
}

#[derive(Debug, Deserialize)]
struct LlmStructuredAction {
    reply_markdown: String,
    action_type: Option<String>,
    target_date: Option<String>,
    theme: Option<String>,
    backdrop_material: Option<String>,
    default_view: Option<String>,
    ui_scale: Option<i32>,
    course_sigla: Option<String>,
    course_name: Option<String>,
    course_cfu: Option<i32>,
    exam_round: Option<String>,
    exam_time: Option<String>,
    todo_text: Option<String>,
}

/// Punto di ingresso principale per l'Assistente AI Multimodale (Tastiera + Voce).
/// Supporta provider cloud/locali configurati dall'utente (Gemini, OpenAI, Ollama/LM Studio)
/// e il motore deterministico integrato a latenza zero (`local`).
pub fn process_natural_command(
    conn: &mut Connection,
    session_id: &str,
    prompt: &str,
) -> Result<AiAssistantResponse, String> {
    let clean_prompt = prompt.trim();
    if clean_prompt.is_empty() {
        return Err("Inserisci o detta un comando per l'Assistente AI.".to_string());
    }

    let settings = DatabaseManager::get_settings(conn).map_err(|e| e.to_string())?;
    let snapshot =
        DatabaseManager::build_session_snapshot(conn, session_id, &settings).map_err(|e| e.to_string())?;

    // Se l'utente ha configurato un provider LLM (gemini / openai / ollama / local con modello scaricato),
    // prova prima l'inferenza strutturata; in caso di fallimento o provider "local" non pronto,
    // esegue il parser deterministico di Intent & Tool Calling a latenza zero.
    let provider = settings.ai_provider.trim().to_lowercase();
    if provider == "gemini" || provider == "openai" || provider == "ollama" || provider == "local" {
        if let Ok(llm_action) = try_call_external_llm(&settings, &snapshot, clean_prompt) {
            return apply_structured_llm_action(
                conn,
                session_id,
                settings,
                llm_action,
                &provider,
            );
        }
    }

    execute_deterministic_intent_engine(conn, session_id, settings, snapshot, clean_prompt)
}

fn try_call_external_llm(
    settings: &AppSettings,
    snapshot: &SessionSnapshot,
    user_prompt: &str,
) -> Result<LlmStructuredAction, String> {
    let provider = settings.ai_provider.trim().to_lowercase();
    let courses_summary: Vec<String> = snapshot
        .courses
        .iter()
        .map(|c| format!("{} ({}, {} CFU, id={})", c.sigla, c.name, c.cfu, c.id))
        .collect();
    let exams_summary: Vec<String> = snapshot
        .exams
        .iter()
        .map(|e| format!("courseId={} data={} turno={} ora={}", e.course_id, e.date, e.round, e.time_slot))
        .collect();

    let system_instruction = format!(
        "Sei l'assistente AI integrato di MySession. Rispondi SOLO con un oggetto JSON valido con campi: \
        reply_markdown (stringa in italiano concisa), \
        action_type (null | \"set_theme\" | \"set_material\" | \"set_view\" | \"set_zoom\" | \"add_course\" | \"add_exam\" | \"add_todo\" | \"navigate_date\"), \
        target_date (YYYY-MM-DD opzionale), theme (\"dark\"|\"light\"|\"auto\"), backdrop_material (\"mica\"|\"acrylic\"|\"tabbed\"), \
        default_view (\"continuous\"|\"month\"|\"week\"|\"agenda\"), ui_scale (80..150), \
        course_sigla, course_name, course_cfu, exam_round, exam_time, todo_text. \
        Data odierna: {}. Corsi attivi: {:?}. Appelli: {:?}. Conflitti: {}.",
        snapshot.telemetry.reference_today,
        courses_summary,
        exams_summary,
        snapshot.conflicts.len()
    );

    let agent = crate::ai::downloader::get_http_agent();
    let raw_json = match provider.as_str() {
        "gemini" => {
            if settings.ai_api_key.trim().is_empty() {
                return Err("API Key Gemini non configurata".to_string());
            }
            let model = if settings.ai_model.trim().is_empty() {
                "gemini-3.8-flash"
            } else {
                settings.ai_model.trim()
            };
            let url = format!(
                "https://generativelanguage.googleapis.com/v1beta/models/{}:generateContent?key={}",
                model,
                settings.ai_api_key.trim()
            );
            let body = serde_json::json!({
                "systemInstruction": { "parts": [{ "text": system_instruction }] },
                "contents": [{ "parts": [{ "text": user_prompt }] }],
                "generationConfig": { "responseMimeType": "application/json" }
            });
            let resp: serde_json::Value = agent.post(&url)
                .send_json(body)
                .map_err(|e| e.to_string())?
                .into_json()
                .map_err(|e| e.to_string())?;
            resp["candidates"][0]["content"]["parts"][0]["text"]
                .as_str()
                .ok_or_else(|| "Risposta Gemini vuota".to_string())?
                .to_string()
        }
        "openai" => {
            if settings.ai_api_key.trim().is_empty() {
                return Err("API Key OpenAI non configurata".to_string());
            }
            let model = if settings.ai_model.trim().is_empty() {
                "gpt-4o-mini"
            } else {
                settings.ai_model.trim()
            };
            let body = serde_json::json!({
                "model": model,
                "response_format": { "type": "json_object" },
                "messages": [
                    { "role": "system", "content": system_instruction },
                    { "role": "user", "content": user_prompt }
                ]
            });
            let resp: serde_json::Value = agent.post("https://api.openai.com/v1/chat/completions")
                .set("Authorization", &format!("Bearer {}", settings.ai_api_key.trim()))
                .send_json(body)
                .map_err(|e| e.to_string())?
                .into_json()
                .map_err(|e| e.to_string())?;
            resp["choices"][0]["message"]["content"]
                .as_str()
                .ok_or_else(|| "Risposta OpenAI vuota".to_string())?
                .to_string()
        }
        "ollama" => {
            let base = settings.ai_endpoint_url.trim().trim_end_matches('/');
            let url = format!("{}/api/generate", base);
            let body = serde_json::json!({
                "model": settings.ai_model.trim(),
                "system": system_instruction,
                "prompt": user_prompt,
                "format": "json",
                "stream": false
            });
            let resp: serde_json::Value = agent.post(&url)
                .send_json(body)
                .map_err(|e| e.to_string())?
                .into_json()
                .map_err(|e| e.to_string())?;
            resp["response"]
                .as_str()
                .ok_or_else(|| "Risposta Ollama vuota".to_string())?
                .to_string()
        }
        "local" => {
            let app_data_dir = DatabaseManager::resolve_app_data_dir();
            let runtime = get_runtime_manager();
            let port = runtime
                .ensure_started(
                    &app_data_dir,
                    &settings.ai_local_model_id,
                    &settings.ai_hardware_preference,
                    settings.ai_idle_timeout_minutes,
                )
                .map_err(|e| e.to_string())?;
            runtime.touch_activity();

            let body = serde_json::json!({
                "model": settings.ai_local_model_id,
                "response_format": { "type": "json_object" },
                "messages": [
                    { "role": "system", "content": system_instruction },
                    { "role": "user", "content": user_prompt }
                ]
            });
            let resp: serde_json::Value = ureq::post(&format!("http://127.0.0.1:{}/v1/chat/completions", port))
                .send_json(body)
                .map_err(|e| e.to_string())?
                .into_json()
                .map_err(|e| e.to_string())?;
            resp["choices"][0]["message"]["content"]
                .as_str()
                .ok_or_else(|| "Risposta AI locale vuota".to_string())?
                .to_string()
        }
        _ => return Err("Provider non supportato per inferenza remota".to_string()),
    };

    serde_json::from_str::<LlmStructuredAction>(&raw_json).map_err(|e| e.to_string())
}

fn apply_structured_llm_action(
    conn: &mut Connection,
    session_id: &str,
    mut settings: AppSettings,
    action: LlmStructuredAction,
    provider_name: &str,
) -> Result<AiAssistantResponse, String> {
    let mut executed_tools = Vec::new();
    let mut navigate_to_date = action.target_date.clone();
    let mut settings_changed = false;

    if let Some(act) = action.action_type.as_deref() {
        match act {
            "set_theme" => {
                if let Some(t) = action.theme {
                    settings.theme = t.clone();
                    settings_changed = true;
                    executed_tools.push(AiToolCallRecord {
                        tool_name: "update_settings.theme".to_string(),
                        summary: format!("Tema impostato su '{}'", t),
                    });
                }
            }
            "set_material" => {
                if let Some(m) = action.backdrop_material {
                    settings.backdrop_material = m.clone();
                    settings_changed = true;
                    executed_tools.push(AiToolCallRecord {
                        tool_name: "update_settings.backdrop_material".to_string(),
                        summary: format!("Materiale DWM impostato su '{}'", m),
                    });
                }
            }
            "set_view" => {
                if let Some(v) = action.default_view {
                    settings.default_view = v.clone();
                    settings_changed = true;
                    executed_tools.push(AiToolCallRecord {
                        tool_name: "update_settings.default_view".to_string(),
                        summary: format!("Vista calendario impostata su '{}'", v),
                    });
                }
            }
            "set_zoom" => {
                if let Some(z) = action.ui_scale {
                    settings.ui_scale = z.clamp(80, 150);
                    settings_changed = true;
                    executed_tools.push(AiToolCallRecord {
                        tool_name: "update_settings.ui_scale".to_string(),
                        summary: format!("Zoom UI impostato a {}%", settings.ui_scale),
                    });
                }
            }
            "add_course" => {
                let sigla = action.course_sigla.unwrap_or_else(|| "CORSO".to_string());
                let name = action.course_name.unwrap_or_else(|| sigla.clone());
                let cfu = action.course_cfu.unwrap_or(9).clamp(1, 45);
                let _ = DatabaseManager::upsert_course(
                    conn,
                    &UpsertCourseInput {
                        id: None,
                        session_id: session_id.to_string(),
                        sigla: sigla.to_uppercase(),
                        name: name.clone(),
                        code: String::new(),
                        cfu,
                        professor: String::new(),
                        color_hex: "#007AFF".to_string(),
                        difficulty_weight: Some(1.0),
                        target_hours_override: None,
                        notes: None,
                        units: None,
                        plan_course_id: None,
                        is_visible: Some(true),
                    },
                )
                .map_err(|e| e.to_string())?;
                executed_tools.push(AiToolCallRecord {
                    tool_name: "upsert_course".to_string(),
                    summary: format!("Creato corso {} ({} CFU)", name, cfu),
                });
            }
            "add_exam" => {
                if let (Some(sigla), Some(date)) = (action.course_sigla, action.target_date.clone()) {
                    let snap = DatabaseManager::build_session_snapshot(conn, session_id, &settings)
                        .map_err(|e| e.to_string())?;
                    if let Some(course) = snap
                        .courses
                        .iter()
                        .find(|c| c.sigla.eq_ignore_ascii_case(&sigla) || c.name.to_lowercase().contains(&sigla.to_lowercase()))
                    {
                        let _ = DatabaseManager::upsert_exam(
                            conn,
                            &UpsertExamInput {
                                id: None,
                                session_id: session_id.to_string(),
                                course_id: course.id.clone(),
                                date: date.clone(),
                                time_slot: action.exam_time,
                                duration_minutes: Some(120),
                                round: action.exam_round.unwrap_or_else(|| "1".to_string()),
                                exam_type: Some("Scritto + Orale".to_string()),
                                classroom: None,
                                weight_fraction: Some(1.0),
                                pair_id: None,
                                title: None,
                                details: None,
                                status: Some("planned".to_string()),
                                grade: None,
                            },
                        )
                        .map_err(|e| e.to_string())?;
                        navigate_to_date = Some(date.clone());
                        executed_tools.push(AiToolCallRecord {
                            tool_name: "upsert_exam".to_string(),
                            summary: format!("Pianificato appello {} in data {}", course.sigla, date),
                        });
                    }
                }
            }
            "add_todo" => {
                let text = action.todo_text.unwrap_or_default();
                let date = action
                    .target_date
                    .clone()
                    .unwrap_or_else(|| Local::now().format("%Y-%m-%d").to_string());
                if !text.trim().is_empty() {
                    let _ = DatabaseManager::upsert_todo(
                        conn,
                        &UpsertTodoInput {
                            id: None,
                            session_id: session_id.to_string(),
                            course_id: None,
                            date: date.clone(),
                            text: text.clone(),
                            priority: Some(1),
                            plan_task_id: None,
                            planned_hours: None,
                            kind: None,
                        },
                    )
                    .map_err(|e| e.to_string())?;
                    navigate_to_date = Some(date.clone());
                    executed_tools.push(AiToolCallRecord {
                        tool_name: "upsert_todo".to_string(),
                        summary: format!("Aggiunto To-Do '{}' per il {}", text, date),
                    });
                }
            }
            _ => {}
        }
    }

    if settings_changed {
        settings = DatabaseManager::save_settings(conn, &settings).map_err(|e| e.to_string())?;
    }

    let updated_snapshot =
        DatabaseManager::build_session_snapshot(conn, session_id, &settings).map_err(|e| e.to_string())?;

    Ok(AiAssistantResponse {
        reply_markdown: action.reply_markdown,
        executed_tools,
        navigate_to_date,
        updated_settings: if settings_changed { Some(settings) } else { None },
        updated_snapshot: Some(updated_snapshot),
        provider_used: provider_name.to_string(),
    })
}

/// Motore deterministico di Intent Parsing & Tool Calling in lingua italiana (zero latenza, offline-ready).
fn execute_deterministic_intent_engine(
    conn: &mut Connection,
    session_id: &str,
    mut settings: AppSettings,
    snapshot: SessionSnapshot,
    prompt: &str,
) -> Result<AiAssistantResponse, String> {
    let lower = prompt.to_lowercase();
    let mut executed_tools: Vec<AiToolCallRecord> = Vec::new();
    let mut navigate_to_date: Option<String> = None;

    // ------------------------------------------------------------------------
    // 1. MUTAZIONI IMPOSTAZIONI VISIVE (Tema, Materiale DWM, Zoom, Vista)
    // ------------------------------------------------------------------------
    if lower.contains("tema scuro") || lower.contains("dark mode") || lower.contains("modalità scura") {
        settings.theme = "dark".to_string();
        executed_tools.push(AiToolCallRecord {
            tool_name: "set_theme_mode".to_string(),
            summary: "Tema impostato su Scuro (Dark Mode)".to_string(),
        });
        let saved = DatabaseManager::save_settings(conn, &settings).map_err(|e| e.to_string())?;
        return Ok(AiAssistantResponse {
            reply_markdown: "Ho attivato il **Tema Scuro** e sincronizzato il backdrop nativo Windows 11 DWM.".to_string(),
            executed_tools,
            navigate_to_date: None,
            updated_settings: Some(saved),
            updated_snapshot: Some(snapshot),
            provider_used: "local-engine".to_string(),
        });
    }

    if lower.contains("tema chiaro") || lower.contains("light mode") || lower.contains("modalità chiara") {
        settings.theme = "light".to_string();
        executed_tools.push(AiToolCallRecord {
            tool_name: "set_theme_mode".to_string(),
            summary: "Tema impostato su Chiaro (Light Mode)".to_string(),
        });
        let saved = DatabaseManager::save_settings(conn, &settings).map_err(|e| e.to_string())?;
        return Ok(AiAssistantResponse {
            reply_markdown: "Ho attivato il **Tema Chiaro** e aggiornato i materiali traslucidi.".to_string(),
            executed_tools,
            navigate_to_date: None,
            updated_settings: Some(saved),
            updated_snapshot: Some(snapshot),
            provider_used: "local-engine".to_string(),
        });
    }

    if lower.contains("tema auto") || lower.contains("tema di sistema") {
        settings.theme = "auto".to_string();
        executed_tools.push(AiToolCallRecord {
            tool_name: "set_theme_mode".to_string(),
            summary: "Tema sincronizzato con il Registro di Windows 11 (Auto)".to_string(),
        });
        let saved = DatabaseManager::save_settings(conn, &settings).map_err(|e| e.to_string())?;
        return Ok(AiAssistantResponse {
            reply_markdown: "Il tema è ora sincronizzato in **Automatico** con il registro di Windows 11.".to_string(),
            executed_tools,
            navigate_to_date: None,
            updated_settings: Some(saved),
            updated_snapshot: Some(snapshot),
            provider_used: "local-engine".to_string(),
        });
    }

    if lower.contains("acrylic") || lower.contains("acrilico") {
        settings.backdrop_material = "acrylic".to_string();
        let saved = DatabaseManager::save_settings(conn, &settings).map_err(|e| e.to_string())?;
        executed_tools.push(AiToolCallRecord {
            tool_name: "set_window_material".to_string(),
            summary: "Materiale DWM impostato su Acrylic".to_string(),
        });
        return Ok(AiAssistantResponse {
            reply_markdown: "Materiale finestra aggiornato a **Windows 11 Acrylic** (`DWMSBT_TRANSIENTWINDOW`).".to_string(),
            executed_tools,
            navigate_to_date: None,
            updated_settings: Some(saved),
            updated_snapshot: Some(snapshot),
            provider_used: "local-engine".to_string(),
        });
    }

    if lower.contains("mica") {
        settings.backdrop_material = "mica".to_string();
        let saved = DatabaseManager::save_settings(conn, &settings).map_err(|e| e.to_string())?;
        executed_tools.push(AiToolCallRecord {
            tool_name: "set_window_material".to_string(),
            summary: "Materiale DWM impostato su Mica".to_string(),
        });
        return Ok(AiAssistantResponse {
            reply_markdown: "Materiale finestra aggiornato a **Windows 11 Mica** (`DWMSBT_MAINWINDOW`).".to_string(),
            executed_tools,
            navigate_to_date: None,
            updated_settings: Some(saved),
            updated_snapshot: Some(snapshot),
            provider_used: "local-engine".to_string(),
        });
    }

    if lower.contains("zoom") || lower.contains("ingrandisci") || lower.contains("rimpicciolisci") {
        let target_scale = extract_number_in_range(&lower, 80, 150).unwrap_or_else(|| {
            if lower.contains("ingrandisci") || lower.contains("aumenta") {
                (settings.ui_scale + 10).clamp(80, 150)
            } else if lower.contains("rimpicciolisci") || lower.contains("riduci") {
                (settings.ui_scale - 10).clamp(80, 150)
            } else {
                100
            }
        });
        settings.ui_scale = target_scale;
        let saved = DatabaseManager::save_settings(conn, &settings).map_err(|e| e.to_string())?;
        executed_tools.push(AiToolCallRecord {
            tool_name: "set_ui_scale".to_string(),
            summary: format!("Zoom interfaccia impostato al {}%", target_scale),
        });
        return Ok(AiAssistantResponse {
            reply_markdown: format!("Ho regolato lo **Zoom dell'interfaccia al {}%**.", target_scale),
            executed_tools,
            navigate_to_date: None,
            updated_settings: Some(saved),
            updated_snapshot: Some(snapshot),
            provider_used: "local-engine".to_string(),
        });
    }

    if lower.contains("vista mese") || lower.contains("vista mensile") {
        settings.default_view = "month".to_string();
        let saved = DatabaseManager::save_settings(conn, &settings).map_err(|e| e.to_string())?;
        executed_tools.push(AiToolCallRecord {
            tool_name: "set_active_view".to_string(),
            summary: "Vista attiva: Mensile".to_string(),
        });
        return Ok(AiAssistantResponse {
            reply_markdown: "Sono passato alla **Vista Mensile**.".to_string(),
            executed_tools,
            navigate_to_date: None,
            updated_settings: Some(saved),
            updated_snapshot: Some(snapshot),
            provider_used: "local-engine".to_string(),
        });
    }

    if lower.contains("vista settimana") || lower.contains("vista settimanale") {
        settings.default_view = "week".to_string();
        let saved = DatabaseManager::save_settings(conn, &settings).map_err(|e| e.to_string())?;
        executed_tools.push(AiToolCallRecord {
            tool_name: "set_active_view".to_string(),
            summary: "Vista attiva: Settimanale".to_string(),
        });
        return Ok(AiAssistantResponse {
            reply_markdown: "Sono passato alla **Vista Settimanale**.".to_string(),
            executed_tools,
            navigate_to_date: None,
            updated_settings: Some(saved),
            updated_snapshot: Some(snapshot),
            provider_used: "local-engine".to_string(),
        });
    }

    if lower.contains("vista agenda") || lower.contains("agenda appelli") {
        settings.default_view = "agenda".to_string();
        let saved = DatabaseManager::save_settings(conn, &settings).map_err(|e| e.to_string())?;
        executed_tools.push(AiToolCallRecord {
            tool_name: "set_active_view".to_string(),
            summary: "Vista attiva: Agenda Appelli".to_string(),
        });
        return Ok(AiAssistantResponse {
            reply_markdown: "Sono passato alla **Vista Agenda Appelli**.".to_string(),
            executed_tools,
            navigate_to_date: None,
            updated_settings: Some(saved),
            updated_snapshot: Some(snapshot),
            provider_used: "local-engine".to_string(),
        });
    }

    if lower.contains("vista continua") || lower.contains("scorrimento infinito") {
        settings.default_view = "continuous".to_string();
        let saved = DatabaseManager::save_settings(conn, &settings).map_err(|e| e.to_string())?;
        executed_tools.push(AiToolCallRecord {
            tool_name: "set_active_view".to_string(),
            summary: "Vista attiva: Continua Infinita".to_string(),
        });
        return Ok(AiAssistantResponse {
            reply_markdown: "Sono passato alla **Vista Continua Infinita** virtualizzata.".to_string(),
            executed_tools,
            navigate_to_date: None,
            updated_settings: Some(saved),
            updated_snapshot: Some(snapshot),
            provider_used: "local-engine".to_string(),
        });
    }

    // ------------------------------------------------------------------------
    // 2. INTERROGAZIONI STATISTICHE (CFU per mese, Corso più critico, Conflitti, Countdown)
    // ------------------------------------------------------------------------
    if lower.contains("quanti cfu") || lower.contains("totale cfu") || lower.contains("crediti") {
        executed_tools.push(AiToolCallRecord {
            tool_name: "query_cfu_statistics".to_string(),
            summary: "Calcolo CFU pianificati per sessione e mese".to_string(),
        });

        if let Some((month_num, month_name)) = detect_italian_month(&lower) {
            let mut course_ids_in_month = std::collections::HashSet::new();
            for ex in &snapshot.exams {
                if let Ok(dt) = NaiveDate::parse_from_str(&ex.date, "%Y-%m-%d") {
                    if dt.month() == month_num {
                        course_ids_in_month.insert(ex.course_id.clone());
                    }
                }
            }
            let cfu_month: i32 = snapshot
                .courses
                .iter()
                .filter(|c| course_ids_in_month.contains(&c.id))
                .map(|c| c.cfu)
                .sum();

            return Ok(AiAssistantResponse {
                reply_markdown: format!(
                    "Nel mese di **{}** hai pianificato appelli per **{} CFU** ({} materie coinvolte su {} CFU totali di sessione).",
                    month_name,
                    cfu_month,
                    course_ids_in_month.len(),
                    snapshot.telemetry.total_cfu
                ),
                executed_tools,
                navigate_to_date: None,
                updated_settings: None,
                updated_snapshot: None,
                provider_used: "local-engine".to_string(),
            });
        }

        return Ok(AiAssistantResponse {
            reply_markdown: format!(
                "Nella sessione **{}** hai pianificato complessivamente **{} CFU** distribuiti su **{} materie** e **{} appelli** (avanzamento unità didattiche: **{}%**).",
                snapshot.session.name,
                snapshot.telemetry.total_cfu,
                snapshot.telemetry.courses_count,
                snapshot.telemetry.exams_count,
                snapshot.telemetry.overall_progress_pct.round()
            ),
            executed_tools,
            navigate_to_date: None,
            updated_settings: None,
            updated_snapshot: None,
            provider_used: "local-engine".to_string(),
        });
    }

    if lower.contains("critico") || lower.contains("difficile") || lower.contains("priorit") {
        executed_tools.push(AiToolCallRecord {
            tool_name: "query_critical_course".to_string(),
            summary: "Analisi indice di criticità Workload Engine".to_string(),
        });

        if let Some(top) = snapshot
            .telemetry
            .course_workloads
            .iter()
            .max_by(|a, b| a.criticality_score.partial_cmp(&b.criticality_score).unwrap_or(std::cmp::Ordering::Equal))
        {
            return Ok(AiAssistantResponse {
                reply_markdown: format!(
                    "Il corso più critico è **{} — {}** (**{} CFU**): restano **{:.1} ore** di studio stimate ({:.0}% completato), con un carico consigliato di **{:.1} h/giorno**.",
                    top.sigla,
                    top.name,
                    top.cfu,
                    top.remaining_study_hours,
                    top.completion_pct,
                    top.recommended_daily_hours
                ),
                executed_tools,
                navigate_to_date: top.next_exam_date.clone(),
                updated_settings: None,
                updated_snapshot: None,
                provider_used: "local-engine".to_string(),
            });
        } else {
            return Ok(AiAssistantResponse {
                reply_markdown: "Non ci sono ancora materie registrate nella sessione attiva per calcolare l'indice di criticità.".to_string(),
                executed_tools,
                navigate_to_date: None,
                updated_settings: None,
                updated_snapshot: None,
                provider_used: "local-engine".to_string(),
            });
        }
    }

    if lower.contains("conflitt") || lower.contains("collision") || lower.contains("sovrapposizion") {
        executed_tools.push(AiToolCallRecord {
            tool_name: "query_conflicts".to_string(),
            summary: "Ispezione Conflict Engine in tempo reale".to_string(),
        });

        if snapshot.conflicts.is_empty() {
            return Ok(AiAssistantResponse {
                reply_markdown: "**Nessun conflitto rilevato!** Tutti gli appelli rispettano il buffer minimo di preparazione calcolato sui CFU.".to_string(),
                executed_tools,
                navigate_to_date: None,
                updated_settings: None,
                updated_snapshot: None,
                provider_used: "local-engine".to_string(),
            });
        } else {
            let bullets: Vec<String> = snapshot
                .conflicts
                .iter()
                .map(|c| format!("- **{}** ({}): {}", c.title, c.date, c.description))
                .collect();
            return Ok(AiAssistantResponse {
                reply_markdown: format!(
                    "Ho rilevato **{} conflitti/criticità di buffer**:\n{}",
                    snapshot.conflicts.len(),
                    bullets.join("\n")
                ),
                executed_tools,
                navigate_to_date: snapshot.conflicts.first().map(|c| c.date.clone()),
                updated_settings: None,
                updated_snapshot: None,
                provider_used: "local-engine".to_string(),
            });
        }
    }

    // ------------------------------------------------------------------------
    // 3. AZIONI CRUD: CREAZIONE CORSO, APPELLO, TO-DO
    // ------------------------------------------------------------------------
    if (lower.starts_with("crea corso ") || lower.starts_with("aggiungi corso ") || lower.starts_with("nuovo corso "))
        && prompt.split_whitespace().count() >= 3
    {
        let tokens: Vec<&str> = prompt.split_whitespace().collect();
        let sigla = tokens.get(2).copied().unwrap_or("CORSO").to_uppercase();
        let cfu = extract_number_in_range(&lower, 1, 45).unwrap_or(9);
        let name = tokens
            .iter()
            .skip(3)
            .copied()
            .filter(|w| !w.to_lowercase().contains("cfu") && w.parse::<i32>().is_err())
            .collect::<Vec<&str>>()
            .join(" ");
        let final_name = if name.trim().is_empty() {
            sigla.clone()
        } else {
            name
        };

        let created = DatabaseManager::upsert_course(
            conn,
            &UpsertCourseInput {
                id: None,
                session_id: session_id.to_string(),
                sigla: sigla.clone(),
                name: final_name.clone(),
                code: String::new(),
                cfu,
                professor: String::new(),
                color_hex: "#007AFF".to_string(),
                difficulty_weight: Some(1.0),
                target_hours_override: None,
                notes: None,
                units: None,
                plan_course_id: None,
                is_visible: Some(true),
            },
        )
        .map_err(|e| e.to_string())?;

        executed_tools.push(AiToolCallRecord {
            tool_name: "upsert_course".to_string(),
            summary: format!("Creato corso {} — {} ({} CFU)", created.sigla, created.name, created.cfu),
        });

        let updated_snapshot =
            DatabaseManager::build_session_snapshot(conn, session_id, &settings).map_err(|e| e.to_string())?;

        return Ok(AiAssistantResponse {
            reply_markdown: format!(
                "Ho creato la materia **{} — {}** (**{} CFU**) nella sessione corrente.",
                created.sigla, created.name, created.cfu
            ),
            executed_tools,
            navigate_to_date: None,
            updated_settings: None,
            updated_snapshot: Some(updated_snapshot),
            provider_used: "local-engine".to_string(),
        });
    }

    if lower.contains("appello") && (lower.contains("aggiungi") || lower.contains("inserisci") || lower.contains("pianifica") || lower.contains("fissa")) {
        if let Some(target_date) = extract_iso_or_italian_date(&lower, &snapshot.telemetry.reference_today) {
            if let Some(course) = snapshot.courses.iter().find(|c| {
                lower.contains(&c.sigla.to_lowercase()) || lower.contains(&c.name.to_lowercase())
            }) {
                let round = if lower.contains("secondo") || lower.contains("2°") || lower.contains("2 ") {
                    "2"
                } else if lower.contains("terzo") || lower.contains("3°") {
                    "3"
                } else if lower.contains("itinere") || lower.contains("parziale") {
                    "itinere_1"
                } else {
                    "1"
                };

                let _ = DatabaseManager::upsert_exam(
                    conn,
                    &UpsertExamInput {
                        id: None,
                        session_id: session_id.to_string(),
                        course_id: course.id.clone(),
                        date: target_date.clone(),
                        time_slot: Some("09:00".to_string()),
                        duration_minutes: Some(120),
                        round: round.to_string(),
                        exam_type: Some("Scritto + Orale".to_string()),
                        classroom: None,
                        weight_fraction: Some(1.0),
                        pair_id: None,
                        title: None,
                        details: None,
                        status: Some("planned".to_string()),
                        grade: None,
                    },
                )
                .map_err(|e| e.to_string())?;

                executed_tools.push(AiToolCallRecord {
                    tool_name: "upsert_exam".to_string(),
                    summary: format!("Pianificato appello di {} il {}", course.sigla, target_date),
                });

                let updated_snapshot =
                    DatabaseManager::build_session_snapshot(conn, session_id, &settings).map_err(|e| e.to_string())?;

                return Ok(AiAssistantResponse {
                    reply_markdown: format!(
                        "Ho pianificato l'appello di **{} ({})** per il **{}**.",
                        course.sigla, course.name, target_date
                    ),
                    executed_tools,
                    navigate_to_date: Some(target_date),
                    updated_settings: None,
                    updated_snapshot: Some(updated_snapshot),
                    provider_used: "local-engine".to_string(),
                });
            }
        }
    }

    if lower.starts_with("todo ") || lower.starts_with("to-do ") || lower.starts_with("aggiungi task ") || lower.starts_with("ricordami di ") {
        let date = extract_iso_or_italian_date(&lower, &snapshot.telemetry.reference_today)
            .unwrap_or_else(|| snapshot.telemetry.reference_today.clone());
        let cleaned_text = prompt
            .trim_start_matches(|c: char| !c.is_whitespace())
            .trim()
            .trim_start_matches("di ")
            .trim();
        let final_text = if cleaned_text.is_empty() { prompt } else { cleaned_text };

        let _ = DatabaseManager::upsert_todo(
            conn,
            &UpsertTodoInput {
                id: None,
                session_id: session_id.to_string(),
                course_id: None,
                date: date.clone(),
                text: final_text.to_string(),
                priority: Some(1),
                plan_task_id: None,
                planned_hours: None,
                kind: None,
            },
        )
        .map_err(|e| e.to_string())?;

        executed_tools.push(AiToolCallRecord {
            tool_name: "upsert_todo".to_string(),
            summary: format!("Task aggiunto per il {}", date),
        });

        let updated_snapshot =
            DatabaseManager::build_session_snapshot(conn, session_id, &settings).map_err(|e| e.to_string())?;

        return Ok(AiAssistantResponse {
            reply_markdown: format!("Ho aggiunto il task **\"{}\"** alla giornata **{}**.", final_text, date),
            executed_tools,
            navigate_to_date: Some(date),
            updated_settings: None,
            updated_snapshot: Some(updated_snapshot),
            provider_used: "local-engine".to_string(),
        });
    }

    if let Some(target_date) = extract_iso_or_italian_date(&lower, &snapshot.telemetry.reference_today) {
        if lower.contains("vai a") || lower.contains("salta a") || lower.contains("mostra") {
            navigate_to_date = Some(target_date.clone());
            executed_tools.push(AiToolCallRecord {
                tool_name: "navigate_calendar".to_string(),
                summary: format!("Salto temporale a {}", target_date),
            });
            return Ok(AiAssistantResponse {
                reply_markdown: format!("Ho spostato il calendario alla data **{}**.", target_date),
                executed_tools,
                navigate_to_date,
                updated_settings: None,
                updated_snapshot: None,
                provider_used: "local-engine".to_string(),
            });
        }
    }

    // Risposta di riepilogo intelligente di default
    let next_info = if let (Some(ex), Some(c), Some(days)) = (
        &snapshot.telemetry.next_target_exam,
        &snapshot.telemetry.next_target_course,
        snapshot.telemetry.days_to_next_target,
    ) {
        format!(
            "Prossimo appello: **{} ({})** il **{}** (tra **{} giorni**).",
            c.sigla, c.name, ex.date, days
        )
    } else {
        "Nessun appello imminente pianificato.".to_string()
    };

    Ok(AiAssistantResponse {
        reply_markdown: format!(
            "Stato **{}**: **{} materie** (**{} CFU**), **{} appelli**, **{} conflitti**.\n{}\n\n*Comandi rapidi supportati:* `Quanti CFU a gennaio?`, `Qual è il corso più critico?`, `Crea corso MAM Meccanica 10 CFU`, `Aggiungi appello di MAM il 2027-01-22`, `Todo Ripassare teorema energia`, `Tema scuro`, `Zoom 115%`.",
            snapshot.session.name,
            snapshot.telemetry.courses_count,
            snapshot.telemetry.total_cfu,
            snapshot.telemetry.exams_count,
            snapshot.conflicts.len(),
            next_info
        ),
        executed_tools,
        navigate_to_date,
        updated_settings: None,
        updated_snapshot: None,
        provider_used: "local-engine".to_string(),
    })
}

fn detect_italian_month(text_lower: &str) -> Option<(u32, &'static str)> {
    let months = [
        ("gennaio", 1, "Gennaio"),
        ("febbraio", 2, "Febbraio"),
        ("marzo", 3, "Marzo"),
        ("aprile", 4, "Aprile"),
        ("maggio", 5, "Maggio"),
        ("giugno", 6, "Giugno"),
        ("luglio", 7, "Luglio"),
        ("agosto", 8, "Agosto"),
        ("settembre", 9, "Settembre"),
        ("ottobre", 10, "Ottobre"),
        ("novembre", 11, "Novembre"),
        ("dicembre", 12, "Dicembre"),
    ];
    for (needle, num, label) in months {
        if text_lower.contains(needle) {
            return Some((num, label));
        }
    }
    None
}

fn extract_number_in_range(text: &str, min_val: i32, max_val: i32) -> Option<i32> {
    for token in text.split(|c: char| !c.is_ascii_digit()) {
        if let Ok(v) = token.parse::<i32>() {
            if v >= min_val && v <= max_val {
                return Some(v);
            }
        }
    }
    None
}

fn extract_iso_or_italian_date(text_lower: &str, reference_today: &str) -> Option<String> {
    if text_lower.contains("oggi") {
        return Some(reference_today.to_string());
    }
    if text_lower.contains("domani") {
        if let Ok(dt) = NaiveDate::parse_from_str(reference_today, "%Y-%m-%d") {
            if let Some(next) = dt.succ_opt() {
                return Some(next.format("%Y-%m-%d").to_string());
            }
        }
    }

    // Cerca formato YYYY-MM-DD
    for word in text_lower.split_whitespace() {
        let clean = word.trim_matches(|c: char| !c.is_ascii_digit() && c != '-');
        if clean.len() == 10 && NaiveDate::parse_from_str(clean, "%Y-%m-%d").is_ok() {
            return Some(clean.to_string());
        }
    }

    // Cerca formato "15 gennaio 2027" o "15 gennaio"
    if let Some((month_num, _)) = detect_italian_month(text_lower) {
        let day = extract_number_in_range(text_lower, 1, 31).unwrap_or(1);
        let year = extract_number_in_range(text_lower, 2024, 2035).unwrap_or_else(|| {
            NaiveDate::parse_from_str(reference_today, "%Y-%m-%d")
                .map(|d| d.year())
                .unwrap_or(2027)
        });
        if let Some(d) = NaiveDate::from_ymd_opt(year, month_num, day as u32) {
            return Some(d.format("%Y-%m-%d").to_string());
        }
    }

    None
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ExtractedLessonItem {
    #[serde(alias = "day_of_week", alias = "day", alias = "giorno", alias = "dayOfWeek", alias = "weekday")]
    pub day_of_week: i32, // 1..7 (1=Lun, 7=Dom)

    #[serde(alias = "start_time", alias = "start", alias = "startTime", alias = "ora_inizio", alias = "inizio", alias = "oraInizio")]
    pub start_time: String, // HH:MM

    #[serde(alias = "end_time", alias = "end", alias = "endTime", alias = "ora_fine", alias = "fine", alias = "oraFine")]
    pub end_time: String,   // HH:MM

    #[serde(alias = "course_name", alias = "courseName", alias = "course", alias = "materia", alias = "subject", alias = "title", alias = "nome_corso", alias = "disciplina")]
    pub course_name: String,

    #[serde(default, alias = "aula", alias = "room", alias = "location", alias = "luogo", alias = "sede")]
    pub classroom: String,

    #[serde(default, alias = "prof", alias = "teacher", alias = "docente", alias = "insegnante", alias = "professore")]
    pub professor: String,
}

/// Normalizza una stringa oraria nel formato rigoroso a 24 ore "HH:MM" (es. "8:30" -> "08:30", "9.00" -> "09:00", "14" -> "14:00")
pub fn normalize_hour_string(s: &str) -> Option<String> {
    let clean = s.trim().replace('.', ":").replace('h', ":").replace('H', ":");
    let parts: Vec<&str> = clean.split(':').collect();
    if parts.is_empty() {
        return None;
    }
    let h_digits: String = parts[0].chars().filter(|c| c.is_ascii_digit()).collect();
    if h_digits.is_empty() {
        return None;
    }
    let hours: u32 = h_digits.parse().ok()?;
    if hours > 23 {
        return None;
    }
    let minutes: u32 = if parts.len() > 1 {
        let m_digits: String = parts[1].chars().filter(|c| c.is_ascii_digit()).collect();
        m_digits.parse().unwrap_or(0)
    } else {
        0
    };
    if minutes > 59 {
        return None;
    }
    Some(format!("{:02}:{:02}", hours, minutes))
}

/// Converte un valore generico (numero 1..7 o stringa "Lunedì", "Mon", "1") nel numero del giorno 1..7
pub fn parse_day_of_week_value(val: &serde_json::Value) -> Option<i32> {
    if let Some(n) = val.as_i64() {
        if (1..=7).contains(&n) {
            return Some(n as i32);
        }
    }
    if let Some(s) = val.as_str() {
        let trimmed = s.trim().to_lowercase();
        if let Ok(n) = trimmed.parse::<i32>() {
            if (1..=7).contains(&n) {
                return Some(n);
            }
        }
        if trimmed.contains("lun") || trimmed.contains("mon") {
            return Some(1);
        }
        if trimmed.contains("mar") || trimmed.contains("tue") {
            return Some(2);
        }
        if trimmed.contains("mer") || trimmed.contains("wed") {
            return Some(3);
        }
        if trimmed.contains("gio") || trimmed.contains("thu") {
            return Some(4);
        }
        if trimmed.contains("ven") || trimmed.contains("fri") {
            return Some(5);
        }
        if trimmed.contains("sab") || trimmed.contains("sat") {
            return Some(6);
        }
        if trimmed.contains("dom") || trimmed.contains("sun") {
            return Some(7);
        }
    }
    None
}

/// Rende tollerante un testo JSON prodotto da modelli AI:
/// 1) Converte apici singoli `'...'` in doppi apici `"..."`
/// 2) Aggiunge doppi apici a chiavi di oggetti non virgolettate (es. `{ lessons :` -> `{ "lessons":`)
/// 3) Chiude correttamente stringhe lasciate aperte da troncamenti
pub fn sanitize_lenient_json(input: &str) -> String {
    let mut out = String::with_capacity(input.len() + 128);
    let chars: Vec<char> = input.chars().collect();
    let n = chars.len();
    let mut i = 0;
    let mut in_double_quote = false;
    let mut in_single_quote = false;
    let mut escape = false;

    while i < n {
        let c = chars[i];

        if in_double_quote {
            out.push(c);
            if escape {
                escape = false;
            } else if c == '\\' {
                escape = true;
            } else if c == '"' {
                in_double_quote = false;
            }
            i += 1;
            continue;
        }

        if in_single_quote {
            if escape {
                out.push(c);
                escape = false;
            } else if c == '\\' {
                out.push(c);
                escape = true;
            } else if c == '\'' {
                out.push('"');
                in_single_quote = false;
            } else if c == '"' {
                out.push_str("\\\"");
            } else {
                out.push(c);
            }
            i += 1;
            continue;
        }

        if c == '"' {
            in_double_quote = true;
            out.push(c);
            i += 1;
            continue;
        }

        if c == '\'' {
            in_single_quote = true;
            out.push('"');
            i += 1;
            continue;
        }

        // Se troviamo una parola/identificatore non virgolettato
        if c.is_ascii_alphabetic() || c == '_' {
            let start = i;
            while i < n && (chars[i].is_ascii_alphanumeric() || chars[i] == '_') {
                i += 1;
            }
            let word: String = chars[start..i].iter().collect();

            // Salta spazi bianchi per verificare se è una chiave seguita da ':'
            let mut j = i;
            while j < n && (chars[j] == ' ' || chars[j] == '\t' || chars[j] == '\r' || chars[j] == '\n') {
                j += 1;
            }

            if j < n && chars[j] == ':' && word != "true" && word != "false" && word != "null" {
                out.push('"');
                out.push_str(&word);
                out.push('"');
            } else {
                out.push_str(&word);
            }
            continue;
        }

        out.push(c);
        i += 1;
    }

    if in_double_quote || in_single_quote {
        out.push('"');
    }

    out
}

/// Ripara un JSON troncato bruscamente a metà generazione (es. per raggiungimento limite token in un loop)
pub fn repair_truncated_json(input: &str) -> String {
    let trimmed = input.trim();
    if trimmed.ends_with('}') || trimmed.ends_with(']') {
        return trimmed.to_string();
    }

    if let Some(last_obj_end) = trimmed.rfind('}') {
        let mut sliced = trimmed[..=last_obj_end].trim().to_string();
        let open_brackets = sliced.chars().filter(|&c| c == '[').count();
        let close_brackets = sliced.chars().filter(|&c| c == ']').count();
        let open_braces = sliced.chars().filter(|&c| c == '{').count();
        let close_braces = sliced.chars().filter(|&c| c == '}').count();

        if open_brackets > close_brackets {
            for _ in 0..(open_brackets - close_brackets) {
                sliced.push(']');
            }
        }
        if open_braces > close_braces {
            for _ in 0..(open_braces - close_braces) {
                sliced.push('}');
            }
        }
        return sliced;
    }

    trimmed.to_string()
}

/// Estrae un valore tollerante da un singolo blocco di lezione senza richiedere JSON rigoroso
fn extract_lenient_val(block: &str, keys: &[&str]) -> Option<String> {
    for key in keys {
        if let Some(pos) = block.find(key) {
            let after_key = &block[pos + key.len()..];
            if let Some(colon_pos) = after_key.find(|c| c == ':' || c == '=') {
                let rest = after_key[colon_pos + 1..].trim_start();
                if rest.starts_with('"') {
                    if let Some(end_quote) = rest[1..].find('"') {
                        let val = &rest[1..=end_quote];
                        return Some(val.trim().to_string());
                    }
                } else if rest.starts_with('\'') {
                    if let Some(end_quote) = rest[1..].find('\'') {
                        let val = &rest[1..=end_quote];
                        return Some(val.trim().to_string());
                    }
                } else {
                    let end_pos = rest.find(|c| c == ',' || c == '}' || c == '\n' || c == '\r').unwrap_or(rest.len());
                    let val = rest[..end_pos].trim().trim_matches(|c| c == '"' || c == '\'');
                    if !val.is_empty() {
                        return Some(val.to_string());
                    }
                }
            }
        }
    }
    None
}

/// Parser di fallback permissivo che recupera le lezioni analizzando i singoli blocchi `{ ... }`
pub fn fallback_extract_lessons_lenient(text: &str) -> Vec<ExtractedLessonItem> {
    let mut items = Vec::new();
    let mut depth: i32 = 0;
    let mut start_idx = None;

    for (idx, ch) in text.char_indices() {
        if ch == '{' {
            if depth >= 1 && start_idx.is_none() {
                start_idx = Some(idx);
            }
            depth += 1;
        } else if ch == '}' {
            depth = depth.saturating_sub(1);
            if depth >= 1 {
                if let Some(start) = start_idx.take() {
                    let block = &text[start..=idx];
                    let day_raw = extract_lenient_val(block, &["dayOfWeek", "day_of_week", "day", "giorno", "weekday"]);
                    let day = day_raw.and_then(|d| parse_day_of_week_value(&serde_json::Value::String(d))).unwrap_or(1);

                    let start_raw = extract_lenient_val(block, &["startTime", "start_time", "start", "ora_inizio", "inizio"]);
                    let end_raw = extract_lenient_val(block, &["endTime", "end_time", "end", "ora_fine", "fine"]);
                    let mut norm_start = start_raw.as_deref().and_then(normalize_hour_string);
                    let mut norm_end = end_raw.as_deref().and_then(normalize_hour_string);

                    if norm_start.is_none() || norm_end.is_none() {
                        if let Some(combined) = extract_lenient_val(block, &["time", "orario", "hours", "fascia_oraria", "slot"]) {
                            let parts: Vec<&str> = combined.split(|c| c == '-' || c == '—' || c == '/' || c == '–').collect();
                            if parts.len() >= 2 {
                                if norm_start.is_none() {
                                    norm_start = normalize_hour_string(parts[0]);
                                }
                                if norm_end.is_none() {
                                    norm_end = normalize_hour_string(parts[1]);
                                }
                            }
                        }
                    }

                    let course_name = extract_lenient_val(block, &["courseName", "course_name", "course", "materia", "subject", "title", "nome_corso"])
                        .unwrap_or_default();
                    let classroom = extract_lenient_val(block, &["classroom", "aula", "room", "location"]).unwrap_or_default();
                    let professor = extract_lenient_val(block, &["professor", "docente", "teacher", "prof"]).unwrap_or_default();

                    if !course_name.is_empty() {
                        items.push(ExtractedLessonItem {
                            day_of_week: day,
                            start_time: norm_start.unwrap_or_else(|| "09:00".to_string()),
                            end_time: norm_end.unwrap_or_else(|| "11:00".to_string()),
                            course_name,
                            classroom,
                            professor,
                        });
                    }
                }
            }
        }
    }

    items
}

/// Pulisce e deserializza in modo super-resiliente il JSON restituito da qualsiasi modello Vision
pub fn clean_and_parse_lessons_json(text: &str) -> Result<Vec<ExtractedLessonItem>, String> {
    let clean = text.trim();
    if clean.is_empty() {
        return Err("Risposta AI vuota.".to_string());
    }

    // 1. Rimuovi blocchi markdown ```json ... ``` se presenti
    let json_candidate = if let Some(start) = clean.find("```") {
        let after_fence = &clean[start + 3..];
        let content_start = if let Some(newline) = after_fence.find('\n') {
            &after_fence[newline + 1..]
        } else {
            after_fence
        };
        if let Some(end) = content_start.rfind("```") {
            &content_start[..end]
        } else {
            content_start
        }
    } else {
        clean
    };

    // 2. Normalizza e correggi eventuali apici singoli o chiavi non virgolettate
    let sanitized = sanitize_lenient_json(json_candidate.trim());
    let repaired = repair_truncated_json(&sanitized);

    // 3. Isola la sottostringa JSON più esterna valida
    let trimmed = repaired.trim();
    let json_slice = match (trimmed.find('{'), trimmed.rfind('}'), trimmed.find('['), trimmed.rfind(']')) {
        (Some(first_obj), Some(last_obj), Some(first_arr), Some(last_arr)) => {
            if first_arr < first_obj && last_arr > last_obj {
                &trimmed[first_arr..=last_arr]
            } else {
                &trimmed[first_obj..=last_obj]
            }
        }
        (Some(first_obj), Some(last_obj), _, _) => &trimmed[first_obj..=last_obj],
        (_, _, Some(first_arr), Some(last_arr)) => &trimmed[first_arr..=last_arr],
        _ => trimmed,
    };

    let parsed_res: Result<serde_json::Value, _> = serde_json::from_str(json_slice);
    let mut raw_lessons = Vec::new();

    match parsed_res {
        Ok(parsed) => {
            let items_array_opt = if let Some(arr) = parsed.as_array() {
                Some(arr.clone())
            } else if let Some(obj) = parsed.as_object() {
                let candidate_keys = ["lessons", "timetable", "schedule", "orario", "classes", "items", "lezioni", "data"];
                let mut found = None;
                for key in &candidate_keys {
                    if let Some(val) = obj.get(*key) {
                        if let Some(arr) = val.as_array() {
                            found = Some(arr.clone());
                            break;
                        }
                    }
                }
                if found.is_none() {
                    for val in obj.values() {
                        if let Some(arr) = val.as_array() {
                            found = Some(arr.clone());
                            break;
                        }
                    }
                }
                found
            } else {
                None
            };

            if let Some(items_array) = items_array_opt {
                let course_aliases = ["courseName", "course_name", "course", "materia", "subject", "title", "nome_corso", "lezione", "name", "disciplina"];
                let start_aliases = ["startTime", "start_time", "start", "ora_inizio", "inizio", "from", "time_start", "oraInizio"];
                let end_aliases = ["endTime", "end_time", "end", "ora_fine", "fine", "to", "time_end", "oraFine"];
                let combined_time_aliases = ["time", "orario", "hours", "fascia_oraria", "fasciaOraria", "slot", "interval"];
                let day_aliases = ["dayOfWeek", "day_of_week", "day", "giorno", "weekday", "day_number"];
                let classroom_aliases = ["classroom", "aula", "room", "location", "luogo", "sede"];
                let professor_aliases = ["professor", "docente", "teacher", "insegnante", "prof", "professore"];

                for item in items_array {
                    if let Some(obj) = item.as_object() {
                        let mut day = 1;
                        for alias in &day_aliases {
                            if let Some(val) = obj.get(*alias) {
                                if let Some(d) = parse_day_of_week_value(val) {
                                    day = d;
                                    break;
                                }
                            }
                        }

                        let mut start_raw = String::new();
                        for alias in &start_aliases {
                            if let Some(val) = obj.get(*alias) {
                                if let Some(s) = val.as_str() {
                                    start_raw = s.trim().to_string();
                                    break;
                                } else if let Some(n) = val.as_i64() {
                                    start_raw = n.to_string();
                                    break;
                                }
                            }
                        }

                        let mut end_raw = String::new();
                        for alias in &end_aliases {
                            if let Some(val) = obj.get(*alias) {
                                if let Some(s) = val.as_str() {
                                    end_raw = s.trim().to_string();
                                    break;
                                } else if let Some(n) = val.as_i64() {
                                    end_raw = n.to_string();
                                    break;
                                }
                            }
                        }

                        let mut norm_start = normalize_hour_string(&start_raw);
                        let mut norm_end = normalize_hour_string(&end_raw);

                        if norm_start.is_none() || norm_end.is_none() {
                            for alias in &combined_time_aliases {
                                if let Some(val) = obj.get(*alias) {
                                    if let Some(combined) = val.as_str() {
                                        let parts: Vec<&str> = combined.split(|c| c == '-' || c == '—' || c == '/' || c == '–').collect();
                                        if parts.len() >= 2 {
                                            if norm_start.is_none() {
                                                norm_start = normalize_hour_string(parts[0]);
                                            }
                                            if norm_end.is_none() {
                                                norm_end = normalize_hour_string(parts[1]);
                                            }
                                        }
                                    }
                                }
                            }
                        }

                        let mut course_name = String::new();
                        for alias in &course_aliases {
                            if let Some(val) = obj.get(*alias) {
                                if let Some(s) = val.as_str() {
                                    if !s.trim().is_empty() {
                                        course_name = s.trim().to_string();
                                        break;
                                    }
                                }
                            }
                        }

                        let mut classroom = String::new();
                        for alias in &classroom_aliases {
                            if let Some(val) = obj.get(*alias) {
                                if let Some(s) = val.as_str() {
                                    if !s.trim().is_empty() {
                                        classroom = s.trim().to_string();
                                        break;
                                    }
                                }
                            }
                        }

                        let mut professor = String::new();
                        for alias in &professor_aliases {
                            if let Some(val) = obj.get(*alias) {
                                if let Some(s) = val.as_str() {
                                    if !s.trim().is_empty() {
                                        professor = s.trim().to_string();
                                        break;
                                    }
                                }
                            }
                        }

                        if !course_name.is_empty() {
                            let final_start = norm_start.unwrap_or_else(|| "09:00".to_string());
                            let final_end = norm_end.unwrap_or_else(|| "11:00".to_string());
                            raw_lessons.push(ExtractedLessonItem {
                                day_of_week: day,
                                start_time: final_start,
                                end_time: final_end,
                                course_name,
                                classroom,
                                professor,
                            });
                        }
                    }
                }
            }
        }
        Err(_) => {
            // Se serde_json fallisce nonostante le correzioni, usiamo il parser lenient sui blocchi
            let fallback_items = fallback_extract_lessons_lenient(text);
            raw_lessons.extend(fallback_items);
        }
    }

    if raw_lessons.is_empty() {
        let fallback_items = fallback_extract_lessons_lenient(text);
        raw_lessons.extend(fallback_items);
    }

    // 4. Deduplicazione lezioni (elimina ripetizioni identiche generate da loop AI)
    let mut unique_lessons = Vec::new();
    for item in raw_lessons {
        let is_duplicate = unique_lessons.iter().any(|existing: &ExtractedLessonItem| {
            existing.day_of_week == item.day_of_week
                && existing.start_time == item.start_time
                && existing.end_time == item.end_time
                && existing.course_name.eq_ignore_ascii_case(&item.course_name)
        });
        if !is_duplicate {
            unique_lessons.push(item);
        }
    }

    if unique_lessons.is_empty() {
        return Err(format!("Nessuna lezione identificata nell'orario. Risposta grezza: {}", text));
    }

    Ok(unique_lessons)
}

/// Limite massimo del payload base64 inviato al provider (≈ 15 MB).
const MAX_IMAGE_BASE64_BYTES: usize = 20 * 1024 * 1024;

fn vision_agent() -> ureq::Agent {
    crate::ai::downloader::get_http_agent()
}

fn sanitize_http_error(prefix: &str, err: ureq::Error) -> String {
    match err {
        ureq::Error::Status(code, resp) => {
            let detail = resp.into_string().unwrap_or_default();
            let detail: String = detail.chars().take(300).collect();
            format!("{}: status code {} {}", prefix, code, detail)
        }
        ureq::Error::Transport(t) => format!("{}: errore di rete ({})", prefix, t.kind()),
    }
}

/// Non tiene alcun lock sul database: le impostazioni vanno lette dal chiamante.
pub fn parse_timetable_image_with_ai(
    settings: &crate::models::AppSettings,
    _session_id: &str,
    image_base64: &str,
    mime_type: &str,
) -> Result<Vec<ExtractedLessonItem>, String> {
    let provider = settings.ai_provider.trim().to_lowercase();
    if image_base64.len() > MAX_IMAGE_BASE64_BYTES {
        return Err("Immagine troppo grande: usa una foto sotto i 15 MB.".to_string());
    }

    let clean_mime = if mime_type.is_empty() { "image/png" } else { mime_type };
    let clean_base64 = if let Some(idx) = image_base64.find(";base64,") {
        &image_base64[idx + 8..]
    } else {
        image_base64
    };

    let prompt_text = "Sei un sistema esperto di OCR e comprensione visiva strutturata di orari universitari e scolastici.\n\
        Analizza con la massima attenzione l'immagine dell'orario ed estrai TUTTE le lezioni settimanali presenti nella tabella o griglia.\n\n\
        REGOLE FONDAMENTALI DI ESTRAZIONE:\n\
        1. ASSE DEI GIORNI: Identifica i giorni della settimana (colonne o righe): Lunedì=1, Martedì=2, Mercoledì=3, Giovedì=4, Venerdì=5, Sabato=6, Domenica=7.\n\
        2. ASSE DEGLI ORARI: Leggi con assoluta precisione l'ora esatta di inizio (startTime) e fine (endTime). Se la fascia oraria è scritta dentro la casella della lezione (es. '08:30-10:30', '14.00 - 16.00'), usa SEMPRE prioritariamente quella. Se è indicata sulla griglia, segui con precisione la riga corrispondente.\n\
        3. FORMATO ORARIO RIGOROSO: Esprimi startTime ed endTime ESCLUSIVAMENTE nel formato a 24 ore 'HH:MM' con zero iniziale (es. '08:30', '09:00', '14:30', '16:00'). Non troncare i minuti e non invertire mai inizio e fine.\n\
        4. NOMI E AULE: Trascrivi fedelmente il nome della materia (courseName), l'aula (classroom) e il docente (professor) se visibili (altrimenti lascia stringa vuota '').\n\
        5. FORMATO RISPOSTA: Restituisci ESCLUSIVAMENTE un oggetto JSON valido avente la chiave 'lessons':\n\
        {\n\
          \"lessons\": [\n\
            {\n\
              \"dayOfWeek\": 1,\n\
              \"startTime\": \"08:30\",\n\
              \"endTime\": \"10:30\",\n\
              \"courseName\": \"Nome Corso\",\n\
              \"classroom\": \"Aula 3\",\n\
              \"professor\": \"Prof. Rossi\"\n\
            }\n\
          ]\n\
        }";

    match provider.as_str() {
        "local" => {
            let app_data_dir = DatabaseManager::resolve_app_data_dir();
            let runtime = get_runtime_manager();
            let port = runtime.ensure_started(
                &app_data_dir,
                &settings.ai_local_model_id,
                &settings.ai_hardware_preference,
                settings.ai_idle_timeout_minutes,
            )?;
            runtime.touch_activity();

            let data_url = format!("data:{};base64,{}", clean_mime, clean_base64);
            let body = serde_json::json!({
                "model": settings.ai_local_model_id,
                "temperature": 0.0,
                "top_p": 0.1,
                "repeat_penalty": 1.25,
                "frequency_penalty": 0.5,
                "presence_penalty": 0.3,
                "max_tokens": 2048,
                "response_format": { "type": "json_object" },
                "messages": [
                    {
                        "role": "system",
                        "content": "Sei un analizzatore OCR per orari universitari. Genera ESCLUSIVAMENTE JSON conforme a RFC 8259 (tutte le chiavi e valori testuali tra doppi apici \"...\"). Non usare mai apici singoli né chiavi non virgolettate. Elenca ciascuna lezione visibile UNA SOLA VOLTA e chiudi immediatamente il JSON con ']}' senza mai ripetere."
                    },
                    {
                        "role": "user",
                        "content": [
                            { "type": "text", "text": prompt_text },
                            {
                                "type": "image_url",
                                "image_url": { "url": data_url }
                            }
                        ]
                    }
                ]
            });

            let resp: serde_json::Value = vision_agent()
                .post(&format!("http://127.0.0.1:{}/v1/chat/completions", port))
                .send_json(body)
                .map_err(|e| sanitize_http_error("Errore chiamata AI Locale (llama-server)", e))?
                .into_json()
                .map_err(|e| format!("Errore parsing risposta AI Locale: {}", e))?;

            let text = resp["choices"][0]["message"]["content"]
                .as_str()
                .ok_or_else(|| "Risposta AI Locale vuota o non valida".to_string())?;

            clean_and_parse_lessons_json(text)
        }
        "gemini" => {
            if settings.ai_api_key.trim().is_empty() {
                return Err("Per analizzare con Gemini, configura la tua API Key nelle Impostazioni.".to_string());
            }
            let model = if settings.ai_model.trim().is_empty() {
                "gemini-2.5-flash"
            } else {
                settings.ai_model.trim()
            };
            let url = format!(
                "https://generativelanguage.googleapis.com/v1beta/models/{}:generateContent",
                model
            );

            let body = serde_json::json!({
                "contents": [{
                    "parts": [
                        { "text": prompt_text },
                        {
                            "inlineData": {
                                "mimeType": clean_mime,
                                "data": clean_base64
                            }
                        }
                    ]
                }],
                "generationConfig": {
                    "responseMimeType": "application/json",
                    "temperature": 0.0
                }
            });

            let resp: serde_json::Value = vision_agent()
                .post(&url)
                .set("x-goog-api-key", settings.ai_api_key.trim())
                .send_json(body)
                .map_err(|e| sanitize_http_error("Errore chiamata Gemini Vision", e))?
                .into_json()
                .map_err(|e| format!("Errore parsing risposta Gemini: {}", e))?;

            let text = resp["candidates"][0]["content"]["parts"][0]["text"]
                .as_str()
                .ok_or_else(|| "Risposta Gemini vuota o non valida".to_string())?;

            clean_and_parse_lessons_json(text)
        }
        "openai" => {
            if settings.ai_api_key.trim().is_empty() {
                return Err("Per analizzare con OpenAI, configura la tua API Key nelle Impostazioni.".to_string());
            }
            let model = if settings.ai_model.trim().is_empty() {
                "gpt-4o"
            } else {
                settings.ai_model.trim()
            };

            let data_url = format!("data:{};base64,{}", clean_mime, clean_base64);

            let body = serde_json::json!({
                "model": model,
                "temperature": 0.0,
                "response_format": { "type": "json_object" },
                "messages": [
                    {
                        "role": "user",
                        "content": [
                            { "type": "text", "text": prompt_text },
                            {
                                "type": "image_url",
                                "image_url": { "url": data_url }
                            }
                        ]
                    }
                ]
            });

            let resp: serde_json::Value = vision_agent()
                .post("https://api.openai.com/v1/chat/completions")
                .set("Authorization", &format!("Bearer {}", settings.ai_api_key.trim()))
                .send_json(body)
                .map_err(|e| sanitize_http_error("Errore chiamata OpenAI Vision", e))?
                .into_json()
                .map_err(|e| format!("Errore parsing risposta OpenAI: {}", e))?;

            let text = resp["choices"][0]["message"]["content"]
                .as_str()
                .ok_or_else(|| "Risposta OpenAI vuota o non valida".to_string())?;

            clean_and_parse_lessons_json(text)
        }
        "ollama" => {
            let base = settings.ai_endpoint_url.trim().trim_end_matches('/');
            let url = format!("{}/api/chat", base);
            let body = serde_json::json!({
                "model": settings.ai_model.trim(),
                "messages": [{
                    "role": "user",
                    "content": prompt_text,
                    "images": [clean_base64]
                }],
                "options": {
                    "temperature": 0.0
                },
                "format": "json",
                "stream": false
            });

            let resp: serde_json::Value = vision_agent()
                .post(&url)
                .send_json(body)
                .map_err(|e| sanitize_http_error("Errore chiamata Ollama Vision", e))?
                .into_json()
                .map_err(|e| format!("Errore parsing risposta Ollama: {}", e))?;

            let text = resp["message"]["content"]
                .as_str()
                .ok_or_else(|| "Risposta Ollama vuota o non valida".to_string())?;

            clean_and_parse_lessons_json(text)
        }
        _ => Err("Seleziona 'AI Locale' o un provider supportato nelle Impostazioni per analizzare la foto dell'orario.".to_string()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_normalize_hour_string() {
        assert_eq!(normalize_hour_string("8:30"), Some("08:30".to_string()));
        assert_eq!(normalize_hour_string("08:30"), Some("08:30".to_string()));
        assert_eq!(normalize_hour_string("8.30"), Some("08:30".to_string()));
        assert_eq!(normalize_hour_string("8h30"), Some("08:30".to_string()));
        assert_eq!(normalize_hour_string("9"), Some("09:00".to_string()));
        assert_eq!(normalize_hour_string("14:00"), Some("14:00".to_string()));
        assert_eq!(normalize_hour_string("14.45"), Some("14:45".to_string()));
        assert_eq!(normalize_hour_string("25:00"), None);
        assert_eq!(normalize_hour_string("invalid"), None);
    }

    #[test]
    fn test_clean_and_parse_lessons_json_resilience() {
        // Formato con markdown codefence, chiavi snake_case, orari senza zero iniziale e giorno come stringa
        let ai_markdown_output = r#"
        Certamente! Ecco le lezioni identificate nella griglia:
        ```json
        {
          "lessons": [
            {
              "giorno": "Lunedì",
              "ora_inizio": "8:30",
              "ora_fine": "10.30",
              "materia": "Fisica Sperimentale 1",
              "aula": "Aula 4B",
              "docente": "Prof. Bianchi"
            },
            {
              "dayOfWeek": 3,
              "startTime": "14:00",
              "endTime": "16:00",
              "courseName": "Analisi Matematica I",
              "classroom": "Aula Magna"
            }
          ]
        }
        ```
        Buono studio!
        "#;

        let parsed = clean_and_parse_lessons_json(ai_markdown_output).expect("Dovrebbe parsare con successo");
        assert_eq!(parsed.len(), 2);

        assert_eq!(parsed[0].day_of_week, 1);
        assert_eq!(parsed[0].start_time, "08:30");
        assert_eq!(parsed[0].end_time, "10:30");
        assert_eq!(parsed[0].course_name, "Fisica Sperimentale 1");
        assert_eq!(parsed[0].classroom, "Aula 4B");
        assert_eq!(parsed[0].professor, "Prof. Bianchi");

        assert_eq!(parsed[1].day_of_week, 3);
        assert_eq!(parsed[1].start_time, "14:00");
        assert_eq!(parsed[1].end_time, "16:00");
        assert_eq!(parsed[1].course_name, "Analisi Matematica I");
        assert_eq!(parsed[1].classroom, "Aula Magna");
        assert_eq!(parsed[1].professor, "");
    }

    #[test]
    fn test_clean_and_parse_lessons_json_combined_slot() {
        let ai_output = r#"
        [
          {
            "day": "giovedì",
            "time": "09:00 - 11:00",
            "subject": "Chimica Generale",
            "room": "Lab 2"
          }
        ]
        "#;
        let parsed = clean_and_parse_lessons_json(ai_output).expect("Dovrebbe parsare array con fascia combinata");
        assert_eq!(parsed.len(), 1);
        assert_eq!(parsed[0].day_of_week, 4);
        assert_eq!(parsed[0].start_time, "09:00");
        assert_eq!(parsed[0].end_time, "11:00");
        assert_eq!(parsed[0].course_name, "Chimica Generale");
        assert_eq!(parsed[0].classroom, "Lab 2");
    }

    #[test]
    fn test_parse_user_unquoted_single_quoted_and_repeating_lessons() {
        // Stringa identica a quella riscontrata dall'utente nello screenshot:
        // chiavi senza virgolette ('lessons:', 'dayOfWeek:'), apici singoli ('08:15'),
        // lezioni duplicate a causa di loop e JSON troncato alla fine.
        let raw_user_error = r#"{ lessons : [ 
            { dayOfWeek : 3, startTime : '08:15', endTime : '10:15', "courseName": "Progettazione di Sistemi Meccanici", "classroom": "BL.28.2.1", "professor": "Filippini Mauro" },
            { 'dayOfWeek': 4, 'startTime': '10:15', 'endTime': '12:15', courseName: 'Progettazione di Sistemi Meccanici', classroom: 'BL.28.2.1', professor: 'Filippini Mauro' },
            { dayOfWeek: 3, startTime: "13:15", endTime: "15:15", courseName: "Progettazione di Sistemi Meccanici", classroom: "BL.28.4", professor: "Filippini Mauro" },
            { dayOfWeek: 4, startTime: "15:15", endTime: "17:15", courseName: "Progettazione di Sistemi Meccanici", classroom: "BL.28.4", professor: "Filippini Mauro" },
            { dayOfWeek: 3, startTime: "08:15", endTime: "10:15", courseName: "Progettazione di Sistemi Meccanici", classroom: "BL.28.2.1", professor: "Filippini Mauro" },
            { dayOfWeek: 4, startTime: "10:15", endTime: "12:15", courseName: "Progettazione di Sistemi Meccanici", classroom: "BL.28.2.1", professor: "Filippini Mauro" },
            { dayOfWeek: 3, startTime: "13:15", endTime: "15:15", courseName: "Progettazione di Sistemi Meccanici", classroom: "BL.28.4", professor: "Filippini Mauro" }
        "#;
        let parsed = clean_and_parse_lessons_json(raw_user_error).expect("Dovrebbe riparare, parsare e deduplicare con successo");
        assert_eq!(parsed.len(), 4);
        assert_eq!(parsed[0].day_of_week, 3);
        assert_eq!(parsed[0].start_time, "08:15");
        assert_eq!(parsed[0].end_time, "10:15");
        assert_eq!(parsed[0].classroom, "BL.28.2.1");

        assert_eq!(parsed[1].day_of_week, 4);
        assert_eq!(parsed[1].start_time, "10:15");
        assert_eq!(parsed[1].end_time, "12:15");
        assert_eq!(parsed[1].classroom, "BL.28.2.1");

        assert_eq!(parsed[2].day_of_week, 3);
        assert_eq!(parsed[2].start_time, "13:15");
        assert_eq!(parsed[2].end_time, "15:15");
        assert_eq!(parsed[2].classroom, "BL.28.4");

        assert_eq!(parsed[3].day_of_week, 4);
        assert_eq!(parsed[3].start_time, "15:15");
        assert_eq!(parsed[3].end_time, "17:15");
        assert_eq!(parsed[3].classroom, "BL.28.4");
    }

    #[test]
    fn test_ai_deterministic_tool_calling_and_queries() {
        let db = DatabaseManager::init_in_memory().unwrap();
        db.with_conn(|conn| {
            let mut settings = DatabaseManager::get_settings(conn).unwrap();
            settings.ai_provider = "none".to_string();
            DatabaseManager::save_settings(conn, &settings).unwrap();
            Ok(())
        })
        .unwrap();

        let session = db
            .with_conn(|conn| {
                DatabaseManager::upsert_session(
                    conn,
                    &crate::models::UpsertSessionInput {
                        id: None,
                        name: "Sessione Estiva".to_string(),
                        subtitle: "Test AI".to_string(),
                        session_type: "estiva".to_string(),
                        academic_year: "2026/2027".to_string(),
                        start_date: "2027-06-01".to_string(),
                        end_date: "2027-07-31".to_string(),
                        notes: None,
                        plan_id: None,
                    },
                )
            })
            .unwrap();

        // 1. Crea corso tramite comando naturale AI
        let res_course = db
            .with_conn(|conn| {
                Ok(process_natural_command(
                    conn,
                    &session.id,
                    "Crea corso ETD Elettrotecnica 12 CFU",
                )
                .unwrap())
            })
            .unwrap();
        assert_eq!(res_course.executed_tools.len(), 1);
        assert_eq!(res_course.executed_tools[0].tool_name, "upsert_course");

        // 2. Pianifica appello a luglio tramite comando naturale AI
        let res_exam = db
            .with_conn(|conn| {
                Ok(process_natural_command(
                    conn,
                    &session.id,
                    "Aggiungi appello di ETD il 2027-07-14",
                )
                .unwrap())
            })
            .unwrap();
        assert_eq!(res_exam.executed_tools[0].tool_name, "upsert_exam");
        assert_eq!(res_exam.navigate_to_date.as_deref(), Some("2027-07-14"));

        // 3. Interroga statistiche CFU a luglio
        let res_stats = db
            .with_conn(|conn| {
                Ok(process_natural_command(
                    conn,
                    &session.id,
                    "Quanti CFU ho pianificato a luglio?",
                )
                .unwrap())
            })
            .unwrap();
        assert!(res_stats.reply_markdown.contains("12 CFU"));
    }
}
