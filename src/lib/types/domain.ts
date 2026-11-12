export type ThemeChoice = "light" | "dark" | "auto";
export type BackdropMaterial = "mica" | "acrylic" | "tabbed" | "solid";
export type WindowControlsStyle = "macos-left" | "win11-right";
export type WindowFrameStyle = "custom" | "standard";
export type SidebarState = "expanded" | "compact";
export type CalendarViewMode = "continuous" | "month" | "week" | "agenda";
export type SessionType = "invernale" | "estiva" | "autunnale" | "custom";
export type EventCategory = "studio" | "scadenza" | "lezione" | "personale";
export type AiProvider = "local" | "gemini" | "openai" | "ollama";

export interface UserProfile {
  id: number;
  studentName: string;
  university: string;
  faculty: string;
  degreeCourse: string;
  enrollmentYear: number | null;
  onboardingCompleted: boolean;
  createdAt: string;
  updatedAt: string;
}

export interface AppSettings {
  theme: ThemeChoice;
  backdropMaterial: BackdropMaterial;
  windowControlsStyle: WindowControlsStyle;
  windowFrameStyle: WindowFrameStyle;
  uiScale: number;
  sidebarState: SidebarState;
  defaultView: CalendarViewMode;
  startDayOfWeek: number;
  weeklyHolidayDay: number;
  studyHoursPerCfu: number;
  accentColor: string;
  holidayColor: string;
  sundayColor: string;
  catStudioColor: string;
  catScadenzaColor: string;
  catLezioneColor: string;
  catPersonaleColor: string;
  aiProvider: AiProvider;
  aiApiKey: string;
  aiEndpointUrl: string;
  aiModel: string;
  aiBarWidth: number;
  aiLocalModelId: string;
  aiIdleTimeoutMinutes: number;
  aiBootStrategy: "on_demand" | "on_app_start" | "contextual";
  aiHardwarePreference: "auto" | "gpu" | "cpu";
  activeSessionId: string | null;
  planFolder: string;
}

export interface AcademicSession {
  id: string;
  name: string;
  subtitle: string;
  sessionType: SessionType;
  academicYear: string;
  startDate: string;
  endDate: string;
  notes: string;
  createdAt: string;
  planId?: string | null;
  isSystem?: boolean;
}

export interface WeeklyLesson {
  id: string;
  sessionId: string;
  courseId: string | null;
  courseName: string;
  dayOfWeek: number; // 1 = Lunedi .. 7 = Domenica
  startTime: string; // HH:MM
  endTime: string;   // HH:MM
  classroom: string;
  professor: string;
  colorHex: string;
}

export interface Course {
  id: string;
  sessionId: string;
  sigla: string;
  name: string;
  code: string;
  cfu: number;
  professor: string;
  colorHex: string;
  difficultyWeight: number;
  targetHoursOverride: number | null;
  notes: string;
  sortOrder: number;
  planCourseId?: string | null;
  isVisible: boolean;
}

export interface StudyUnit {
  id: string;
  courseId: string;
  sessionId: string;
  unitNumber: number;
  title: string;
  description: string;
  estimatedHours: number;
  isCompleted: boolean;
  assignedDate: string | null;
  completedAt: string | null;
  planUnitId?: string | null;
}

export interface Exam {
  id: string;
  sessionId: string;
  courseId: string;
  date: string;
  timeSlot: string;
  durationMinutes: number;
  round: string;
  examType: string;
  classroom: string;
  weightFraction: number;
  pairId: string | null;
  title: string;
  details: string;
  status: "planned" | "passed" | "failed" | "withdrawn";
  grade: string | null;
}

export interface GenericEvent {
  id: string;
  sessionId: string;
  courseId: string | null;
  date: string;
  timeSlot: string;
  title: string;
  category: EventCategory;
  details: string;
}

export interface TodoSubItem {
  id: string;
  todoId: string;
  text: string;
  done: boolean;
  sortOrder: number;
  createdAt: string;
  planSubtaskId?: string | null;
}

export interface TodoItem {
  id: string;
  sessionId: string;
  courseId: string | null;
  date: string;
  text: string;
  done: boolean;
  priority: number;
  createdAt: string;
  planTaskId?: string | null;
  plannedHours?: number | null;
  kind?: string | null;
  subtasks?: TodoSubItem[];
}

export interface PlanState {
  planId: string;
  sessionId: string;
  appliedRevision: number;
  baseJson: string;
  pendingRevision: number | null;
  sourcePath: string;
  updatedAt: string;
}

export interface DailyNote {
  sessionId: string;
  date: string;
  contentMarkdown: string;
  updatedAt: string;
}

