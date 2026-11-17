import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import { ipc } from "../api/ipc";
import type {
  AcademicSession,
  HolidayInfo,
  PendingPlanNotification,
  PlanDiffSummary,
  SessionSnapshot,
  SubmitQuizAttemptInput,
  UpsertCalendarEventInput,
  UpsertCourseInput,
  UpsertExamInput,
  UpsertSessionInput,
  UpsertStudyUnitInput,
  UpsertTaskInput,
  UpsertTaskSubItemInput,
  UpsertWeeklyLessonInput
} from "../types/domain";
import { navStore } from "./navigation.svelte";
import { themeStore } from "./theme.svelte";
import { updaterStore } from "./updater.svelte";

class SessionStore {
  isBootstrapping = $state<boolean>(true);
  isBusy = $state<boolean>(false);
  dbPath = $state<string>("");
  sessions = $state<AcademicSession[]>([]);
  activeSnapshot = $state<SessionSnapshot | null>(null);
  holidays = $state<HolidayInfo[]>([]);
  pendingPlans = $state<PendingPlanNotification[]>([]);

  private unlistenPlans: UnlistenFn | null = null;

  activeSession = $derived<AcademicSession | null>(this.activeSnapshot?.session ?? null);

  async bootstrap(): Promise<void> {
    this.isBootstrapping = true;
    try {
      const payload = await ipc.bootstrapApp();
      this.dbPath = payload.dbPath;
      this.sessions = payload.sessions;
      this.activeSnapshot = payload.activeSnapshot;
      this.pendingPlans = payload.pendingPlans ?? [];
      await themeStore.init(payload.settings);

      const year = new Date().getFullYear();
      this.holidays = await ipc.getHolidaysRange(year - 1, year + 2);

      this.unlistenPlans?.();
      this.unlistenPlans = await listen<PendingPlanNotification[]>("plan://pending", (ev) => {
        if (Array.isArray(ev.payload)) {
          this.pendingPlans = ev.payload;
        }
      });

      setTimeout(() => {
        void updaterStore.checkForUpdates(true);
      }, 2500);
    } catch (err) {
      navStore.showToast(`Errore avvio: ${String(err)}`, "error");
    } finally {
      this.isBootstrapping = false;
    }
  }

  async refreshActiveSnapshot(): Promise<void> {
    const sid = this.activeSnapshot?.session.id ?? themeStore.settings.activeSessionId;
    if (!sid) {
      this.activeSnapshot = null;
      return;
    }
    this.activeSnapshot = await ipc.getSessionSnapshot(sid);
  }

  async selectSession(sessionId: string): Promise<void> {
    this.isBusy = true;
    try {
      const snap = await ipc.setActiveAcademicSession(sessionId);
      this.activeSnapshot = snap;
      themeStore.settings = { ...themeStore.settings, activeSessionId: snap.session.id };
      navStore.isSessionPopoverOpen = false;
    } catch (err) {
      navStore.showToast(`Impossibile cambiare sessione: ${String(err)}`, "error");
    } finally {
      this.isBusy = false;
    }
  }

  async createOrUpdateSession(input: UpsertSessionInput): Promise<AcademicSession | null> {
    this.isBusy = true;
    try {
      const sess = await ipc.upsertAcademicSession(input);
      const snap = await ipc.setActiveAcademicSession(sess.id);
      const exists = this.sessions.some((s) => s.id === sess.id);
      this.sessions = exists
        ? this.sessions.map((s) => (s.id === sess.id ? sess : s))
        : [sess, ...this.sessions];
      this.activeSnapshot = snap;
      themeStore.settings = { ...themeStore.settings, activeSessionId: sess.id };
      navStore.showToast(`Sessione "${sess.name}" salvata`, "success");
      return sess;
    } catch (err) {
      navStore.showToast(`Errore salvataggio sessione: ${String(err)}`, "error");
      return null;
    } finally {
      this.isBusy = false;
    }
  }

  async deleteSession(sessionId: string): Promise<void> {
    this.isBusy = true;
    try {
      const boot = await ipc.deleteAcademicSession(sessionId);
      this.sessions = boot.sessions;
      this.activeSnapshot = boot.activeSnapshot;
      themeStore.settings = boot.settings;
      navStore.showToast("Sessione eliminata", "info");
    } catch (err) {
      navStore.showToast(`Errore eliminazione: ${String(err)}`, "error");
    } finally {
      this.isBusy = false;
    }
  }

  // Courses & Units
  async upsertCourse(input: UpsertCourseInput): Promise<void> {
    await ipc.upsertCourse(input);
    await this.refreshActiveSnapshot();
  }

  async deleteCourse(courseId: string): Promise<void> {
    await ipc.deleteCourse(courseId);
    await this.refreshActiveSnapshot();
  }

  async upsertStudyUnit(input: UpsertStudyUnitInput): Promise<void> {
    await ipc.upsertStudyUnit(input);
    await this.refreshActiveSnapshot();
  }

