use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AiModelCatalogItem {
    pub id: String,
    pub name: String,
    pub tagline: String,
    pub tier: String,
    pub tier_level: i32,
    pub size_gb: f64,
    pub vram_req_gb: f64,
    pub recommended_ram_gb: f64,
    pub file_name: String,
    pub mmproj_file_name: String,
    pub download_url: String,
    pub mmproj_download_url: String,
    pub context_window: usize,
    pub description: String,
    pub recommended_for: String,
    pub is_downloaded: bool,
    pub is_active: bool,
}

/// Catalogo semplificato v2.00: un singolo modello locale leggero, veloce e multimodale.
pub fn get_ai_model_catalog() -> Vec<AiModelCatalogItem> {
    vec![AiModelCatalogItem {
        id: "qwen2.5-vl-3b".to_string(),
        name: "Qwen2.5-VL (3B) — Locale Base".to_string(),
        tagline: "Leggero, veloce e multimodale (testo + scansione foto orari)".to_string(),
        tier: "base".to_string(),
        tier_level: 1,
        size_gb: 2.2,
        vram_req_gb: 3.0,
        recommended_ram_gb: 8.0,
        file_name: "Qwen2.5-VL-3B-Instruct-Q4_K_M.gguf".to_string(),
        mmproj_file_name: "mmproj-Qwen2.5-VL-3B-Instruct-f16.gguf".to_string(),
        download_url: "https://huggingface.co/ggml-org/Qwen2.5-VL-3B-Instruct-GGUF/resolve/main/Qwen2.5-VL-3B-Instruct-Q4_K_M.gguf".to_string(),
        mmproj_download_url: "https://huggingface.co/ggml-org/Qwen2.5-VL-3B-Instruct-GGUF/resolve/main/mmproj-Qwen2.5-VL-3B-Instruct-f16.gguf".to_string(),
        context_window: 8192,
        description: "Modello locale base ottimizzato per avviarsi on-demand e spegnersi automaticamente dopo l'inattività. Eccellente in italiano e nella lettura di orari universitari da screenshot.".to_string(),
        recommended_for: "Uso offline rapido senza dipendenze esterne. In alternativa puoi collegare Antigravity CLI, Claude Code o Codex.".to_string(),
        is_downloaded: false,
        is_active: true,
    }]
}
