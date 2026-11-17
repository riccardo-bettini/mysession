use crate::models::CliAgentInfo;
use std::path::{Path, PathBuf};
use std::process::Command;

#[cfg(windows)]
fn apply_no_window(cmd: &mut Command) {
    use std::os::windows::process::CommandExt;
    const CREATE_NO_WINDOW: u32 = 0x08000000;
    cmd.creation_flags(CREATE_NO_WINDOW);
}

#[cfg(not(windows))]
fn apply_no_window(_cmd: &mut Command) {}

fn resolve_executable(bin_name: &str) -> Option<PathBuf> {
    let candidates: Vec<String> = if cfg!(windows) {
        if bin_name.ends_with(".exe") || bin_name.ends_with(".cmd") || bin_name.ends_with(".bat") {
            vec![bin_name.to_string()]
        } else {
            vec![
                format!("{bin_name}.exe"),
                format!("{bin_name}.cmd"),
                format!("{bin_name}.bat"),
                bin_name.to_string(),
            ]
        }
    } else {
        vec![bin_name.to_string()]
    };

    if let Ok(path_env) = std::env::var("PATH") {
        for dir in std::env::split_paths(&path_env) {
            for cand in &candidates {
                let p = dir.join(cand);
                if p.is_file() {
                    return Some(p);
                }
            }
        }
    }

    // Extra standard user binary paths on Windows
    #[cfg(windows)]
    {
        let mut extra_dirs: Vec<PathBuf> = Vec::new();
        if let Ok(appdata) = std::env::var("APPDATA") {
            extra_dirs.push(PathBuf::from(&appdata).join("npm"));
        }
        if let Ok(local) = std::env::var("LOCALAPPDATA") {
            extra_dirs.push(PathBuf::from(&local).join("Programs").join("Antigravity").join("bin"));
            extra_dirs.push(PathBuf::from(&local).join("Microsoft").join("WinGet").join("Links"));
        }
        if let Ok(home) = std::env::var("USERPROFILE") {
            extra_dirs.push(PathBuf::from(&home).join(".cargo").join("bin"));
            extra_dirs.push(PathBuf::from(&home).join(".local").join("bin"));
        }
        for dir in extra_dirs {
            for cand in &candidates {
                let p = dir.join(cand);
                if p.is_file() {
                    return Some(p);
                }
            }
        }
    }

    None
}

pub fn detect_cli_agents(custom_command: &str) -> Vec<CliAgentInfo> {
    let agy_path = resolve_executable("agy");
    let claude_path = resolve_executable("claude");
    let codex_path = resolve_executable("codex");

    let custom_trim = custom_command.trim();
    let custom_bin = custom_trim.split_whitespace().next().unwrap_or("");
    let custom_path = if !custom_bin.is_empty() {
        resolve_executable(custom_bin).or_else(|| {
            let p = Path::new(custom_bin);
            if p.is_file() {
                Some(p.to_path_buf())
            } else {
                None
            }
        })
    } else {
        None
    };

    vec![
        CliAgentInfo {
            id: "cli_agy".to_string(),
            name: "Antigravity CLI".to_string(),
            command: "agy".to_string(),
            installed: agy_path.is_some(),
            resolved_path: agy_path.map(|p| p.to_string_lossy().to_string()),
        },
        CliAgentInfo {
            id: "cli_claude".to_string(),
            name: "Claude Code".to_string(),
            command: "claude".to_string(),
            installed: claude_path.is_some(),
            resolved_path: claude_path.map(|p| p.to_string_lossy().to_string()),
        },
        CliAgentInfo {
            id: "cli_codex".to_string(),
            name: "OpenAI Codex CLI".to_string(),
            command: "codex".to_string(),
            installed: codex_path.is_some(),
            resolved_path: codex_path.map(|p| p.to_string_lossy().to_string()),
        },
        CliAgentInfo {
            id: "cli_custom".to_string(),
            name: "Agente CLI Custom".to_string(),
            command: custom_trim.to_string(),
            installed: custom_path.is_some(),
            resolved_path: custom_path.map(|p| p.to_string_lossy().to_string()),
        },
    ]
}

/// Invoca un agente CLI esterno (`agy`, `claude`, `codex`, o custom) in modalità headless.
/// Se viene fornita un'immagine, la salva su un file temporaneo (già passato come Path) e allega il percorso.
pub fn run_cli_agent(
    provider: &str,
    custom_cli_command: &str,
    full_prompt: &str,
    image_path: Option<&Path>,
) -> Result<String, String> {
    let prompt_with_image = if let Some(img) = image_path {
        format!(
            "{}\n\n[IMMAGINE ALLEGATA SU DISCO: {} — Analizza questa immagine usando i tuoi strumenti di lettura/visione file prima di rispondere.]",
            full_prompt,
            img.to_string_lossy()
        )
    } else {
        full_prompt.to_string()
    };

    let mut cmd = match provider {
        "cli_agy" => {
            let exe = resolve_executable("agy")
                .ok_or_else(|| "Antigravity CLI ('agy') non trovato nel PATH.".to_string())?;
            let mut c = Command::new(exe);
            c.arg("-p").arg(&prompt_with_image);
            c
        }
        "cli_claude" => {
            let exe = resolve_executable("claude")
                .ok_or_else(|| "Claude Code ('claude') non trovato nel PATH.".to_string())?;
            let mut c = Command::new(exe);
            c.arg("-p").arg(&prompt_with_image).arg("--output-format").arg("text");
            if let Some(img) = image_path {
                if let Some(parent) = img.parent() {
                    c.arg("--add-dir").arg(parent);
                }
            }
            c
        }
        "cli_codex" => {
            let exe = resolve_executable("codex")
                .ok_or_else(|| "Codex CLI ('codex') non trovato nel PATH.".to_string())?;
            let mut c = Command::new(exe);
            c.arg("exec");
            if let Some(img) = image_path {
                c.arg("-i").arg(img);
            }
            c.arg(&prompt_with_image);
            c
        }
        "cli_custom" => {
            let raw = custom_cli_command.trim();
            if raw.is_empty() {
                return Err("Comando CLI Custom non configurato nelle Impostazioni.".to_string());
            }
            let parts: Vec<&str> = raw.split_whitespace().collect();
            let bin = parts[0];
            let exe = resolve_executable(bin).unwrap_or_else(|| PathBuf::from(bin));
            let mut c = Command::new(exe);
            let mut replaced_prompt = false;
            for arg in parts.iter().skip(1) {
                if *arg == "{prompt}" {
                    c.arg(&prompt_with_image);
                    replaced_prompt = true;
                } else if *arg == "{image}" {
                    if let Some(img) = image_path {
                        c.arg(img);
                    }
                } else {
                    c.arg(arg);
                }
            }
            if !replaced_prompt {
                c.arg(&prompt_with_image);
            }
            c
        }
        _ => return Err(format!("Provider CLI non riconosciuto: {}", provider)),
    };

    apply_no_window(&mut cmd);

    let output = cmd
        .output()
        .map_err(|e| format!("Errore nell'avvio dell'agente CLI ({}): {}", provider, e))?;

    let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();

    if !output.status.success() {
        let detail = if !stderr.is_empty() { stderr } else { stdout };
        return Err(format!(
            "L'agente CLI {} ha terminato con errore ({}): {}",
            provider,
            output.status,
            detail.chars().take(400).collect::<String>()
        ));
    }

    if stdout.is_empty() && !stderr.is_empty() {
        return Ok(stderr);
    }
    if stdout.is_empty() {
        return Err(format!("L'agente CLI {} non ha prodotto alcun output.", provider));
    }

    Ok(stdout)
}