  async toggleStudyUnit(unitId: string, completed: boolean): Promise<void> {
    await ipc.toggleStudyUnit(unitId, completed);
    await this.refreshActiveSnapshot();
  }

  // Exams
  async upsertExam(input: UpsertExamInput): Promise<void> {
    await ipc.upsertExam(input);
    await this.refreshActiveSnapshot();
  }

  async deleteExam(examId: string): Promise<void> {
    await ipc.deleteExam(examId);
    await this.refreshActiveSnapshot();
  }

  // Operational Blocks, Tasks & Subitems
  async toggleOperationalBlock(blockId: string, done: boolean): Promise<void> {
    await ipc.toggleOperationalBlock(blockId, done);
    await this.refreshActiveSnapshot();
  }

  async upsertTask(input: UpsertTaskInput): Promise<void> {
    await ipc.upsertTask(input);
    await this.refreshActiveSnapshot();
  }

  async toggleTask(taskId: string, done: boolean): Promise<void> {
    await ipc.toggleTask(taskId, done);
    await this.refreshActiveSnapshot();
  }

  async deleteTask(taskId: string): Promise<void> {
    await ipc.deleteTask(taskId);
    await this.refreshActiveSnapshot();
  }

  async upsertTaskSubitem(input: UpsertTaskSubItemInput): Promise<void> {
    await ipc.upsertTaskSubitem(input);
    await this.refreshActiveSnapshot();
  }

  async toggleTaskSubitem(subitemId: string, done: boolean): Promise<void> {
    await ipc.toggleTaskSubitem(subitemId, done);
    await this.refreshActiveSnapshot();
  }

  async deleteTaskSubitem(subitemId: string): Promise<void> {
    await ipc.deleteTaskSubitem(subitemId);
    await this.refreshActiveSnapshot();
  }

  // Quizzes
  async submitQuizAttempt(input: SubmitQuizAttemptInput): Promise<void> {
    await ipc.submitQuizAttempt(input);
    await this.refreshActiveSnapshot();
  }

  // Weekly Lessons & Calendar Events & Notes
  async upsertWeeklyLesson(input: UpsertWeeklyLessonInput): Promise<void> {
    await ipc.upsertWeeklyLesson(input);
    await this.refreshActiveSnapshot();
  }

  async deleteWeeklyLesson(lessonId: string): Promise<void> {
    await ipc.deleteWeeklyLesson(lessonId);
    await this.refreshActiveSnapshot();
  }

  async projectLessonsToCalendar(startDate: string, endDate: string): Promise<number> {
    if (!this.activeSnapshot) return 0;
    const count = await ipc.projectLessonsToCalendar(this.activeSnapshot.session.id, startDate, endDate);
    await this.refreshActiveSnapshot();
    return count;
  }

  async upsertCalendarEvent(input: UpsertCalendarEventInput): Promise<void> {
    await ipc.upsertCalendarEvent(input);
    await this.refreshActiveSnapshot();
  }

  async deleteCalendarEvent(eventId: string): Promise<void> {
    await ipc.deleteCalendarEvent(eventId);
    await this.refreshActiveSnapshot();
  }

  async saveDailyNote(date: string, markdown: string): Promise<void> {
    if (!this.activeSnapshot) return;
    await ipc.saveDailyNote(this.activeSnapshot.session.id, date, markdown);
    await this.refreshActiveSnapshot();
  }

  // Study Plan Folder Sync
  async setPlanFolder(folderPath: string): Promise<PendingPlanNotification[]> {
    const pending = await ipc.setPlanFolder(folderPath);
    themeStore.settings = { ...themeStore.settings, planFolder: folderPath };
    this.pendingPlans = pending;
    return pending;
  }

  async scanPendingPlans(): Promise<PendingPlanNotification[]> {
    const pending = await ipc.scanPendingPlans();
    this.pendingPlans = pending;
    return pending;
  }

  async previewPlan(filePath: string): Promise<PlanDiffSummary> {
    return await ipc.previewPlanUpdate(filePath);
  }

  async applyPlan(
    filePath: string,
    conflictResolutions: Record<string, string> = {},
    targetSessionId: string | null = null
  ): Promise<void> {
    this.isBusy = true;
    try {
      const snap = await ipc.applyPlanUpdate(filePath, conflictResolutions, targetSessionId);
      const boot = await ipc.bootstrapApp();
      this.sessions = boot.sessions;
      this.activeSnapshot = snap;
      this.pendingPlans = boot.pendingPlans;
      themeStore.settings = { ...boot.settings, activeSessionId: snap.session.id };
      navStore.pendingPlanModal = null;
      navStore.showToast(`Piano "${snap.session.name}" importato!`, "success");
    } catch (err) {
      navStore.showToast(`Errore importazione piano: ${String(err)}`, "error");
    } finally {
      this.isBusy = false;
    }
  }

  destroy(): void {
    this.unlistenPlans?.();
  }
}

export const sessionStore = new SessionStore();
