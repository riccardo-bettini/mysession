import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { ipc } from "../api/ipc";
import type {
  AppSettings,
  BackdropMaterial,
  CalendarViewMode,
  MaxButtonBounds,
  ThemeChoice,
  WindowControlsStyle
} from "../types/domain";

export const UI_SCALE_MIN = 80;
export const UI_SCALE_MAX = 150;
export const UI_SCALE_STEP = 5;

function clampScalePercent(value: number): number {
  const rounded = Math.round(value / UI_SCALE_STEP) * UI_SCALE_STEP;
  return Math.min(UI_SCALE_MAX, Math.max(UI_SCALE_MIN, rounded));
}

class ThemeAndWindowStore {
  settings = $state<AppSettings>({
    theme: "auto",
    backdropMaterial: "mica",
    windowControlsStyle: "macos-left",
    windowFrameStyle: "custom",
    uiScale: 100,
    sidebarState: "expanded",
    defaultView: "continuous",
    startDayOfWeek: 1,
    weeklyHolidayDay: 0,
    studyHoursPerCfu: 15.0,
    accentColor: "#007AFF",
    holidayColor: "#FF3B30",
    sundayColor: "#FF9500",
    catStudioColor: "#007AFF",
    catScadenzaColor: "#FF9500",
    catLezioneColor: "#AF52DE",
    catPersonaleColor: "#34C759",
    aiProvider: "local",
    aiApiKey: "",
    aiEndpointUrl: "http://127.0.0.1:11434",
    aiModel: "gemini-2.5-flash",
    aiBarWidth: 760,
    activeSessionId: null,
    planFolder: ""
  });

  systemIsDark = $state<boolean>(true);
  isMaximized = $state<boolean>(false);
  isInitialized = $state<boolean>(false);

  effectiveTheme = $derived<"dark" | "light">(
    this.settings.theme === "dark"
      ? "dark"
      : this.settings.theme === "light"
        ? "light"
        : this.systemIsDark
          ? "dark"
          : "light"
  );

  uiScalePercent = $derived<number>(clampScalePercent(this.settings.uiScale));

  private unlistenThemeEvent: UnlistenFn | null = null;
  private unlistenResizeEvent: UnlistenFn | null = null;

  async init(initialSettings: AppSettings): Promise<void> {
    this.settings = {
      ...initialSettings,
      uiScale: clampScalePercent(initialSettings.uiScale || 100)
    };

    try {
      const themeInfo = await ipc.getSystemThemeInfo();
      this.systemIsDark = themeInfo.systemIsDark;
    } catch {
      if (typeof window !== "undefined" && window.matchMedia) {
        this.systemIsDark = window.matchMedia("(prefers-color-scheme: dark)").matches;
      }
    }

    this.applyDomAttributes();

    try {
      await ipc.applyWindowMaterial(this.settings.backdropMaterial, this.settings.theme);
    } catch {
      // Fallback silenzioso fuori dal runtime Tauri
    }

    await this.setupNativeListeners();
    this.isInitialized = true;
  }

  applyDomAttributes(): void {
    if (typeof document === "undefined") return;
    const root = document.documentElement;
    const scalePct = clampScalePercent(this.settings.uiScale);
    const scaleRatio = scalePct / 100;

    root.setAttribute("data-theme", this.effectiveTheme);
    root.setAttribute("data-theme-mode", this.settings.theme);
    root.setAttribute("data-material", this.settings.backdropMaterial);
    root.setAttribute("data-controls", this.settings.windowControlsStyle);
    root.style.setProperty("--ui-scale", scaleRatio.toFixed(2));
    root.style.setProperty("--accent-primary", this.settings.accentColor || "#0A84FF");
    root.style.fontSize = `${(16 * scaleRatio).toFixed(2)}px`;
  }