export interface HolidayInfo {
  date: string;
  name: string;
  isNational: boolean;
}

export interface ConflictItem {
  id: string;
  kind: "direct_date_overlap" | "direct_time_overlap" | "insufficient_cfu_buffer";
  severity: "critical" | "warning";
  date: string;
  secondaryDate: string | null;
  examIds: string[];
  courseSiglas: string[];
  daysGap: number;
  recommendedMinDays: number;
  title: string;
  description: string;
  suggestion: string;
}

export interface CourseWorkloadMetric {
  courseId: string;
  sigla: string;
  name: string;
  colorHex: string;
  cfu: number;
  totalUnits: number;
  completedUnits: number;
  completionPct: number;
  remainingStudyHours: number;
  nextExamDate: string | null;
  daysUntilNextExam: number | null;
  recommendedDailyHours: number;
  criticalityScore: number;
}

export interface HorizonPin {
  examId: string;
  courseId: string;
  sigla: string;
  colorHex: string;
  date: string;
  round: string;
  title: string;
  positionPct: number;
  hasConflict: boolean;
}

export interface HorizonTick {
  year: number;
  month: number;
  label: string;
  positionPct: number;
}

export interface TelemetrySummary {
  referenceToday: string;
  nextTargetExam: Exam | null;
  nextTargetCourse: Course | null;
  daysToNextTarget: number | null;
  coursesCount: number;
  examsCount: number;
  totalCfu: number;
  completedUnitsTotal: number;
  unitsTotal: number;
  overallProgressPct: number;
  mostCriticalCourseSigla: string | null;
  recommendedHoursToday: number;
  courseWorkloads: CourseWorkloadMetric[];
  dailyWorkloadMap: Record<string, number>;
  horizonStartDate: string;
  horizonEndDate: string;
  horizonTodayPct: number | null;
  horizonPins: HorizonPin[];
  horizonTicks: HorizonTick[];
}

export interface SessionSnapshot {
  session: AcademicSession;
  courses: Course[];
  studyUnits: StudyUnit[];
  exams: Exam[];
  genericEvents: GenericEvent[];
  todos: TodoItem[];
  dailyNotes: DailyNote[];
  weeklyLessons?: WeeklyLesson[];
  conflicts: ConflictItem[];
  ignoredConflicts?: ConflictItem[];
  telemetry: TelemetrySummary;
}

export interface BootstrapPayload {
  appVersion: string;
  dbPath: string;
  profile: UserProfile;
  settings: AppSettings;
  sessions: AcademicSession[];
  activeSnapshot: SessionSnapshot | null;
}

// ============================================================================
// Input DTOs
// ============================================================================

export interface UpsertProfileInput {
  studentName: string;
  university: string;
  faculty: string;
  degreeCourse: string;
  enrollmentYear?: number | null;
  onboardingCompleted: boolean;
}

export interface UpsertSessionInput {
  id?: string | null;
  name: string;
  subtitle: string;
  sessionType: SessionType;
  academicYear: string;
  startDate: string;
  endDate: string;
  notes?: string | null;
  planId?: string | null;
}

export interface StudyUnitDraft {
  id?: string | null;
  unitNumber: number;
  title: string;
  description?: string | null;
  estimatedHours?: number | null;
  isCompleted?: boolean | null;
  assignedDate?: string | null;
  planUnitId?: string | null;
}

export interface UpsertCourseInput {
  id?: string | null;
  sessionId: string;
  sigla: string;
  name: string;
  code: string;
  cfu: number;
  professor: string;
  colorHex: string;
  difficultyWeight?: number | null;
  targetHoursOverride?: number | null;
  notes?: string | null;
  units?: StudyUnitDraft[] | null;
  planCourseId?: string | null;
  isVisible?: boolean | null;
}

export interface UpsertExamInput {
  id?: string | null;
  sessionId: string;
  courseId: string;
  date: string;
  timeSlot?: string | null;
  durationMinutes?: number | null;
  round: string;
  examType?: string | null;
  classroom?: string | null;
  weightFraction?: number | null;
  pairId?: string | null;
  title?: string | null;
  details?: string | null;
  status?: "planned" | "passed" | "failed" | "withdrawn" | null;
  grade?: string | null;
}

export interface CreateItinerePairInput {
  sessionId: string;
  courseId: string;
  date1: string;
  date2: string;
  timeSlot1?: string | null;
  timeSlot2?: string | null;
  classroom?: string | null;
  details?: string | null;
}

export interface UpsertGenericEventInput {
  id?: string | null;
  sessionId: string;
  courseId?: string | null;
  date: string;
  timeSlot?: string | null;
  title: string;
  category: EventCategory;
  details?: string | null;
}

