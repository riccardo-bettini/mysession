import { invoke } from "@tauri-apps/api/core";
import type {
  AcademicSession,
  AiAssistantResponse,
  AiModelCatalogItem,
  AiRuntimeStatus,
  AppSettings,
  BootstrapPayload,
  CliAgentInfo,
  Course,
  ExtractedLessonItem,
  HolidayInfo,
  MaxButtonBounds,
  PendingPlanNotification,
  PlanDiffSummary,
  QuizAttempt,
  SessionSnapshot,
  SubmitQuizAttemptInput,
  SystemHardwareInfo,
  SystemThemeState,
  TaskItem,
  UpsertCalendarEventInput,
  UpsertCourseInput,
  UpsertExamInput,
  UpsertSessionInput,
  UpsertStudyUnitInput,
  UpsertTaskInput,
  UpsertTaskSubItemInput,
  UpsertWeeklyLessonInput,
  WeeklyLesson
} from "../types/domain";

export const ipc = {
  healthCheck: () => invoke<string>("health_check"),
  bootstrapApp: () => invoke<BootstrapPayload>("bootstrap_app"),

  getAppSettings: () => invoke<AppSettings>("get_app_settings"),
  saveAppSettings: (settings: AppSettings) => invoke<AppSettings>("save_app_settings", { settings }),
  applyWindowMaterial: (material: string, theme: string) =>
    invoke<SystemThemeState>("apply_window_material", { material, theme }),
  getSystemThemeInfo: () => invoke<SystemThemeState>("get_system_theme_info"),
  setMaximizeButtonBounds: (bounds: MaxButtonBounds) =>
    invoke<void>("set_maximize_button_bounds", { bounds }),

  getSessionSnapshot: (sessionId: string) =>
    invoke<SessionSnapshot>("get_session_snapshot", { sessionId }),
  upsertAcademicSession: (input: UpsertSessionInput) =>
    invoke<AcademicSession>("upsert_academic_session", { input }),
  setActiveAcademicSession: (sessionId: string | null) =>
    invoke<SessionSnapshot>("set_active_academic_session", { sessionId }),
  deleteAcademicSession: (sessionId: string) =>
    invoke<BootstrapPayload>("delete_academic_session", { sessionId }),

  upsertCourse: (input: UpsertCourseInput) => invoke<Course>("upsert_course", { input }),
  deleteCourse: (courseId: string) => invoke<void>("delete_course", { courseId }),

  upsertStudyUnit: (input: UpsertStudyUnitInput) => invoke<void>("upsert_study_unit", { input }),
  toggleStudyUnit: (unitId: string, completed: boolean) =>
    invoke<void>("toggle_study_unit", { unitId, completed }),

  upsertExam: (input: UpsertExamInput) => invoke<void>("upsert_exam", { input }),
  deleteExam: (examId: string) => invoke<void>("delete_exam", { examId }),

  toggleOperationalBlock: (blockId: string, done: boolean) =>
    invoke<void>("toggle_operational_block", { blockId, done }),

  upsertTask: (input: UpsertTaskInput) => invoke<TaskItem>("upsert_task", { input }),
  toggleTask: (taskId: string, done: boolean) => invoke<void>("toggle_task", { taskId, done }),
  deleteTask: (taskId: string) => invoke<void>("delete_task", { taskId }),

  upsertTaskSubitem: (input: UpsertTaskSubItemInput) => invoke<void>("upsert_task_subitem", { input }),
  toggleTaskSubitem: (subitemId: string, done: boolean) =>
    invoke<void>("toggle_task_subitem", { subitemId, done }),
  deleteTaskSubitem: (subitemId: string) => invoke<void>("delete_task_subitem", { subitemId }),

  submitQuizAttempt: (input: SubmitQuizAttemptInput) =>
    invoke<QuizAttempt>("submit_quiz_attempt", { input }),

  upsertWeeklyLesson: (input: UpsertWeeklyLessonInput) =>
    invoke<WeeklyLesson>("upsert_weekly_lesson", { input }),
  deleteWeeklyLesson: (lessonId: string) => invoke<void>("delete_weekly_lesson", { lessonId }),
  projectLessonsToCalendar: (sessionId: string, startDate: string, endDate: string) =>
    invoke<number>("project_lessons_to_calendar", { sessionId, startDate, endDate }),

  upsertCalendarEvent: (input: UpsertCalendarEventInput) =>
    invoke<void>("upsert_calendar_event", { input }),
  deleteCalendarEvent: (eventId: string) => invoke<void>("delete_calendar_event", { eventId }),

  saveDailyNote: (sessionId: string, date: string, contentMarkdown: string) =>
    invoke<void>("save_daily_note", { sessionId, date, contentMarkdown }),
  getHolidaysRange: (startYear: number, endYear: number) =>
    invoke<HolidayInfo[]>("get_holidays_range", { startYear, endYear }),

  setPlanFolder: (folderPath: string) =>
    invoke<PendingPlanNotification[]>("set_plan_folder", { folderPath }),
  scanPendingPlans: () => invoke<PendingPlanNotification[]>("scan_pending_plans"),
  previewPlanUpdate: (filePath: string) =>
    invoke<PlanDiffSummary>("preview_plan_update", { filePath }),
  applyPlanUpdate: (
    filePath: string,
    conflictResolutions: Record<string, string>,
    targetSessionId: string | null = null
  ) =>
    invoke<SessionSnapshot>("apply_plan_update", {
      filePath,
      conflictResolutions,
      targetSessionId
    }),

  openFolderInExplorer: (path: string) => invoke<void>("open_folder_in_explorer", { path }),
  relaunchApp: () => invoke<void>("relaunch_app"),

  // AI & CLI Agents
  getSystemHardwareInfo: () => invoke<SystemHardwareInfo>("get_system_hardware_info"),
  getCliAgentsStatus: () => invoke<CliAgentInfo[]>("get_cli_agents_status"),
  getAiModelsStatus: () => invoke<AiModelCatalogItem[]>("get_ai_models_status"),
  startAiModelDownload: (modelId: string) => invoke<void>("start_ai_model_download", { modelId }),
  cancelAiModelDownload: (modelId: string) => invoke<void>("cancel_ai_model_download", { modelId }),
  deleteAiModel: (modelId: string) => invoke<void>("delete_ai_model", { modelId }),
  getAiRuntimeStatus: () => invoke<AiRuntimeStatus>("get_ai_runtime_status"),
  stopAiRuntime: () => invoke<void>("stop_ai_runtime"),
  startAiRuntime: (modelId?: string) => invoke<void>("start_ai_runtime", { modelId }),

  executeAiCommand: (
    sessionId: string,
    prompt: string,
    imageBase64?: string | null,
    mimeType?: string | null
  ) =>
    invoke<AiAssistantResponse>("execute_ai_command", {
      sessionId,
      prompt,
      imageBase64: imageBase64 ?? null,
      mimeType: mimeType ?? null
    }),
  undoAiAction: (snapshotId: string) => invoke<SessionSnapshot>("undo_ai_action", { snapshotId }),
  parseTimetableImage: (imageBase64: string, mimeType: string) =>
    invoke<ExtractedLessonItem[]>("parse_timetable_image", { imageBase64, mimeType }),
  parseTimetablePrompt: (prompt: string) =>
    invoke<ExtractedLessonItem[]>("parse_timetable_prompt", { prompt })
};
