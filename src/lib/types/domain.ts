export type ThemeChoice = "light" | "dark" | "auto";
export type BackdropMaterial = "mica" | "acrylic" | "tabbed" | "solid";
export type SectionId = "home" | "calendar" | "today" | "courses" | "quiz" | "progress" | "ai";
export type TaskCategory = "teoria" | "esercizi" | "temi_esame" | "dimostrazioni" | "domande_orale" | "ripasso";
export type AiProvider = "local" | "cli_agy" | "cli_claude" | "cli_codex" | "cli_custom";
export type SessionType = "invernale" | "estiva" | "autunnale" | "custom";

export interface AppSettings {
  theme: ThemeChoice;
  backdropMaterial: BackdropMaterial;
  uiScale: number;
  accentColor: string;
  catTeoriaColor: string;
  catEserciziColor: string;
  catTemiEsameColor: string;
  catDimostrazioniColor: string;
  catDomandeOraleColor: string;
  sidebarSections: SectionId[];
  hiddenSections: SectionId[];
  planFolder: string;
  activeSessionId: string | null;
  studyHoursPerCfu: number;
  startDayOfWeek: number;
  aiProvider: AiProvider;
  aiLocalModelId: string;
  aiCustomCliCommand: string;
  aiIdleTimeoutMinutes: number;
}

export interface AcademicSession {
  id: string;
  planId?: string | null;
  name: string;
  subtitle: string;
  sessionType: SessionType;
  academicYear: string;
  startDate: string;
  endDate: string;
  maxHoursPerDay: number;
  studyWeekdays: number[];
  offDates: string[];
  reviewDaysBeforeExam: number;
  notes: string;
  createdAt: string;
}

export interface Course {
  id: string;
  sessionId: string;
  planCourseId?: string | null;
  sigla: string;
  name: string;
  code: string;
  cfu: number;
  professor: string;
  colorHex: string;
  difficultyWeight: number;
  targetHours: number | null;
  isActive: boolean;
  notes: string;
  sortOrder: number;
}

export interface StudyUnit {
  id: string;
  courseId: string;
  sessionId: string;
  planUnitId?: string | null;
  unitNumber: number;
  chapter: string;
  title: string;
  description: string;
  source: string;
  estimatedHours: number;
  priority: number;
  isCompleted: boolean;
  completedAt: string | null;
}

export interface Exam {
  id: string;
  sessionId: string;
  courseId: string;
  planExamId?: string | null;
  date: string;
  timeSlot: string;
  durationMinutes: number;
  round: string;
  examType: string;
  classroom: string;
  title: string;
  details: string;
  status: "planned" | "passed" | "failed" | "withdrawn";
  grade: string | null;
}

export interface OperationalBlock {
  id: string;
  sessionId: string;
  date: string;
  courseId: string | null;
  blockOrder: number;
  category: TaskCategory;
  title: string;
  chapter: string;
  topic: string;
  pages: string;
  source: string;
  durationHours: number;
  instructions: string;
  done: boolean;
  planBlockId?: string | null;
}

export interface DailyOperationalPlan {
  sessionId: string;
  date: string;
  dailyGoal: string;
  strategyNote: string;
  blocks: OperationalBlock[];
}

export interface TaskSubItem {
  id: string;
  taskId: string;
  text: string;
  category: string;
  pages: string;
  done: boolean;
  sortOrder: number;
  planSubtaskId?: string | null;
}

export interface TaskItem {
  id: string;
  sessionId: string;
  courseId: string | null;
  unitId: string | null;
  date: string;
  category: TaskCategory;
  title: string;
  chapter: string;
  topic: string;
  pages: string;
  source: string;
  plannedHours: number;
  done: boolean;
  priority: number;
  planTaskId?: string | null;
  createdAt: string;
  subtasks: TaskSubItem[];
}

export interface QuizQuestion {
  id: string;
  quizId: string;
  questionOrder: number;
  questionType: "multiple_choice" | "open_flashcard";
  category: TaskCategory;
  chapter: string;
  topic: string;
  prompt: string;
  options: string[];
  correctIndex: number | null;
  explanation: string;
  planQuestionId?: string | null;
}

export interface QuizAttempt {
  id: string;
  quizId: string;
  sessionId: string;
  courseId: string;
  scorePct: number;
  correctCount: number;
  totalCount: number;
  answersJson: string;
  wrongQuestionIds: string[];
  completedAt: string;
}

export interface DailyQuiz {
  id: string;
  sessionId: string;
  courseId: string;
  date: string;
  title: string;
  topicSummary: string;
  planQuizId?: string | null;
  questions: QuizQuestion[];
  latestAttempt: QuizAttempt | null;
  attemptsCount: number;
}

export interface WeeklyLesson {
  id: string;
  sessionId: string;
  courseId: string | null;
  courseName: string;
  dayOfWeek: number;
  startTime: string;
  endTime: string;
  classroom: string;
  professor: string;
  colorHex: string;
}

