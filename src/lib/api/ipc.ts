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
  AiAssistantResponse,
  PendingPlanNotification,
  PlanDiffSummary,
  WeeklyLesson,
  UpsertWeeklyLessonInput,
  ExtractedLessonItem,
  AiModelCatalogItem,
  SystemHardwareInfo,
  AiRuntimeStatus
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

  ignoreConflict: (sessionId: string, conflictKey: string) =>
    invoke<SessionSnapshot>("ignore_conflict", { sessionId, conflictKey }),

  restoreConflict: (sessionId: string, conflictKey: string) =>
    invoke<SessionSnapshot>("restore_conflict", { sessionId, conflictKey }),

  ignoreAllConflicts: (sessionId: string, conflictKeys: string[]) =>
    invoke<SessionSnapshot>("ignore_all_conflicts", { sessionId, conflictKeys }),

  restoreAllConflicts: (sessionId: string) =>
    invoke<SessionSnapshot>("restore_all_conflicts", { sessionId }),

  getHolidaysRange: (startYear: number, endYear: number) =>
    invoke<HolidayInfo[]>("get_holidays_range", { startYear, endYear }),

  exportBackupJson: () => invoke<string>("export_backup_json"),

  importBackupJson: (jsonPayload: string) =>
    invoke<BootstrapPayload>("import_backup_json", { jsonPayload }),

  executeAiCommand: (sessionId: string, prompt: string) =>
    invoke<AiAssistantResponse>("execute_ai_command", { sessionId, prompt }),

  setPlanFolder: (folderPath: string) =>
    invoke<AppSettings>("set_plan_folder", { folderPath }),

  scanPendingPlans: () =>
    invoke<PendingPlanNotification[]>("scan_pending_plans"),

  previewPlanUpdate: (filePath: string) =>
    invoke<PlanDiffSummary>("preview_plan_update", { filePath }),

  applyPlanUpdate: (
    filePath: string,
    conflictResolutions: Record<string, string> = {},
    targetSessionId?: string | null
  ) =>
    invoke<SessionSnapshot>("apply_plan_update", {
      filePath,
      conflictResolutions,
      targetSessionId
    }),

  setCourseVisibility: (sessionId: string, courseId: string, isVisible: boolean) =>
    invoke<SessionSnapshot>("set_course_visibility", { sessionId, courseId, isVisible }),

  getWeeklyLessons: (sessionId: string) =>
    invoke<WeeklyLesson[]>("get_weekly_lessons", { sessionId }),

  upsertWeeklyLesson: (input: UpsertWeeklyLessonInput) =>
    invoke<WeeklyLesson>("upsert_weekly_lesson", { input }),

  deleteWeeklyLesson: (lessonId: string) =>
    invoke<boolean>("delete_weekly_lesson", { lessonId }),

  projectLessonsToCalendar: (sessionId: string, startDate: string, endDate: string) =>
    invoke<number>("project_lessons_to_calendar", { sessionId, startDate, endDate }),

  parseTimetableImage: (sessionId: string, imageBase64: string, mimeType: string) =>
    invoke<ExtractedLessonItem[]>("parse_timetable_image", { sessionId, imageBase64, mimeType }),

  parseTimetablePrompt: (sessionId: string, prompt: string) =>
    invoke<ExtractedLessonItem[]>("parse_timetable_prompt", { sessionId, prompt }),

  openFolderInExplorer: (path: string) =>
    invoke<void>("open_folder_in_explorer", { path }),

  getSystemHardwareInfo: () =>
    invoke<SystemHardwareInfo>("get_system_hardware_info"),

  getAiModelsStatus: () =>
    invoke<AiModelCatalogItem[]>("get_ai_models_status"),

  startAiModelDownload: (modelId: string) =>
    invoke<void>("start_ai_model_download", { modelId }),

  cancelAiModelDownload: (modelId: string) =>
    invoke<void>("cancel_ai_model_download", { modelId }),

  deleteAiModel: (modelId: string) =>
    invoke<void>("delete_ai_model", { modelId }),

  getAiRuntimeStatus: () =>
    invoke<AiRuntimeStatus>("get_ai_runtime_status"),

  stopAiRuntime: () =>
    invoke<void>("stop_ai_runtime"),

  startAiRuntime: (modelId?: string) =>
    invoke<void>("start_ai_runtime", { modelId })
};
