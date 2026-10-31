use crate::ai::catalog::get_ai_model_catalog;
use serde::Serialize;
use std::fs::{self, File};
use std::io::{Read, Write};
use std::path::Path;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Instant;
use tauri::{AppHandle, Emitter};

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AiModelDownloadProgress {
    pub model_id: String,
    pub file_name: String,
    pub downloaded_bytes: u64,
    pub total_bytes: u64,
    pub progress_pct: f64,
    pub speed_mb_per_sec: f64,
    pub is_completed: bool,
    pub error: Option<String>,
}

pub struct ModelDownloadManager {
    cancel_flags: Arc<Mutex<std::collections::HashMap<String, Arc<AtomicBool>>>>,
}

impl Default for ModelDownloadManager {
    fn default() -> Self {
        Self::new()
    }
}

impl ModelDownloadManager {
    pub fn new() -> Self {
        Self {
            cancel_flags: Arc::new(Mutex::new(std::collections::HashMap::new())),
        }
    }

    pub fn cancel(&self, model_id: &str) {
        let guard = self.cancel_flags.lock().unwrap();
        if let Some(flag) = guard.get(model_id) {
            flag.store(true, Ordering::Relaxed);
        }
    }

    pub fn start_download(
        &self,
        app_handle: AppHandle,
        app_data_dir: std::path::PathBuf,
        model_id: String,
    ) -> Result<(), String> {
        let catalog = get_ai_model_catalog();
        let item = catalog
            .into_iter()
            .find(|m| m.id == model_id)
            .ok_or_else(|| format!("Modello '{}' non trovato nel catalogo", model_id))?;

        let cancel_flag = Arc::new(AtomicBool::new(false));
        {
            let mut guard = self.cancel_flags.lock().unwrap();
            guard.insert(model_id.clone(), cancel_flag.clone());
        }

        std::thread::spawn(move || {
            let models_dir = app_data_dir.join("models");
            if let Err(e) = fs::create_dir_all(&models_dir) {
                emit_error(&app_handle, &model_id, &format!("Errore creazione cartella: {}", e));
                return;
            }

            // 1. Scarica il file principale del modello GGUF
            let model_dest = models_dir.join(&item.file_name);
            if !model_dest.exists() {
                if let Err(e) = download_single_file(
                    &app_handle,
                    &model_id,
                    &item.file_name,
                    &item.download_url,
                    &model_dest,
                    &cancel_flag,
                ) {
                    emit_error(&app_handle, &model_id, &e);
                    return;
                }
            }

            // 2. Scarica il proiettore visivo mmproj (necessario per VLM)
            if !item.mmproj_file_name.is_empty() && !item.mmproj_download_url.is_empty() {
                let mmproj_dest = models_dir.join(&item.mmproj_file_name);
                if !mmproj_dest.exists() {
                    if let Err(e) = download_single_file(
                        &app_handle,
                        &model_id,
                        &item.mmproj_file_name,
                        &item.mmproj_download_url,
                        &mmproj_dest,
                        &cancel_flag,
                    ) {
                        emit_error(&app_handle, &model_id, &e);
                        return;
                    }
                }
            }

            // Notifica completamento
            let _ = app_handle.emit(
                "ai-model-download-progress",
                AiModelDownloadProgress {
                    model_id: model_id.clone(),
                    file_name: item.file_name,
                    downloaded_bytes: 0,
                    total_bytes: 0,
                    progress_pct: 100.0,
                    speed_mb_per_sec: 0.0,
                    is_completed: true,
                    error: None,
                },
            );
        });

        Ok(())
    }
}

fn download_single_file(
    app: &AppHandle,
    model_id: &str,
    display_name: &str,
    url: &str,
    dest_path: &Path,
    cancel_flag: &Arc<AtomicBool>,
) -> Result<(), String> {
    let part_path = dest_path.with_extension("part");

    let resp = ureq::get(url)
        .call()
        .map_err(|e| format!("Errore di rete durante il download di {}: {}", display_name, e))?;

    let total_bytes: u64 = resp
        .header("Content-Length")
        .and_then(|s| s.parse().ok())
        .unwrap_or(0);

    let mut reader = resp.into_reader();
    let mut file = File::create(&part_path).map_err(|e| format!("Errore creazione file part: {}", e))?;

    let mut buffer = [0u8; 64 * 1024]; // 64 KB buffer
    let mut downloaded_bytes: u64 = 0;
    let start_time = Instant::now();
    let mut last_emit = Instant::now();

    loop {
        if cancel_flag.load(Ordering::Relaxed) {
            drop(file);
            let _ = fs::remove_file(&part_path);
            return Err("Download annullato dall'utente.".to_string());
        }

        let bytes_read = reader
            .read(&mut buffer)
            .map_err(|e| format!("Errore lettura stream download: {}", e))?;

        if bytes_read == 0 {
            break;
        }

        file.write_all(&buffer[..bytes_read])
            .map_err(|e| format!("Errore scrittura disco: {}", e))?;

        downloaded_bytes += bytes_read as u64;

        if last_emit.elapsed() >= std::time::Duration::from_millis(300) {
            let elapsed_secs = start_time.elapsed().as_secs_f64().max(0.1);
            let speed_mb = ((downloaded_bytes as f64) / (1024.0 * 1024.0)) / elapsed_secs;
            let pct = if total_bytes > 0 {
                ((downloaded_bytes as f64) / (total_bytes as f64)) * 100.0
            } else {
                0.0
            };

            let _ = app.emit(
                "ai-model-download-progress",
                AiModelDownloadProgress {
                    model_id: model_id.to_string(),
                    file_name: display_name.to_string(),
                    downloaded_bytes,
                    total_bytes,
                    progress_pct: (pct * 10.0).round() / 10.0,
                    speed_mb_per_sec: (speed_mb * 10.0).round() / 10.0,
                    is_completed: false,
                    error: None,
                },
            );

            last_emit = Instant::now();
        }
    }

    drop(file);
    fs::rename(&part_path, dest_path).map_err(|e| format!("Errore finalizzazione file: {}", e))?;
    Ok(())
}

fn emit_error(app: &AppHandle, model_id: &str, error: &str) {
    let _ = app.emit(
        "ai-model-download-progress",
        AiModelDownloadProgress {
            model_id: model_id.to_string(),
            file_name: String::new(),
            downloaded_bytes: 0,
            total_bytes: 0,
            progress_pct: 0.0,
            speed_mb_per_sec: 0.0,
            is_completed: false,
            error: Some(error.to_string()),
        },
    );
}