export interface CalendarEvent {
  id: string;
  sessionId: string;
  courseId: string | null;
  date: string;
  timeSlot: string;
  title: string;
  category: string;
  details: string;
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
  kind: string;
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

export interface CategoryProgress {
  category: TaskCategory;
  totalTasks: number;
  completedTasks: number;
  plannedHours: number;
  completedHours: number;
  completionPct: number;
}

export interface CourseProgressMetric {
  courseId: string;
  sigla: string;
  name: string;
  colorHex: string;
  cfu: number;
  professor: string;
  nextExamDate: string | null;
  daysUntilExam: number | null;
  totalTasks: number;
  completedTasks: number;
  totalUnits: number;
  completedUnits: number;
  completionPct: number;
  plannedHoursTotal: number;
  completedHoursTotal: number;
  quizzesTotal: number;
  quizzesCompleted: number;
  quizAverageScore: number | null;
  recommendedDailyHours: number;
  criticalityScore: number;
  categoryBreakdown: CategoryProgress[];
}

export interface TelemetrySummary {
  referenceToday: string;
  nextTargetExam: Exam | null;
  nextTargetCourse: Course | null;
  daysToNextTarget: number | null;
  coursesCount: number;
  examsCount: number;
  totalCfu: number;
  tasksTotal: number;
  tasksCompleted: number;
  quizzesTotal: number;
  quizzesCompleted: number;
  quizOverallAvgScore: number | null;
  plannedHoursTotal: number;
  completedHoursTotal: number;
  overallProgressPct: number;
  hoursLast7Days: [string, number][];
  categoryTotals: CategoryProgress[];
  courseProgress: CourseProgressMetric[];
  dailyWorkloadMap: Record<string, number>;
}

export interface SessionSnapshot {
  session: AcademicSession;
  courses: Course[];
  studyUnits: StudyUnit[];
  exams: Exam[];
  operationalPlans: DailyOperationalPlan[];
  tasks: TaskItem[];
  quizzes: DailyQuiz[];
  quizAttempts: QuizAttempt[];
  weeklyLessons: WeeklyLesson[];
  calendarEvents: CalendarEvent[];
  dailyNotes: DailyNote[];
  conflicts: ConflictItem[];
  telemetry: TelemetrySummary;
}

export interface PendingPlanNotification {
  planId: string;
  planName: string;
  revision: number;
  filePath: string;
  isNewSession: boolean;
}

export interface BootstrapPayload {
  appVersion: string;
  dbPath: string;
  settings: AppSettings;
  sessions: AcademicSession[];
  activeSnapshot: SessionSnapshot | null;
  pendingPlans: PendingPlanNotification[];
}

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

export interface ExtractedLessonItem {
  dayOfWeek: number;
  startTime: string;
  endTime: string;
  courseName: string;
  classroom: string;
  professor: string;
}

export interface CliAgentInfo {
  id: string;
  name: string;
  command: string;
  installed: boolean;
  resolvedPath: string | null;
}

export interface AiToolCallRecord {
  toolName: string;
  summary: string;
}

export interface AiAssistantResponse {
  replyMarkdown: string;
  executedTools: AiToolCallRecord[];
  undoSnapshotId: string | null;
  navigateToDate: string | null;
  navigateToSection: SectionId | null;
  updatedSettings: AppSettings | null;
  updatedSnapshot: SessionSnapshot | null;
  providerUsed: string;
}

export interface PlanDiffSummary {
  isInitialImport: boolean;
  planId: string;
  oldRevision: number;
  newRevision: number;
  addedTasks: Array<{ taskId: string; courseId: string; date: string; title: string; plannedHours: number; category: string }>;
  removedTasks: Array<{ taskId: string; courseId: string; date: string; title: string; plannedHours: number; category: string }>;
  movedTasks: Array<{ taskId: string; courseId: string; title: string; oldDate: string; newDate: string; plannedHours: number }>;
  conflicts: Array<{ taskId: string; courseId: string; baseDate: string; myDate: string; theirDate: string; baseTitle: string; myTitle: string; theirTitle: string; isCompleted: boolean }>;
  quizzesCount: number;
  examsChanged: Array<{ examId: string; courseId: string; title: string; oldDate: string | null; newDate: string; oldStatus: string | null; newStatus: string }>;
  coursesChanged: Array<{ courseId: string; sigla: string; name: string; isAdded: boolean; isActiveChanged: boolean; active: boolean }>;
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

export interface UpsertSessionInput {
  id?: string | null;
  name: string;
  subtitle: string;
  sessionType: SessionType;
  academicYear: string;
  startDate: string;
  endDate: string;
  maxHoursPerDay?: number | null;
  studyWeekdays?: number[] | null;
  offDates?: string[] | null;
  reviewDaysBeforeExam?: number | null;
  notes?: string | null;
}

export interface UpsertCourseInput {
  id?: string | null;
  sessionId: string;
  sigla: string;
  name: string;
  code?: string | null;
  cfu: number;
  professor?: string | null;
  colorHex: string;
  difficultyWeight?: number | null;
  targetHours?: number | null;
  isActive?: boolean | null;
  notes?: string | null;
}

export interface UpsertStudyUnitInput {
  id?: string | null;
  sessionId: string;
  courseId: string;
  unitNumber: number;
  chapter?: string | null;
  title: string;
  description?: string | null;
  source?: string | null;
  estimatedHours: number;
  priority?: number | null;
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
  title?: string | null;
  details?: string | null;
  status?: "planned" | "passed" | "failed" | "withdrawn" | null;
  grade?: string | null;
}

export interface UpsertTaskInput {
  id?: string | null;
  sessionId: string;
  courseId?: string | null;
  unitId?: string | null;
  date: string;
  category: TaskCategory;
  title: string;
  chapter?: string | null;
  topic?: string | null;
  pages?: string | null;
  source?: string | null;
  plannedHours?: number | null;
  priority?: number | null;
}

export interface UpsertTaskSubItemInput {
  id?: string | null;
  taskId: string;
  text: string;
  category?: string | null;
  pages?: string | null;
  done?: boolean | null;
  sortOrder?: number | null;
}

export interface SubmitQuizAttemptInput {
  quizId: string;
  sessionId: string;
  courseId: string;
  scorePct: number;
  correctCount: number;
  totalCount: number;
  answersJson: string;
  wrongQuestionIds: string[];
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

export interface UpsertCalendarEventInput {
  id?: string | null;
  sessionId: string;
  courseId?: string | null;
  date: string;
  timeSlot?: string | null;
  title: string;
  category: string;
  details?: string | null;
}