  private async setupNativeListeners(): Promise<void> {
    if (typeof window === "undefined") return;

    // 1. Listener per WM_SETTINGCHANGE da Rust (cambio tema del registro di Windows 11)
    try {
      if (this.unlistenThemeEvent) this.unlistenThemeEvent();
      this.unlistenThemeEvent = await listen<boolean>("system-theme-changed", async (event) => {
        this.systemIsDark = Boolean(event.payload);
        this.applyDomAttributes();
        if (this.settings.theme === "auto") {
          await ipc.applyWindowMaterial(this.settings.backdropMaterial, "auto");
        }
      });
    } catch {
      // Non in contesto Tauri
    }

    // 2. Listener stato massimizzazione della finestra Tauri
    try {
      const appWindow = getCurrentWindow();
      this.isMaximized = await appWindow.isMaximized();
      if (this.unlistenResizeEvent) this.unlistenResizeEvent();
      this.unlistenResizeEvent = await appWindow.onResized(async () => {
        try {
          this.isMaximized = await appWindow.isMaximized();
        } catch {
          // ignore
        }
      });
    } catch {
      // Non in contesto Tauri
    }

    // 3. MediaQuery fallback per prefers-color-scheme
    if (window.matchMedia) {
      const mq = window.matchMedia("(prefers-color-scheme: dark)");
      mq.addEventListener("change", (e) => {
        this.systemIsDark = e.matches;
        this.applyDomAttributes();
      });
    }
  }

  async persistAndSync(): Promise<void> {
    this.applyDomAttributes();
    try {
      const saved = await ipc.saveAppSettings(this.settings);
      this.settings = saved;
    } catch {
      // Ignora errori in modalità offline/test
    }
  }

  async setThemeMode(mode: ThemeChoice): Promise<void> {
    this.settings.theme = mode;
    await this.persistAndSync();
  }

  async setTheme(mode: ThemeChoice): Promise<void> {
    await this.setThemeMode(mode);
  }

  async cycleThemeMode(): Promise<void> {
    const order: ThemeChoice[] = ["auto", "dark", "light"];
    const idx = order.indexOf(this.settings.theme);
    const next = order[(idx + 1) % order.length];
    await this.setThemeMode(next);
  }

  async setWindowMaterial(material: BackdropMaterial): Promise<void> {
    this.settings.backdropMaterial = material;
    await this.persistAndSync();
  }

  async setBackdropMaterial(material: BackdropMaterial): Promise<void> {
    await this.setWindowMaterial(material);
  }

  async setWindowControlsStyle(style: WindowControlsStyle): Promise<void> {
    this.settings.windowControlsStyle = style;
    await this.persistAndSync();
  }

  async toggleWindowControlsStyle(): Promise<void> {
    const next: WindowControlsStyle =
      this.settings.windowControlsStyle === "macos-left" ? "win11-right" : "macos-left";
    await this.setWindowControlsStyle(next);
  }

  async setUiScale(scalePercent: number): Promise<void> {
    this.settings.uiScale = clampScalePercent(scalePercent);
    await this.persistAndSync();
  }

  async zoomIn(): Promise<void> {
    await this.setUiScale(this.settings.uiScale + UI_SCALE_STEP);
  }

  async zoomOut(): Promise<void> {
    await this.setUiScale(this.settings.uiScale - UI_SCALE_STEP);
  }

  async resetZoom(): Promise<void> {
    await this.setUiScale(100);
  }

  async setActiveView(view: CalendarViewMode): Promise<void> {
    this.settings.defaultView = view;
    await this.persistAndSync();
  }

  async setDefaultView(view: CalendarViewMode): Promise<void> {
    await this.setActiveView(view);
  }

  async updateSettings(partial: Partial<AppSettings>): Promise<void> {
    this.settings = {
      ...this.settings,
      ...partial
    };
    await this.persistAndSync();
  }

  async registerMaximizeButtonElement(el: HTMLElement | null): Promise<void> {
    if (!el || typeof window === "undefined") return;
    const rect = el.getBoundingClientRect();
    const dpr = window.devicePixelRatio || 1;
    const bounds: MaxButtonBounds = {
      x: Math.round(rect.left * dpr),
      y: Math.round(rect.top * dpr),
      width: Math.round(rect.width * dpr),
      height: Math.round(rect.height * dpr)
    };
    try {
      await ipc.setMaximizeButtonBounds(bounds);
    } catch {
      // ignore outside Tauri
    }
  }

  async minimizeWindow(): Promise<void> {
    try {
      await getCurrentWindow().minimize();
    } catch {
      // ignore
    }
  }

  async toggleMaximizeWindow(): Promise<void> {
    try {
      const win = getCurrentWindow();
      await win.toggleMaximize();
      this.isMaximized = await win.isMaximized();
    } catch {
      // ignore
    }
  }

  async closeWindow(): Promise<void> {
    try {
      await getCurrentWindow().close();
    } catch {
      // ignore
    }
  }
}

export const themeStore = new ThemeAndWindowStore();
