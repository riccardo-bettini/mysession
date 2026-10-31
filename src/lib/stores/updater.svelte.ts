import { check, type Update } from "@tauri-apps/plugin-updater";
import { invoke } from "@tauri-apps/api/core";

export type UpdateStatus =
  | "idle"
  | "checking"
  | "available"
  | "downloading"
  | "ready"
  | "up-to-date"
  | "error";

class UpdaterStore {
  status = $state<UpdateStatus>("idle");
  availableVersion = $state<string | null>(null);
  releaseDate = $state<string | null>(null);
  releaseNotes = $state<string | null>(null);
  downloadProgress = $state<number>(0);
  downloadedBytes = $state<number>(0);
  totalBytes = $state<number>(0);
  lastCheckDate = $state<string | null>(null);
  errorMessage = $state<string | null>(null);
  isBannerDismissed = $state<boolean>(false);

  private activeUpdate: Update | null = null;

  async checkForUpdates(silent = false): Promise<boolean> {
    if (this.status === "checking" || this.status === "downloading") return false;

    this.status = "checking";
    this.errorMessage = null;

    try {
      const update = await check();
      this.lastCheckDate = new Date().toLocaleTimeString("it-IT", {
        hour: "2-digit",
        minute: "2-digit"
      });

      if (update && update.available) {
        this.activeUpdate = update;
        this.availableVersion = update.version;
        this.releaseDate = update.date ?? null;
        this.releaseNotes = update.body ?? null;
        this.status = "available";
        this.isBannerDismissed = false;
        return true;
      } else {
        this.activeUpdate = null;
        this.status = "up-to-date";
        return false;
      }
    } catch (err: unknown) {
      const msg = err instanceof Error ? err.message : String(err);
      this.errorMessage = msg;
      this.status = "error";
      if (!silent) {
        console.warn("Verifica aggiornamenti fallita:", err);
      }
      return false;
    }
  }

  async downloadAndInstall(): Promise<void> {
    if (!this.activeUpdate) return;
    this.status = "downloading";
    this.downloadProgress = 0;
    this.errorMessage = null;

    try {
      let downloaded = 0;
      let total = 0;

      await this.activeUpdate.downloadAndInstall((event) => {
        switch (event.event) {
          case "Started":
            total = event.data.contentLength ?? 0;
            this.totalBytes = total;
            break;
          case "Progress":
            downloaded += event.data.chunkLength;
            this.downloadedBytes = downloaded;
            if (total > 0) {
              this.downloadProgress = Math.min(100, Math.round((downloaded / total) * 100));
            }
            break;
          case "Finished":
            this.downloadProgress = 100;
            break;
        }
      });

      this.status = "ready";
    } catch (err: unknown) {
      const msg = err instanceof Error ? err.message : String(err);
      this.errorMessage = msg;
      this.status = "error";
      console.error("Installazione aggiornamento non riuscita:", err);
    }
  }

  async relaunch(): Promise<void> {
    try {
      await invoke("relaunch_app");
    } catch (err) {
      console.error("Errore durante il riavvio:", err);
      if (typeof window !== "undefined") {
        window.location.reload();
      }
    }
  }

  dismissBanner(): void {
    this.isBannerDismissed = true;
  }
}

export const updaterStore = new UpdaterStore();
