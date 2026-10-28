import { ipc } from "../api/ipc";
import type {
  AcademicSession,
  AiAssistantResponse,
  BootstrapPayload,
  Course,
  CreateItinerePairInput,
  Exam,
  GenericEvent,
  HolidayInfo,
  SessionSnapshot,
  UpsertCourseInput,
  UpsertExamInput,
  UpsertGenericEventInput,
  UpsertProfileInput,
  UpsertSessionInput,
  UpsertTodoInput,
  UserProfile
} from "../types/domain";
import { APP_VERSION } from "../constants/version";
import { getTodayIso } from "../utils/calendar";
import { themeStore } from "./theme.svelte";

class SessionWorkspaceStore {
  appVersion = $state<string>(APP_VERSION);
  dbPath = $state<string>("");
  isSessionLocked = $state<boolean>(
    typeof localStorage !== "undefined" && localStorage.getItem("mysession_lock_to_session") !== null
      ? localStorage.getItem("mysession_lock_to_session") === "true"
      : true
  );
  profile = $state<UserProfile>({
    id: 1,
    studentName: "",
    university: "",
    faculty: "",
    degreeCourse: "",
    enrollmentYear: null,
    onboardingCompleted: false,
    createdAt: "",
    updatedAt: ""
  });

  sessions = $state<AcademicSession[]>([]);
  activeSnapshot = $state<SessionSnapshot | null>(null);
  holidays = $state<HolidayInfo[]>([]);
  todayIso = $state<string>(getTodayIso());
  isLoading = $state<boolean>(true);
  lastError = $state<string | null>(null);

  activeSession = $derived<AcademicSession | null>(
    this.activeSnapshot?.session ?? this.sessions[0] ?? null
  );

  courses = $derived<Course[]>(this.activeSnapshot?.courses ?? []);
  exams = $derived<Exam[]>(this.activeSnapshot?.exams ?? []);
  genericEvents = $derived<GenericEvent[]>(this.activeSnapshot?.genericEvents ?? []);

  toggleSessionLock(force?: boolean): boolean {
    this.isSessionLocked = force !== undefined ? force : !this.isSessionLocked;
    if (typeof localStorage !== "undefined") {
      localStorage.setItem("mysession_lock_to_session", String(this.isSessionLocked));
    }
    return this.isSessionLocked;
  }

  applyBootstrapPayload(payload: BootstrapPayload): void {
    this.appVersion = payload.appVersion;
    this.dbPath = payload.dbPath;
    this.profile = payload.profile;
    this.sessions = payload.sessions;
    this.activeSnapshot = payload.activeSnapshot;
    if (payload.activeSnapshot?.telemetry.referenceToday) {
      this.todayIso = payload.activeSnapshot.telemetry.referenceToday;
    }
  }

  async bootstrap(): Promise<BootstrapPayload | null> {
    this.isLoading = true;
    this.lastError = null;
    try {
      const [payload, hols] = await Promise.all([
        ipc.bootstrapApp(),
        ipc.getHolidaysRange(2023, 2033)
      ]);
      this.holidays = hols;
      this.applyBootstrapPayload(payload);
      await themeStore.init(payload.settings);
      return payload;
    } catch (e) {
      this.lastError = e instanceof Error ? e.message : String(e);
      themeStore.applyDomAttributes();
      return null;
    } finally {
      this.isLoading = false;
    }
  }

  async refreshActiveSnapshot(): Promise<void> {
    const sessionId = this.activeSession?.id;
    if (!sessionId) return;
    try {
      this.activeSnapshot = await ipc.getSessionSnapshot(sessionId);
    } catch (e) {
      this.lastError = e instanceof Error ? e.message : String(e);
    }
  }

  async upsertProfile(input: UpsertProfileInput): Promise<UserProfile> {
    const updated = await ipc.upsertUserProfile(input);
    this.profile = updated;
    return updated;
  }

  async setActiveSession(sessionId: string): Promise<BootstrapPayload> {
    const payload = await ipc.setActiveAcademicSession(sessionId);
    this.applyBootstrapPayload(payload);
    return payload;
  }

  async upsertSession(input: UpsertSessionInput): Promise<BootstrapPayload> {
    const payload = await ipc.upsertAcademicSession(input);
    this.applyBootstrapPayload(payload);
    return payload;
  }

  async deleteSession(sessionId: string): Promise<BootstrapPayload> {
    const payload = await ipc.deleteAcademicSession(sessionId);
    this.applyBootstrapPayload(payload);
    return payload;
  }

