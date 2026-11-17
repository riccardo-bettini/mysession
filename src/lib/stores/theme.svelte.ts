import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { ipc } from "../api/ipc";
import type { AppSettings, MaxButtonBounds } from "../types/domain";
import { hexToRgb, normalizeHex } from "../utils/color";

export const UI_SCALE_MIN = 80;
export const UI_SCALE_MAX = 140;
export const UI_SCALE_STEP = 5;

function clampScale(v: number): number {
  return Math.min(UI_SCALE_MAX, Math.max(UI_SCALE_MIN, Math.round(v / UI_SCALE_STEP) * UI_SCALE_STEP));
}

class ThemeStore {
  settings = $state<AppSettings>({
    theme: "light",
    backdropMaterial: "mica",
    uiScale: 100,
    accentColor: "#7C5CFC",
    catTeoriaColor: "#3B82F6",
    catEserciziColor: "#10B981",
    catTemiEsameColor: "#F59E0B",
    catDimostrazioniColor: "#8B5CF6",
    catDomandeOraleColor: "#EC4899",
    sidebarSections: ["home", "calendar", "today", "courses", "quiz", "progress", "ai"],
    hiddenSections: [],
    planFolder: "",
    activeSessionId: null,
    studyHoursPerCfu: 15.0,
    startDayOfWeek: 1,
    aiProvider: "local",
    aiLocalModelId: "qwen2.5-vl-3b",
    aiCustomCliCommand: "",
    aiIdleTimeoutMinutes: 5,
  });

  systemIsDark = $state<boolean>(false);
  isMaximized = $state<boolean>(false);
  private unlistenTheme: UnlistenFn | null = null;
  private unlistenResize: UnlistenFn | null = null;

  effectiveTheme = $derived<"light" | "dark">(
    this.settings.theme === "dark"
      ? "dark"
      : this.settings.theme === "light"
        ? "light"
        : this.systemIsDark
          ? "dark"
          : "light"
  );

  async init(initial: AppSettings): Promise<void> {
    this.settings = {
      ...initial,
      uiScale: clampScale(initial.uiScale || 100)
    };

    try {
      const info = await ipc.getSystemThemeInfo();
      this.systemIsDark = info.systemIsDark;
    } catch {
      if (typeof window !== "undefined" && window.matchMedia) {
        this.systemIsDark = window.matchMedia("(prefers-color-scheme: dark)").matches;
      }
    }

    this.applyDomVariables();
    await this.syncNativeWindow();

    try {
      const win = getCurrentWindow();
      this.isMaximized = await win.isMaximized();
      this.unlistenResize?.();
      this.unlistenResize = await win.onResized(async () => {
        try {
          this.isMaximized = await win.isMaximized();
        } catch {
          // ignore
        }
      });
      this.unlistenTheme?.();
      this.unlistenTheme = await listen<boolean>("system-theme-changed", async (ev) => {
        this.systemIsDark = ev.payload;
        this.applyDomVariables();
        if (this.settings.theme === "auto") {
          await this.syncNativeWindow();
        }
      });
    } catch {
      // ignore outside Tauri
    }
  }

  applyDomVariables(): void {
    if (typeof document === "undefined") return;
    const root = document.documentElement;
    root.setAttribute("data-theme", this.effectiveTheme);
    root.style.setProperty("--ui-scale", String(clampScale(this.settings.uiScale) / 100));

    const accent = normalizeHex(this.settings.accentColor, "#7C5CFC");
    const rgb = hexToRgb(accent);
    root.style.setProperty("--accent", accent);
    root.style.setProperty("--accent-soft", `rgba(${rgb.r}, ${rgb.g}, ${rgb.b}, 0.20)`);

    root.style.setProperty("--cat-teoria", normalizeHex(this.settings.catTeoriaColor, "#3B82F6"));
    root.style.setProperty("--cat-esercizi", normalizeHex(this.settings.catEserciziColor, "#10B981"));
    root.style.setProperty("--cat-temi-esame", normalizeHex(this.settings.catTemiEsameColor, "#F59E0B"));
    root.style.setProperty("--cat-dimostrazioni", normalizeHex(this.settings.catDimostrazioniColor, "#8B5CF6"));
    root.style.setProperty("--cat-domande-orale", normalizeHex(this.settings.catDomandeOraleColor, "#EC4899"));
  }

  async syncNativeWindow(): Promise<void> {
    try {
      await ipc.applyWindowMaterial(this.settings.backdropMaterial, this.settings.theme);
    } catch {
      // ignore
    }
  }

  async updateSettings(patch: Partial<AppSettings>): Promise<AppSettings> {
    const next: AppSettings = {
      ...this.settings,
      ...patch,
      uiScale: clampScale(patch.uiScale ?? this.settings.uiScale)
    };
    this.settings = next;
    this.applyDomVariables();
    const saved = await ipc.saveAppSettings(next);
    this.settings = saved;
    this.applyDomVariables();
    return saved;
  }

  async registerMaximizeBounds(el: HTMLElement | null): Promise<void> {
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
      // ignore
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

  destroy(): void {
    this.unlistenTheme?.();
    this.unlistenResize?.();
  }
}

export const themeStore = new ThemeStore();
