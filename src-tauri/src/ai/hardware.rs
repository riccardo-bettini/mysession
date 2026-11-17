use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SystemHardwareInfo {
    pub total_ram_gb: f64,
    pub available_ram_gb: f64,
    pub gpu_name: String,
    pub dedicated_vram_gb: f64,
    pub has_dedicated_gpu: bool,
    pub recommended_model_id: String,
    pub recommended_reason: String,
}

#[cfg(windows)]
pub fn detect_system_hardware() -> SystemHardwareInfo {
    use windows::Win32::Graphics::Dxgi::{CreateDXGIFactory1, IDXGIFactory1};
    use windows::Win32::System::SystemInformation::{GlobalMemoryStatusEx, MEMORYSTATUSEX};

    let mut total_ram_gb = 16.0;
    let mut available_ram_gb = 8.0;

    unsafe {
        let mut mem_status = MEMORYSTATUSEX {
            dwLength: std::mem::size_of::<MEMORYSTATUSEX>() as u32,
            ..Default::default()
        };
        if GlobalMemoryStatusEx(&mut mem_status).is_ok() {
            total_ram_gb = (mem_status.ullTotalPhys as f64) / (1024.0 * 1024.0 * 1024.0);
            available_ram_gb = (mem_status.ullAvailPhys as f64) / (1024.0 * 1024.0 * 1024.0);
        }
    }

    let mut gpu_name = "Scheda Grafica Integrata".to_string();
    let mut dedicated_vram_gb = 0.0;
    let mut has_dedicated_gpu = false;

    unsafe {
        if let Ok(factory) = CreateDXGIFactory1::<IDXGIFactory1>() {
            let mut adapter_index = 0;
            let mut best_vram: usize = 0;
            let mut best_name = String::new();

            while let Ok(adapter) = factory.EnumAdapters1(adapter_index) {
                if let Ok(desc) = adapter.GetDesc1() {
                    let desc_str = String::from_utf16_lossy(&desc.Description)
                        .trim_matches('\0')
                        .trim()
                        .to_string();
                    if !desc_str.to_lowercase().contains("basic render")
                        && desc.DedicatedVideoMemory > best_vram
                    {
                        best_vram = desc.DedicatedVideoMemory;
                        best_name = desc_str;
                    }
                }
                adapter_index += 1;
            }

            if !best_name.is_empty() {
                gpu_name = best_name;
                dedicated_vram_gb = (best_vram as f64) / (1024.0 * 1024.0 * 1024.0);
                has_dedicated_gpu = dedicated_vram_gb >= 1.5;
            }
        }
    }

    SystemHardwareInfo {
        total_ram_gb: (total_ram_gb * 10.0).round() / 10.0,
        available_ram_gb: (available_ram_gb * 10.0).round() / 10.0,
        gpu_name: gpu_name.clone(),
        dedicated_vram_gb: (dedicated_vram_gb * 10.0).round() / 10.0,
        has_dedicated_gpu,
        recommended_model_id: "qwen2.5-vl-3b".to_string(),
        recommended_reason: format!(
            "Sistema rilevato: {:.1} GB RAM, {}. Modello locale base Qwen2.5-VL 3B pronto all'uso on-demand.",
            total_ram_gb, gpu_name
        ),
    }
}

#[cfg(not(windows))]
pub fn detect_system_hardware() -> SystemHardwareInfo {
    SystemHardwareInfo {
        total_ram_gb: 16.0,
        available_ram_gb: 8.0,
        gpu_name: "Generic GPU".to_string(),
        dedicated_vram_gb: 4.0,
        has_dedicated_gpu: true,
        recommended_model_id: "qwen2.5-vl-3b".to_string(),
        recommended_reason: "Profilo standard desktop.".to_string(),
    }
}