export interface UpsertTodoInput {
  id?: string | null;
  sessionId: string;
  courseId?: string | null;
  date: string;
  text: string;
  priority?: number | null;
  planTaskId?: string | null;
  plannedHours?: number | null;
  kind?: string | null;
}

export interface UpsertTodoSubItemInput {
  id?: string | null;
  todoId: string;
  text: string;
  done?: boolean | null;
  sortOrder?: number | null;
  planSubtaskId?: string | null;
}

export interface UpsertWeeklyLessonInput {
  id?: string | null;
  sessionId: string;
  courseId?: string | null;
  courseName: string;
  dayOfWeek: number;
  startTime: string;
  endTime: string;
  classroom?: string | null;
  professor?: string | null;
  colorHex?: string | null;
}

// ============================================================================
// Windowing & DWM Native State (Fase 3)
// ============================================================================

export interface SystemThemeState {
  systemIsDark: boolean;
  effectiveIsDark: boolean;
  activeMaterial: BackdropMaterial;
  activeThemeMode: ThemeChoice;
}

export interface MaxButtonBounds {
  x: number;
  y: number;
  width: number;
  height: number;
}

// ============================================================================
// Multimodal AI Assistant & Tool Calling (Fase 6)
// ============================================================================

export interface AiToolCallRecord {
  toolName: string;
  summary: string;
}

export interface AiAssistantResponse {
  replyMarkdown: string;
  executedTools: AiToolCallRecord[];
  navigateToDate: string | null;
  updatedSettings: AppSettings | null;
  updatedSnapshot: SessionSnapshot | null;
  providerUsed: string;
}

// ============================================================================
// Study Plan Import & Diff Engine (Fase 2 & 3)
// ============================================================================

export interface PendingPlanNotification {
  planId: string;
  planName: string;
  revision: number;
  filePath: string;
  isNewSession: boolean;
}

export interface TaskChangeItem {
  taskId: string;
  courseId: string;
  date: string;
  text: string;
  plannedHours: number;
  kind: string;
}

export interface TaskMoveItem {
  taskId: string;
  courseId: string;
  text: string;
  oldDate: string;
  newDate: string;
  plannedHours: number;
}

export interface TaskConflictItem {
  taskId: string;
  courseId: string;
  baseDate: string;
  myDate: string;
  theirDate: string;
  baseText: string;
  myText: string;
  theirText: string;
  isCompleted: boolean;
}

export interface ExamChangeItem {
  examId: string;
  courseId: string;
  title: string;
  oldDate: string | null;
  newDate: string;
  oldStatus: string | null;
  newStatus: string;
}

export interface CourseChangeItem {
  courseId: string;
  sigla: string;
  name: string;
  isAdded: boolean;
  isActiveChanged: boolean;
  active: boolean;
}

export interface ExtractedLessonItem {
  dayOfWeek: number; // 1=Lun .. 7=Dom
  startTime: string; // HH:MM
  endTime: string;   // HH:MM
  courseName: string;
  classroom: string;
  professor: string;
}

export interface PlanDiffSummary {
  isInitialImport: boolean;
  planId: string;
  oldRevision: number;
  newRevision: number;
  addedTasks: TaskChangeItem[];
  removedTasks: TaskChangeItem[];
  movedTasks: TaskMoveItem[];
  conflicts: TaskConflictItem[];
  examsChanged: ExamChangeItem[];
  coursesChanged: CourseChangeItem[];
}

export interface AiModelCatalogItem {
  id: string;
  name: string;
  tagline: string;
  tier: string;
  tierLevel: number;
  sizeGb: number;
  vramReqGb: number;
  recommendedRamGb: number;
  fileName: string;
  mmprojFileName: string;
  downloadUrl: string;
  mmprojDownloadUrl: string;
  contextWindow: number;
  description: string;
  recommendedFor: string;
  isDownloaded: boolean;
  isActive: boolean;
}

export interface SystemHardwareInfo {
  totalRamGb: number;
  availableRamGb: number;
  gpuName: string;
  dedicatedVramGb: number;
  hasDedicatedGpu: boolean;
  recommendedModelId: string;
  recommendedReason: string;
}

export interface AiRuntimeStatus {
  state: "stopped" | "starting" | "ready" | "error";
  activeModelId?: string | null;
  port?: number | null;
  idleTimeoutMinutes: number;
  secondsUntilIdleShutdown?: number | null;
  errorMessage?: string | null;
}

export interface AiModelDownloadProgress {
  modelId: string;
  fileName: string;
  downloadedBytes: number;
  totalBytes: number;
  progressPct: number;
  speedMbPerSec: number;
  isCompleted: boolean;
  error?: string | null;
}
