import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import { open } from "@tauri-apps/plugin-dialog";
import { ipc } from "../api/ipc";
import { sessionStore } from "./session.svelte";
import { themeStore } from "./theme.svelte";
import type {
  PendingPlanNotification,
  PlanDiffSummary
} from "../types/domain";

class PlanWorkspaceStore {
  pendingPlans = $state<PendingPlanNotification[]>([]);
  activePreview = $state<PlanDiffSummary | null>(null);
  activePreviewFilePath = $state<string | null>(null);
  activeConflictResolutions = $state<Record<string, "keep_mine" | "use_theirs">>({});
  isUpdateModalOpen = $state<boolean>(false);
  isScanning = $state<boolean>(false);
  planFolder = $state<string>("");
  lastError = $state<string | null>(null);

  private unlistenPending: UnlistenFn | null = null;

  async init(): Promise<void> {
    this.planFolder = themeStore.settings.planFolder || "";

    if (this.unlistenPending) {
      this.unlistenPending();
      this.unlistenPending = null;
    }

    try {
      this.unlistenPending = await listen<PendingPlanNotification[]>(
        "plan://pending",
        (event) => {
          this.pendingPlans = event.payload || [];
        }
      );
    } catch (e) {
      console.warn("Impossibile registrare il listener plan://pending", e);
    }

    if (this.planFolder) {
      await this.scan();
    }
  }

  async setFolder(path: string): Promise<void> {
    try {
      const updated = await ipc.setPlanFolder(path);
      this.planFolder = updated.planFolder;
      themeStore.settings.planFolder = updated.planFolder;
      await this.scan();
    } catch (e) {
      this.lastError = e instanceof Error ? e.message : String(e);
      throw e;
    }
  }

  async browseFolder(): Promise<string | null> {
    try {
      const selected = await open({
        directory: true,
        multiple: false,
        title: "Seleziona cartella condivisa dei Piani di Studio"
      });
      if (typeof selected === "string" && selected.trim()) {
        await this.setFolder(selected.trim());
        return selected.trim();
      }
      return null;
    } catch (e) {
      this.lastError = e instanceof Error ? e.message : String(e);
      return null;
    }
  }

  async openPlanFolder(): Promise<void> {
    try {
      let targetPath = this.planFolder?.trim();
      if (!targetPath) {
        const selected = await this.browseFolder();
        if (!selected) return;
        targetPath = selected;
      }
      await ipc.openFolderInExplorer(targetPath);
    } catch (e) {
      this.lastError = e instanceof Error ? e.message : String(e);
      console.error("Errore durante l'apertura della cartella dei piani:", e);
    }
  }

  async scan(): Promise<PendingPlanNotification[]> {
    this.isScanning = true;
    this.lastError = null;
    try {
      const plans = await ipc.scanPendingPlans();
      this.pendingPlans = plans;
      return plans;
    } catch (e) {
      this.lastError = e instanceof Error ? e.message : String(e);
      return [];
    } finally {
      this.isScanning = false;
    }
  }

  async openPreview(plan: PendingPlanNotification): Promise<void> {
    this.lastError = null;
    try {
      const diff = await ipc.previewPlanUpdate(plan.filePath);
      this.activePreview = diff;
      this.activePreviewFilePath = plan.filePath;
      const res: Record<string, "keep_mine" | "use_theirs"> = {};
      for (const c of diff.conflicts) {
        res[c.taskId] = "use_theirs";
      }
      this.activeConflictResolutions = res;
      this.isUpdateModalOpen = true;
    } catch (e) {
      this.lastError = e instanceof Error ? e.message : String(e);
      throw e;
    }
  }

  closePreview(): void {
    this.isUpdateModalOpen = false;
    this.activePreview = null;
    this.activePreviewFilePath = null;
    this.activeConflictResolutions = {};
  }

  setConflictResolution(taskId: string, choice: "keep_mine" | "use_theirs"): void {
    this.activeConflictResolutions = {
      ...this.activeConflictResolutions,
      [taskId]: choice
    };
  }

  async applyActivePlan(targetSessionId?: string | null): Promise<void> {
    if (!this.activePreviewFilePath) return;
    try {
      const snap = await ipc.applyPlanUpdate(
        this.activePreviewFilePath,
        this.activeConflictResolutions,
        targetSessionId
      );
      sessionStore.activeSnapshot = snap;
      const bootstrap = await ipc.bootstrapApp();
      sessionStore.applyBootstrapPayload(bootstrap);
      this.closePreview();
      await this.scan();
    } catch (e) {
      this.lastError = e instanceof Error ? e.message : String(e);
      throw e;
    }
  }
}

export const planStore = new PlanWorkspaceStore();
