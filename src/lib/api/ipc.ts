import { invoke } from "@tauri-apps/api/core";
import type {
  AppSettings,
  BootstrapPayload,
  CreateItinerePairInput,
  HolidayInfo,
  SessionSnapshot,
  UpsertCourseInput,
  UpsertExamInput,
  UpsertGenericEventInput,
  UpsertProfileInput,
  UpsertSessionInput,
  UpsertTodoInput,
  UserProfile,
  BackdropMaterial,
  ThemeChoice,
  SystemThemeState,
  MaxButtonBounds,
  AiAssistantResponse
} from "../types/domain";

export const ipc = {
  healthCheck: () => invoke<string>("health_check"),

  bootstrapApp: () => invoke<BootstrapPayload>("bootstrap_app"),

  upsertUserProfile: (input: UpsertProfileInput) =>
    invoke<UserProfile>("upsert_user_profile", { input }),

  getAppSettings: () => invoke<AppSettings>("get_app_settings"),

  saveAppSettings: (settings: AppSettings) =>
    invoke<AppSettings>("save_app_settings", { settings }),

  applyWindowMaterial: (backdropMaterial: BackdropMaterial, theme: ThemeChoice) =>
    invoke<SystemThemeState>("apply_window_material", { backdropMaterial, theme }),

  getSystemThemeInfo: () => invoke<SystemThemeState>("get_system_theme_info"),

  setMaximizeButtonBounds: (bounds: MaxButtonBounds) =>
    invoke<void>("set_maximize_button_bounds", { bounds }),

  getSessionSnapshot: (sessionId: string) =>
    invoke<SessionSnapshot>("get_session_snapshot", { sessionId }),

  upsertAcademicSession: (input: UpsertSessionInput) =>
    invoke<BootstrapPayload>("upsert_academic_session", { input }),

  setActiveAcademicSession: (sessionId: string) =>
    invoke<BootstrapPayload>("set_active_academic_session", { sessionId }),

  deleteAcademicSession: (sessionId: string) =>
    invoke<BootstrapPayload>("delete_academic_session", { sessionId }),

  upsertCourse: (input: UpsertCourseInput) =>
    invoke<SessionSnapshot>("upsert_course", { input }),

  deleteCourse: (sessionId: string, courseId: string) =>
    invoke<SessionSnapshot>("delete_course", { sessionId, courseId }),

  toggleStudyUnit: (sessionId: string, unitId: string, isCompleted: boolean) =>
    invoke<SessionSnapshot>("toggle_study_unit", { sessionId, unitId, isCompleted }),

  assignStudyUnitDate: (sessionId: string, unitId: string, assignedDate: string | null) =>
    invoke<SessionSnapshot>("assign_study_unit_date", { sessionId, unitId, assignedDate }),

  upsertExam: (input: UpsertExamInput) =>
    invoke<SessionSnapshot>("upsert_exam", { input }),

  createItinerePair: (input: CreateItinerePairInput) =>
    invoke<SessionSnapshot>("create_itinere_pair", { input }),

  deleteExam: (sessionId: string, examId: string, deletePair = false) =>
    invoke<SessionSnapshot>("delete_exam", { sessionId, examId, deletePair }),

  upsertGenericEvent: (input: UpsertGenericEventInput) =>
    invoke<SessionSnapshot>("upsert_generic_event", { input }),

  deleteGenericEvent: (sessionId: string, eventId: string) =>
    invoke<SessionSnapshot>("delete_generic_event", { sessionId, eventId }),

  upsertTodo: (input: UpsertTodoInput) =>
    invoke<SessionSnapshot>("upsert_todo", { input }),

  toggleTodo: (sessionId: string, todoId: string) =>
    invoke<SessionSnapshot>("toggle_todo", { sessionId, todoId }),

  deleteTodo: (sessionId: string, todoId: string) =>
    invoke<SessionSnapshot>("delete_todo", { sessionId, todoId }),

  saveDailyNote: (sessionId: string, date: string, contentMarkdown: string) =>
    invoke<SessionSnapshot>("save_daily_note", { sessionId, date, contentMarkdown }),

  getHolidaysRange: (startYear: number, endYear: number) =>
    invoke<HolidayInfo[]>("get_holidays_range", { startYear, endYear }),

  exportBackupJson: () => invoke<string>("export_backup_json"),

  importBackupJson: (jsonPayload: string) =>
    invoke<BootstrapPayload>("import_backup_json", { jsonPayload }),

  executeAiCommand: (sessionId: string, prompt: string) =>
    invoke<AiAssistantResponse>("execute_ai_command", { sessionId, prompt })
};
