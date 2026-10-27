use crate::db::DatabaseManager;
use crate::models::{
    AppSettings, SessionSnapshot, UpsertCourseInput, UpsertExamInput, UpsertTodoInput,
};
use chrono::{Datelike, Local, NaiveDate};
use rusqlite::Connection;
use serde::{Deserialize, Serialize};

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

    // Se l'utente ha configurato un provider LLM esterno (gemini / openai / ollama),
    // prova prima l'inferenza strutturata; in caso di provider "local" o errore di rete,
    // esegue il parser deterministico di Intent & Tool Calling.
    let provider = settings.ai_provider.trim().to_lowercase();
    if provider == "gemini" || provider == "openai" || provider == "ollama" {
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

    let raw_json = match provider.as_str() {
        "gemini" => {
            if settings.ai_api_key.trim().is_empty() {
                return Err("API Key Gemini non configurata".to_string());
            }
            let model = if settings.ai_model.trim().is_empty() {
                "gemini-2.5-flash"
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
            let resp: serde_json::Value = ureq::post(&url)
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
            let resp: serde_json::Value = ureq::post("https://api.openai.com/v1/chat/completions")
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
            let resp: serde_json::Value = ureq::post(&url)
                .send_json(body)
                .map_err(|e| e.to_string())?
                .into_json()
                .map_err(|e| e.to_string())?;
            resp["response"]
                .as_str()
                .ok_or_else(|| "Risposta Ollama vuota".to_string())?
                .to_string()
        }
        _ => return Err("Provider locale".to_string()),
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ai_deterministic_tool_calling_and_queries() {
        let db = DatabaseManager::init_in_memory().unwrap();
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