  async upsertCourse(input: UpsertCourseInput): Promise<SessionSnapshot> {
    const snap = await ipc.upsertCourse(input);
    this.activeSnapshot = snap;
    return snap;
  }

  async deleteCourse(courseId: string): Promise<SessionSnapshot | null> {
    const sessionId = this.activeSession?.id;
    if (!sessionId) return null;
    const snap = await ipc.deleteCourse(sessionId, courseId);
    this.activeSnapshot = snap;
    return snap;
  }

  async toggleStudyUnit(unitId: string, isCompleted: boolean): Promise<SessionSnapshot | null> {
    const sessionId = this.activeSession?.id;
    if (!sessionId) return null;
    const snap = await ipc.toggleStudyUnit(sessionId, unitId, isCompleted);
    this.activeSnapshot = snap;
    return snap;
  }

  async assignStudyUnitDate(
    unitId: string,
    assignedDate: string | null
  ): Promise<SessionSnapshot | null> {
    const sessionId = this.activeSession?.id;
    if (!sessionId) return null;
    const snap = await ipc.assignStudyUnitDate(sessionId, unitId, assignedDate);
    this.activeSnapshot = snap;
    return snap;
  }

  async upsertExam(input: UpsertExamInput): Promise<SessionSnapshot> {
    const snap = await ipc.upsertExam(input);
    this.activeSnapshot = snap;
    return snap;
  }

  async createItinerePair(input: CreateItinerePairInput): Promise<SessionSnapshot> {
    const snap = await ipc.createItinerePair(input);
    this.activeSnapshot = snap;
    return snap;
  }

  async deleteExam(examId: string, deletePair = false): Promise<SessionSnapshot | null> {
    const sessionId = this.activeSession?.id;
    if (!sessionId) return null;
    const snap = await ipc.deleteExam(sessionId, examId, deletePair);
    this.activeSnapshot = snap;
    return snap;
  }

  async upsertGenericEvent(input: UpsertGenericEventInput): Promise<SessionSnapshot> {
    const snap = await ipc.upsertGenericEvent(input);
    this.activeSnapshot = snap;
    return snap;
  }

  async deleteGenericEvent(eventId: string): Promise<SessionSnapshot | null> {
    const sessionId = this.activeSession?.id;
    if (!sessionId) return null;
    const snap = await ipc.deleteGenericEvent(sessionId, eventId);
    this.activeSnapshot = snap;
    return snap;
  }

  async upsertTodo(input: UpsertTodoInput): Promise<SessionSnapshot> {
    const snap = await ipc.upsertTodo(input);
    this.activeSnapshot = snap;
    return snap;
  }

  async toggleTodo(todoId: string, _done?: boolean): Promise<SessionSnapshot | null> {
    const sessionId = this.activeSession?.id;
    if (!sessionId) return null;
    const snap = await ipc.toggleTodo(sessionId, todoId);
    this.activeSnapshot = snap;
    return snap;
  }

  async deleteTodo(todoId: string): Promise<SessionSnapshot | null> {
    const sessionId = this.activeSession?.id;
    if (!sessionId) return null;
    const snap = await ipc.deleteTodo(sessionId, todoId);
    this.activeSnapshot = snap;
    return snap;
  }

  async saveDailyNote(date: string, contentMarkdown: string): Promise<SessionSnapshot | null> {
    const sessionId = this.activeSession?.id;
    if (!sessionId) return null;
    const snap = await ipc.saveDailyNote(sessionId, date, contentMarkdown);
    this.activeSnapshot = snap;
    return snap;
  }

  async executeAiCommand(prompt: string): Promise<AiAssistantResponse> {
    const sessionId = this.activeSession?.id;
    if (!sessionId) {
      throw new Error("Crea o seleziona prima una sessione accademica attiva.");
    }
    const response = await ipc.executeAiCommand(sessionId, prompt);
    if (response.updatedSnapshot) {
      this.activeSnapshot = response.updatedSnapshot;
    }
    if (response.updatedSettings) {
      await themeStore.init(response.updatedSettings);
    }
    return response;
  }

  async exportBackupJson(): Promise<string> {
    return ipc.exportBackupJson();
  }

  async importBackupJson(jsonPayload: string): Promise<BootstrapPayload> {
    const payload = await ipc.importBackupJson(jsonPayload);
    this.applyBootstrapPayload(payload);
    await themeStore.init(payload.settings);
    return payload;
  }
}

export const sessionStore = new SessionWorkspaceStore();
