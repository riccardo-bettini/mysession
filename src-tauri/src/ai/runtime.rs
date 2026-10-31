use crate::ai::catalog::get_ai_model_catalog;
use serde::{Deserialize, Serialize};
use std::net::TcpListener;
use std::path::{Path, PathBuf};
use std::process::{Child, Command};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum AiRuntimeState {
    Stopped,
    Starting,
    Ready,
    Error,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AiRuntimeStatus {
    pub state: AiRuntimeState,
    pub active_model_id: Option<String>,
    pub port: Option<u16>,
    pub idle_timeout_minutes: i32,
    pub seconds_until_idle_shutdown: Option<u64>,
    pub error_message: Option<String>,
}

pub struct AiRuntimeManager {
    inner: Arc<Mutex<RuntimeInner>>,
}

struct RuntimeInner {
    state: AiRuntimeState,
    active_model_id: Option<String>,
    port: Option<u16>,
    child: Option<Child>,
    last_activity: Instant,
    idle_timeout_minutes: i32,
    error_message: Option<String>,
    shutdown_flag: Arc<std::sync::atomic::AtomicBool>,
}

impl Default for AiRuntimeManager {
    fn default() -> Self {
        Self::new()
    }
}

impl AiRuntimeManager {
    pub fn new() -> Self {
        Self {
            inner: Arc::new(Mutex::new(RuntimeInner {
                state: AiRuntimeState::Stopped,
                active_model_id: None,
                port: None,
                child: None,
                last_activity: Instant::now(),
                idle_timeout_minutes: 5,
                error_message: None,
                shutdown_flag: Arc::new(std::sync::atomic::AtomicBool::new(false)),
            })),
        }
    }

    /// Ritorna lo stato attuale del runtime per il frontend
    pub fn get_status(&self) -> AiRuntimeStatus {
        let guard = self.inner.lock().unwrap();
        let seconds_until_idle = if guard.state == AiRuntimeState::Ready && guard.idle_timeout_minutes > 0 {
            let timeout_secs = (guard.idle_timeout_minutes as u64) * 60;
            let elapsed = guard.last_activity.elapsed().as_secs();
            Some(timeout_secs.saturating_sub(elapsed))
        } else {
            None
        };

        AiRuntimeStatus {
            state: guard.state.clone(),
            active_model_id: guard.active_model_id.clone(),
            port: guard.port,
            idle_timeout_minutes: guard.idle_timeout_minutes,
            seconds_until_idle_shutdown: seconds_until_idle,
            error_message: guard.error_message.clone(),
        }
    }

    /// Segnala attività per resettare il timer di inattività
    pub fn touch_activity(&self) {
        if let Ok(mut guard) = self.inner.lock() {
            guard.last_activity = Instant::now();
        }
    }

    /// Assicura che il modello locale specificato sia pronto per rispondere
    pub fn ensure_started(
        &self,
        app_data_dir: &Path,
        model_id: &str,
        hardware_pref: &str,
        idle_timeout_minutes: i32,
    ) -> Result<u16, String> {
        let (already_ready, port) = {
            let mut guard = self.inner.lock().unwrap();
            guard.idle_timeout_minutes = idle_timeout_minutes;
            guard.last_activity = Instant::now();
            if guard.state == AiRuntimeState::Ready && guard.active_model_id.as_deref() == Some(model_id) {
                if let Some(p) = guard.port {
                    return Ok(p);
                }
            }
            (false, guard.port)
        };

        if already_ready {
            if let Some(p) = port {
                return Ok(p);
            }
        }

        // Se è in esecuzione con un altro modello, lo spegniamo prima
        self.stop();

        // 1. Risolvi il modello dal catalogo
        let catalog = get_ai_model_catalog();
        let item = catalog
            .into_iter()
            .find(|m| m.id == model_id)
            .ok_or_else(|| format!("Modello '{}' non trovato nel catalogo", model_id))?;

        let models_dir = app_data_dir.join("models");
        let model_path = models_dir.join(&item.file_name);
        let mmproj_path = models_dir.join(&item.mmproj_file_name);

        if !model_path.exists() {
            return Err(format!(
                "Il file del modello ({}) non è presente sul disco. Scaricalo nelle Impostazioni dell'app.",
                item.file_name
            ));
        }

        // 2. Risolvi o approvvigiona l'eseguibile llama-server
        let server_exe = ensure_llama_server_binary(app_data_dir)?;

        // 3. Riserva una porta TCP effimera libera
        let free_port = find_free_tcp_port()?;

        // 4. Configura argomenti di lancio
        let ngl = match hardware_pref.to_lowercase().as_str() {
            "cpu" => "0",
            _ => "99", // Offload completo GPU (Vulkan/DirectML)
        };

        let mut cmd = Command::new(&server_exe);
        cmd.arg("-m")
            .arg(&model_path)
            .arg("--port")
            .arg(free_port.to_string())
            .arg("--host")
            .arg("127.0.0.1")
            .arg("-ngl")
            .arg(ngl)
            .arg("-c")
            .arg(item.context_window.to_string())
            .arg("--log-disable");

        if mmproj_path.exists() {
            cmd.arg("--mmproj").arg(&mmproj_path);
        }

        // Su Windows impedisce l'apertura di finestre prompt nere indesiderate
        #[cfg(windows)]
        {
            use std::os::windows::process::CommandExt;
            const CREATE_NO_WINDOW: u32 = 0x08000000;
            cmd.creation_flags(CREATE_NO_WINDOW);
        }

        {
            let mut guard = self.inner.lock().unwrap();
            guard.state = AiRuntimeState::Starting;
            guard.error_message = None;
        }

        let child = cmd.spawn().map_err(|e| {
            let err = format!("Impossibile avviare llama-server ({:?}): {}", server_exe, e);
            let mut guard = self.inner.lock().unwrap();
            guard.state = AiRuntimeState::Error;
            guard.error_message = Some(err.clone());
            err
        })?;

        // Su Windows proteggiamo il processo associandolo a un JobObject con kill-on-close
        #[cfg(windows)]
        {
            attach_job_object_kill_on_close(&child);
        }

        let shutdown_flag = Arc::new(std::sync::atomic::AtomicBool::new(false));

        {
            let mut guard = self.inner.lock().unwrap();
            guard.child = Some(child);
            guard.port = Some(free_port);
            guard.active_model_id = Some(model_id.to_string());
            guard.last_activity = Instant::now();
            guard.shutdown_flag = shutdown_flag.clone();
        }

        // 5. Attendi che il server sia pronto (health poll fino a 25 secondi)
        let health_url = format!("http://127.0.0.1:{}/health", free_port);
        let start_time = Instant::now();
        let mut is_ready = false;

        while start_time.elapsed() < Duration::from_secs(25) {
            std::thread::sleep(Duration::from_millis(300));

            // Verifica se il processo è terminato prematuramente per errore
            {
                let mut guard = self.inner.lock().unwrap();
                if let Some(ref mut c) = guard.child {
                    if let Ok(Some(status)) = c.try_wait() {
                        guard.state = AiRuntimeState::Error;
                        let err = format!("Il server AI si è interrotto subito con codice: {}", status);
                        guard.error_message = Some(err.clone());
                        return Err(err);
                    }
                }
            }

            if let Ok(resp) = ureq::get(&health_url).timeout(Duration::from_millis(600)).call() {
                if resp.status() == 200 {
                    is_ready = true;
                    break;
                }
            }
        }

        if !is_ready {
            self.stop();
            return Err("Timeout durante l'avvio del modello AI in memoria.".to_string());
        }

        {
            let mut guard = self.inner.lock().unwrap();
            guard.state = AiRuntimeState::Ready;
            guard.last_activity = Instant::now();
        }

        // 6. Avvia il monitor per lo spegnimento su inattività in background
        let inner_clone = Arc::clone(&self.inner);
        std::thread::spawn(move || {
            loop {
                std::thread::sleep(Duration::from_secs(5));

                let should_stop = {
                    let guard = inner_clone.lock().unwrap();
                    if guard.shutdown_flag.load(std::sync::atomic::Ordering::Relaxed) {
                        break;
                    }
                    if guard.state != AiRuntimeState::Ready {
                        break;
                    }
                    if guard.idle_timeout_minutes <= 0 {
                        // 0 = Mai spegnere (sempre attivo)
                        false
                    } else {
                        let timeout = Duration::from_secs((guard.idle_timeout_minutes as u64) * 60);
                        guard.last_activity.elapsed() >= timeout
                    }
                };

                if should_stop {
                    let mut guard = inner_clone.lock().unwrap();
                    if let Some(ref mut child) = guard.child {
                        let _ = child.kill();
                    }
                    guard.child = None;
                    guard.port = None;
                    guard.state = AiRuntimeState::Stopped;
                    guard.shutdown_flag.store(true, std::sync::atomic::Ordering::Relaxed);
                    break;
                }
            }
        });

        Ok(free_port)
    }

    /// Spegne il server AI e libera istantaneamente la memoria RAM e VRAM
    pub fn stop(&self) {
        let mut guard = self.inner.lock().unwrap();
        guard.shutdown_flag.store(true, std::sync::atomic::Ordering::Relaxed);
        if let Some(ref mut child) = guard.child {
            let _ = child.kill();
        }
        guard.child = None;
        guard.port = None;
        guard.state = AiRuntimeState::Stopped;
    }
}

fn find_free_tcp_port() -> Result<u16, String> {
    let listener = TcpListener::bind("127.0.0.1:0").map_err(|e| e.to_string())?;
    let port = listener.local_addr().map_err(|e| e.to_string())?.port();
    drop(listener);
    Ok(port)
}

fn find_llama_server_binary(app_data_dir: &Path) -> Result<PathBuf, String> {
    // 1. In %APPDATA%\com.mysession.app\bin\llama-server.exe
    let in_app_data = app_data_dir.join("bin").join("llama-server.exe");
    if in_app_data.exists() {
        return Ok(in_app_data);
    }

    // 2. Accanto all'eseguibile principale mysession.exe
    if let Ok(current_exe) = std::env::current_exe() {
        if let Some(dir) = current_exe.parent() {
            let beside = dir.join("llama-server.exe");
            if beside.exists() {
                return Ok(beside);
            }
            let in_bin = dir.join("bin").join("llama-server.exe");
            if in_bin.exists() {
                return Ok(in_bin);
            }
        }
    }

    // 3. Nel PATH di sistema
    if let Ok(path_env) = std::env::var("PATH") {
        for p in std::env::split_paths(&path_env) {
            let candidate = p.join("llama-server.exe");
            if candidate.exists() {
                return Ok(candidate);
            }
        }
    }

    Err(
        "Binario 'llama-server.exe' non trovato. Assicurati che sia presente in %APPDATA%\\com.mysession.app\\bin o accanto all'app.".to_string(),
    )
}

/// Assicura che l'eseguibile del motore AI `llama-server.exe` sia disponibile sul sistema;
/// se assente, lo scarica automaticamente dai rilasci ufficiali GitHub (versione Vulkan/CPU)
pub fn ensure_llama_server_binary(app_data_dir: &Path) -> Result<PathBuf, String> {
    if let Ok(bin) = find_llama_server_binary(app_data_dir) {
        return Ok(bin);
    }

    let bin_dir = app_data_dir.join("bin");
    let _ = std::fs::create_dir_all(&bin_dir);

    let zip_url = "https://github.com/ggml-org/llama.cpp/releases/download/b12020/llama-b12020-bin-win-vulkan-x64.zip";
    let temp_zip = bin_dir.join("llama-engine.zip");

    let agent = crate::ai::downloader::get_http_agent();
    let resp = agent
        .get(zip_url)
        .set("User-Agent", "Mozilla/5.0 (Windows NT 10.0; Win64; x64) MySession/1.0")
        .call()
        .map_err(|e| format!("Impossibile scaricare il motore AI llama-server: {}", e))?;

    let mut reader = resp.into_reader();
    let mut file = std::fs::File::create(&temp_zip).map_err(|e| e.to_string())?;
    std::io::copy(&mut reader, &mut file).map_err(|e| e.to_string())?;
    drop(file);

    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        const CREATE_NO_WINDOW: u32 = 0x08000000;
        let mut cmd = Command::new("tar");
        cmd.arg("-xf").arg(&temp_zip).arg("-C").arg(&bin_dir).creation_flags(CREATE_NO_WINDOW);
        let status = cmd.status().map_err(|e| format!("Errore estrazione tar: {}", e))?;
        let _ = std::fs::remove_file(&temp_zip);
        if !status.success() {
            return Err("Estrazione del motore AI non riuscita.".to_string());
        }
    }

    find_llama_server_binary(app_data_dir)
}

#[cfg(windows)]
fn attach_job_object_kill_on_close(child: &Child) {
    use std::os::windows::io::AsRawHandle;
    use windows::Win32::Foundation::HANDLE;
    use windows::Win32::System::JobObjects::{
        AssignProcessToJobObject, CreateJobObjectW, SetInformationJobObject,
        JobObjectExtendedLimitInformation, JOBOBJECT_EXTENDED_LIMIT_INFORMATION,
        JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE,
    };

    unsafe {
        if let Ok(job) = CreateJobObjectW(None, None) {
            let mut info = JOBOBJECT_EXTENDED_LIMIT_INFORMATION::default();
            info.BasicLimitInformation.LimitFlags = JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE;

            let set_res = SetInformationJobObject(
                job,
                JobObjectExtendedLimitInformation,
                &info as *const _ as *const std::ffi::c_void,
                std::mem::size_of::<JOBOBJECT_EXTENDED_LIMIT_INFORMATION>() as u32,
            );

            if set_res.is_ok() {
                let process_handle = HANDLE(child.as_raw_handle());
                let _ = AssignProcessToJobObject(job, process_handle);
            }
        }
    }
}
