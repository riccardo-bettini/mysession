use serde::{Deserialize, Serialize};
use std::sync::atomic::{AtomicBool, AtomicI32, Ordering};
use tauri::{AppHandle, Emitter, WebviewWindow};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SystemThemeState {
    pub system_is_dark: bool,
    pub effective_is_dark: bool,
    pub active_material: String,
    pub active_theme_mode: String,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MaxButtonBounds {
    pub x: i32,
    pub y: i32,
    pub width: i32,
    pub height: i32,
}

// Coordinate fisiche (in pixel schermo relativi al client/window origin) del pulsante Massimizza
// utilizzate dall'intercettore WM_NCHITTEST -> HTMAXBUTTON per Windows 11 Snap Layouts.
static MAX_BTN_X: AtomicI32 = AtomicI32::new(56);
static MAX_BTN_Y: AtomicI32 = AtomicI32::new(10);
static MAX_BTN_W: AtomicI32 = AtomicI32::new(16);
static MAX_BTN_H: AtomicI32 = AtomicI32::new(16);
static MAX_BTN_ENABLED: AtomicBool = AtomicBool::new(true);

pub fn update_maximize_button_bounds(bounds: MaxButtonBounds) {
    MAX_BTN_X.store(bounds.x.max(0), Ordering::Relaxed);
    MAX_BTN_Y.store(bounds.y.max(0), Ordering::Relaxed);
    MAX_BTN_W.store(bounds.width.max(8), Ordering::Relaxed);
    MAX_BTN_H.store(bounds.height.max(8), Ordering::Relaxed);
    MAX_BTN_ENABLED.store(true, Ordering::Relaxed);
}

/// Legge dal Registro di Windows (`HKCU\Software\Microsoft\Windows\CurrentVersion\Themes\Personalize`)
/// il valore `AppsUseLightTheme` per determinare se Windows 11 è in modalità Scura.
#[cfg(windows)]
pub fn read_windows_system_dark_mode() -> bool {
    use windows::core::w;
    use windows::Win32::System::Registry::{
        RegGetValueW, HKEY_CURRENT_USER, RRF_RT_REG_DWORD,
    };

    let mut data: u32 = 1; // Default: Light (1)
    let mut data_size: u32 = std::mem::size_of::<u32>() as u32;

    let status = unsafe {
        RegGetValueW(
            HKEY_CURRENT_USER,
            w!("Software\\Microsoft\\Windows\\CurrentVersion\\Themes\\Personalize"),
            w!("AppsUseLightTheme"),
            RRF_RT_REG_DWORD,
            None,
            Some((&mut data as *mut u32).cast()),
            Some(&mut data_size),
        )
    };

    if status.is_ok() {
        data == 0
    } else {
        true // Fallback estetico dark
    }
}

#[cfg(not(windows))]
pub fn read_windows_system_dark_mode() -> bool {
    true
}

pub fn resolve_effective_dark_mode(theme_mode: &str) -> bool {
    match theme_mode.trim().to_lowercase().as_str() {
        "dark" => true,
        "light" => false,
        _ => read_windows_system_dark_mode(),
    }
}

pub fn get_system_theme_state(theme_mode: &str, window_material: &str) -> SystemThemeState {
    let system_is_dark = read_windows_system_dark_mode();
    let effective_is_dark = resolve_effective_dark_mode(theme_mode);
    SystemThemeState {
        system_is_dark,
        effective_is_dark,
        active_material: window_material.to_string(),
        active_theme_mode: theme_mode.to_string(),
    }
}

/// Applica gli attributi nativi DWM di Windows 11:
/// - `DWMWA_USE_IMMERSIVE_DARK_MODE` (20)
/// - `DWMWA_WINDOW_CORNER_PREFERENCE` (33 -> DWMWCP_ROUND)
/// - `DWMWA_SYSTEMBACKDROP_TYPE` (38 -> Mica / Acrylic / Tabbed / None)
/// - `DwmExtendFrameIntoClientArea` con margini -1 per ombre native e superficie traslucida continua.
#[cfg(windows)]
pub fn apply_window_backdrop_and_theme(
    window: &WebviewWindow,
    window_material: &str,
    theme_mode: &str,
) -> Result<SystemThemeState, String> {
    use raw_window_handle::{HasWindowHandle, RawWindowHandle};
    use windows::Win32::Foundation::{BOOL, HWND};
    use windows::Win32::Graphics::Dwm::{
        DwmExtendFrameIntoClientArea, DwmSetWindowAttribute, DWMSBT_MAINWINDOW, DWMSBT_NONE,
        DWMSBT_TABBEDWINDOW, DWMSBT_TRANSIENTWINDOW, DWMWA_SYSTEMBACKDROP_TYPE,
        DWMWA_USE_IMMERSIVE_DARK_MODE, DWMWA_WINDOW_CORNER_PREFERENCE, DWMWCP_ROUND,
        DWM_SYSTEMBACKDROP_TYPE, DWM_WINDOW_CORNER_PREFERENCE,
    };
    use windows::Win32::UI::Controls::MARGINS;

    let effective_dark = resolve_effective_dark_mode(theme_mode);

    let handle = window
        .window_handle()
        .map_err(|e| format!("Impossibile ottenere window handle: {e}"))?;

    let hwnd = match handle.as_raw() {
        RawWindowHandle::Win32(win32_handle) => HWND(win32_handle.hwnd.get() as *mut _),
        _ => return Err("Handle finestra non è Win32".to_string()),
    };

    unsafe {
        // 1. Estende il frame DWM nell'intera area client per preservare ombre native e trasparenza
        let margins = MARGINS {
            cxLeftWidth: -1,
            cxRightWidth: -1,
            cyTopHeight: -1,
            cyBottomHeight: -1,
        };
        let _ = DwmExtendFrameIntoClientArea(hwnd, &margins);

        // 2. Angoli arrotondati nativi Windows 11 con continuità d'ombra
        let corner_pref: DWM_WINDOW_CORNER_PREFERENCE = DWMWCP_ROUND;
        let _ = DwmSetWindowAttribute(
            hwnd,
            DWMWA_WINDOW_CORNER_PREFERENCE,
            (&corner_pref as *const DWM_WINDOW_CORNER_PREFERENCE).cast(),
            std::mem::size_of::<DWM_WINDOW_CORNER_PREFERENCE>() as u32,
        );

        // 3. Sincronizza Dark / Light mode con DWM
        let dark_bool = BOOL::from(effective_dark);
        let _ = DwmSetWindowAttribute(
            hwnd,
            DWMWA_USE_IMMERSIVE_DARK_MODE,
            (&dark_bool as *const BOOL).cast(),
            std::mem::size_of::<BOOL>() as u32,
        );

        // 4. Pulisce eventuali effetti vibrancy precedenti prima di applicare il nuovo materiale
        let _ = window_vibrancy::clear_mica(window);
        let _ = window_vibrancy::clear_acrylic(window);
        let _ = window_vibrancy::clear_tabbed(window);

        // 5. Inietta DWMWA_SYSTEMBACKDROP_TYPE + window_vibrancy fallback/tuning
        let material_norm = window_material.trim().to_lowercase();
        let backdrop_type: DWM_SYSTEMBACKDROP_TYPE = match material_norm.as_str() {
            "acrylic" => DWMSBT_TRANSIENTWINDOW,
            "tabbed" => DWMSBT_TABBEDWINDOW,
            "solid" => DWMSBT_NONE,
            _ => DWMSBT_MAINWINDOW, // Default: "mica"
        };

        let _ = DwmSetWindowAttribute(
            hwnd,
            DWMWA_SYSTEMBACKDROP_TYPE,
            (&backdrop_type as *const DWM_SYSTEMBACKDROP_TYPE).cast(),
            std::mem::size_of::<DWM_SYSTEMBACKDROP_TYPE>() as u32,
        );

        match material_norm.as_str() {
            "acrylic" => {
                let tint = if effective_dark {
                    Some((18, 18, 24, 175))
                } else {
                    Some((245, 245, 250, 185))
                };
                let _ = window_vibrancy::apply_acrylic(window, tint);
            }
            "tabbed" => {
                let _ = window_vibrancy::apply_tabbed(window, Some(effective_dark));
            }
            "solid" => {}
            _ => {
                let _ = window_vibrancy::apply_mica(window, Some(effective_dark));
            }
        }
    }

    Ok(get_system_theme_state(theme_mode, window_material))
}

#[cfg(not(windows))]
pub fn apply_window_backdrop_and_theme(
    _window: &WebviewWindow,
    window_material: &str,
    theme_mode: &str,
) -> Result<SystemThemeState, String> {
    Ok(get_system_theme_state(theme_mode, window_material))
}

/// Installa una Subclass Win32 (`SetWindowSubclass`) sulla finestra principale per:
/// 1. Intercettare `WM_NCHITTEST` e restituire `HTMAXBUTTON` (9) quando il cursore si trova sopra
///    il pulsante Massimizza della Unified Toolbar, abilitando i **Windows 11 Snap Layouts** nativi.
/// 2. Intercettare `WM_SETTINGCHANGE` (`ImmersiveColorSet`) per rilevare in tempo reale il cambio
///    del tema di Windows 11 e notificare il frontend tramite l'evento `"system-theme-changed"`.
#[cfg(windows)]
pub fn install_window_subclass(window: &WebviewWindow, app_handle: AppHandle) -> Result<(), String> {
    use raw_window_handle::{HasWindowHandle, RawWindowHandle};
    use windows::Win32::Foundation::{HWND, LPARAM, LRESULT, RECT, WPARAM};
    use windows::Win32::UI::Shell::{DefSubclassProc, SetWindowSubclass};
    use windows::Win32::UI::WindowsAndMessaging::{
        GetWindowRect, HTMAXBUTTON, WM_NCHITTEST, WM_SETTINGCHANGE,
    };

    let handle = window
        .window_handle()
        .map_err(|e| format!("Impossibile ottenere window handle per subclass: {e}"))?;

    let hwnd = match handle.as_raw() {
        RawWindowHandle::Win32(win32_handle) => HWND(win32_handle.hwnd.get() as *mut _),
        _ => return Err("Handle finestra non è Win32".to_string()),
    };

    let boxed_app = Box::new(app_handle);
    let app_ptr = Box::into_raw(boxed_app) as usize;

    unsafe extern "system" fn subclass_proc(
        hwnd: HWND,
        u_msg: u32,
        w_param: WPARAM,
        l_param: LPARAM,
        _u_id_subclass: usize,
        dw_ref_data: usize,
    ) -> LRESULT {
        match u_msg {
            WM_NCHITTEST => {
                let base_res = DefSubclassProc(hwnd, u_msg, w_param, l_param);
                if MAX_BTN_ENABLED.load(Ordering::Relaxed) {
                    let screen_x = (l_param.0 & 0xFFFF) as i16 as i32;
                    let screen_y = ((l_param.0 >> 16) & 0xFFFF) as i16 as i32;

                    let mut win_rect = RECT::default();
                    if GetWindowRect(hwnd, &mut win_rect).is_ok() {
                        let rel_x = screen_x - win_rect.left;
                        let rel_y = screen_y - win_rect.top;

                        let bx = MAX_BTN_X.load(Ordering::Relaxed);
                        let by = MAX_BTN_Y.load(Ordering::Relaxed);
                        let bw = MAX_BTN_W.load(Ordering::Relaxed);
                        let bh = MAX_BTN_H.load(Ordering::Relaxed);

                        if rel_x >= bx && rel_x <= (bx + bw) && rel_y >= by && rel_y <= (by + bh) {
                            return LRESULT(HTMAXBUTTON as isize);
                        }
                    }
                }
                base_res
            }
            WM_SETTINGCHANGE => {
                if dw_ref_data != 0 {
                    let app = &*(dw_ref_data as *const AppHandle);
                    let system_is_dark = read_windows_system_dark_mode();
                    let _ = app.emit("system-theme-changed", system_is_dark);
                }
                DefSubclassProc(hwnd, u_msg, w_param, l_param)
            }
            _ => DefSubclassProc(hwnd, u_msg, w_param, l_param),
        }
    }

    unsafe {
        let ok = SetWindowSubclass(hwnd, Some(subclass_proc), 0x4D59_5353, app_ptr);
        if !ok.as_bool() {
            return Err("SetWindowSubclass fallita".to_string());
        }
    }

    let _ = window.set_shadow(true);
    Ok(())
}

#[cfg(not(windows))]
pub fn install_window_subclass(_window: &WebviewWindow, _app_handle: AppHandle) -> Result<(), String> {
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_resolve_effective_dark_mode_explicit_and_auto() {
        assert!(resolve_effective_dark_mode("dark"));
        assert!(!resolve_effective_dark_mode("light"));
        // "auto" reads from Windows registry without panicking
        let auto_mode = resolve_effective_dark_mode("auto");
        let sys_mode = read_windows_system_dark_mode();
        assert_eq!(auto_mode, sys_mode);
    }

    #[test]
    fn test_update_maximize_button_bounds() {
        update_maximize_button_bounds(MaxButtonBounds {
            x: 1380,
            y: 8,
            width: 36,
            height: 28,
        });
        assert_eq!(MAX_BTN_X.load(Ordering::Relaxed), 1380);
        assert_eq!(MAX_BTN_Y.load(Ordering::Relaxed), 8);
        assert_eq!(MAX_BTN_W.load(Ordering::Relaxed), 36);
        assert_eq!(MAX_BTN_H.load(Ordering::Relaxed), 28);
    }
}
