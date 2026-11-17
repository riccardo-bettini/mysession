import type {
  CalendarEvent,
  ConflictItem,
  Course,
  DailyNote,
  DailyOperationalPlan,
  DailyQuiz,
  Exam,
  HolidayInfo,
  SessionSnapshot,
  TaskItem
} from "../types/domain";

export const MESI_ITALIANI: readonly string[] = [
  "Gennaio", "Febbraio", "Marzo", "Aprile", "Maggio", "Giugno",
  "Luglio", "Agosto", "Settembre", "Ottobre", "Novembre", "Dicembre"
] as const;

export const MESI_ITALIANI_SHORT: readonly string[] = [
  "Gen", "Feb", "Mar", "Apr", "Mag", "Giu",
  "Lug", "Ago", "Set", "Ott", "Nov", "Dic"
] as const;

const WEEKDAY_SHORT_BY_JS: readonly string[] = ["Dom", "Lun", "Mar", "Mer", "Gio", "Ven", "Sab"];
const WEEKDAY_MINI_BY_JS: readonly string[] = ["D", "L", "M", "M", "G", "V", "S"];
const WEEKDAY_LONG_BY_JS: readonly string[] = [
  "Domenica", "Lunedì", "Martedì", "Mercoledì", "Giovedì", "Venerdì", "Sabato"
];

export interface WeekdayHeader {
  jsDay: number;
  shortLabel: string;
  miniLabel: string;
  longLabel: string;
}

export interface CalendarDayCell {
  isoDate: string;
  year: number;
  month: number;
  day: number;
  jsWeekday: number;
  isCurrentMonth: boolean;
  isToday: boolean;
}

export interface DayAggregatedData {
  isoDate: string;
  holiday: HolidayInfo | null;
  exams: Exam[];
  events: CalendarEvent[];
  tasks: TaskItem[];
  quizzes: DailyQuiz[];
  operationalPlan: DailyOperationalPlan | null;
  note: DailyNote | null;
  conflicts: ConflictItem[];
  plannedHours: number;
}

export function toIsoDate(year: number, month1Based: number, day: number): string {
  return `${String(year).padStart(4, "0")}-${String(month1Based).padStart(2, "0")}-${String(day).padStart(2, "0")}`;
}

export function getTodayIso(): string {
  const now = new Date();
  return toIsoDate(now.getFullYear(), now.getMonth() + 1, now.getDate());
}

export function parseIsoDate(isoDate: string): { year: number; month: number; day: number } {
  const parts = (isoDate || "").split("-").map((p) => parseInt(p, 10));
  if (parts.length === 3 && !parts.some(Number.isNaN)) {
    return { year: parts[0], month: parts[1], day: parts[2] };
  }
  const now = new Date();
  return { year: now.getFullYear(), month: now.getMonth() + 1, day: now.getDate() };
}

export function formatItalianDateLong(isoDate: string): string {
  const { year, month, day } = parseIsoDate(isoDate);
  const dt = new Date(year, month - 1, day);
  return `${WEEKDAY_LONG_BY_JS[dt.getDay()] ?? ""} ${day} ${MESI_ITALIANI[month - 1] ?? ""} ${year}`;
}

export function formatItalianDateShort(isoDate: string): string {
  const { month, day } = parseIsoDate(isoDate);
  return `${day} ${MESI_ITALIANI_SHORT[month - 1] ?? ""}`;
}

export function formatWeekdayShort(isoDate: string): string {
  const { year, month, day } = parseIsoDate(isoDate);
  const dt = new Date(year, month - 1, day);
  return WEEKDAY_SHORT_BY_JS[dt.getDay()] ?? "";
}

export function addDaysIso(isoDate: string, deltaDays: number): string {
  const { year, month, day } = parseIsoDate(isoDate);
  const dt = new Date(year, month - 1, day + deltaDays);
  return toIsoDate(dt.getFullYear(), dt.getMonth() + 1, dt.getDate());
}

export function addMonths(year: number, month1Based: number, deltaMonths: number): { year: number; month: number } {
  const total = year * 12 + (month1Based - 1) + deltaMonths;
  return { year: Math.floor(total / 12), month: (((total % 12) + 12) % 12) + 1 };
}

export function diffDaysIso(fromIso: string, toIso: string): number {
  const a = parseIsoDate(fromIso);
  const b = parseIsoDate(toIso);
  return Math.round((Date.UTC(b.year, b.month - 1, b.day) - Date.UTC(a.year, a.month - 1, a.day)) / 86_400_000);
}

export function getWeekdayHeaders(startDayOfWeek = 1): WeekdayHeader[] {
  const start = ((startDayOfWeek % 7) + 7) % 7;
  const headers: WeekdayHeader[] = [];
  for (let i = 0; i < 7; i++) {
    const jsDay = (start + i) % 7;
    headers.push({
      jsDay,
      shortLabel: WEEKDAY_SHORT_BY_JS[jsDay],
      miniLabel: WEEKDAY_MINI_BY_JS[jsDay],
      longLabel: WEEKDAY_LONG_BY_JS[jsDay]
    });
  }
  return headers;
}

export function buildMonthGrid(
  year: number,
  month1Based: number,
  startDayOfWeek = 1,
  todayIso = getTodayIso()
): CalendarDayCell[] {
  const start = ((startDayOfWeek % 7) + 7) % 7;
  const firstDayJs = new Date(year, month1Based - 1, 1).getDay();
  const leadingOffset = (firstDayJs - start + 7) % 7;
  const daysInMonth = new Date(year, month1Based, 0).getDate();
  const rowCount = Math.ceil((leadingOffset + daysInMonth) / 7);
  const totalCells = rowCount * 7;

  const cells: CalendarDayCell[] = [];
  for (let i = 0; i < totalCells; i++) {
    const cellDate = new Date(year, month1Based - 1, i - leadingOffset + 1);
    const cYear = cellDate.getFullYear();
    const cMonth = cellDate.getMonth() + 1;
    const cDay = cellDate.getDate();
    const isoDate = toIsoDate(cYear, cMonth, cDay);
    cells.push({
      isoDate,
      year: cYear,
      month: cMonth,
      day: cDay,
      jsWeekday: cellDate.getDay(),
      isCurrentMonth: cYear === year && cMonth === month1Based,
      isToday: isoDate === todayIso
    });
  }
  return cells;
}

export function buildWeekGrid(
  anchorIso: string,
  startDayOfWeek = 1,
  todayIso = getTodayIso()
): CalendarDayCell[] {
  const start = ((startDayOfWeek % 7) + 7) % 7;
  const { year, month, day } = parseIsoDate(anchorIso);
  const anchorJsDay = new Date(year, month - 1, day).getDay();
  const offsetToStart = (anchorJsDay - start + 7) % 7;

  const cells: CalendarDayCell[] = [];
  for (let i = 0; i < 7; i++) {
    const cellDate = new Date(year, month - 1, day - offsetToStart + i);
    const cYear = cellDate.getFullYear();
    const cMonth = cellDate.getMonth() + 1;
    const cDay = cellDate.getDate();
    const isoDate = toIsoDate(cYear, cMonth, cDay);
    cells.push({
      isoDate,
      year: cYear,
      month: cMonth,
      day: cDay,
      jsWeekday: cellDate.getDay(),
      isCurrentMonth: cMonth === month,
      isToday: isoDate === todayIso
    });
  }
  return cells;
}

export function indexSnapshotByDate(
  snapshot: SessionSnapshot | null,
  holidays: HolidayInfo[]
): {
  byDate: Map<string, DayAggregatedData>;
  courseById: Map<string, Course>;
} {
  const byDate = new Map<string, DayAggregatedData>();
  const courseById = new Map<string, Course>();

  const ensureDay = (isoDate: string): DayAggregatedData => {
    let entry = byDate.get(isoDate);
    if (!entry) {
      entry = {
        isoDate,
        holiday: null,
        exams: [],
        events: [],
        tasks: [],
        quizzes: [],
        operationalPlan: null,
        note: null,
        conflicts: [],
        plannedHours: 0
      };
      byDate.set(isoDate, entry);
    }
    return entry;
  };

  for (const h of holidays) {
    ensureDay(h.date).holiday = h;
  }

  if (!snapshot) {
    return { byDate, courseById };
  }

  for (const c of snapshot.courses) {
    courseById.set(c.id, c);
  }

  for (const ex of snapshot.exams) {
    ensureDay(ex.date).exams.push(ex);
  }

  for (const ev of snapshot.calendarEvents) {
    ensureDay(ev.date).events.push(ev);
  }

  for (const t of snapshot.tasks) {
    const day = ensureDay(t.date);
    day.tasks.push(t);
    day.plannedHours = Math.round((day.plannedHours + t.plannedHours) * 10) / 10;
  }

  for (const q of snapshot.quizzes) {
    ensureDay(q.date).quizzes.push(q);
  }

  for (const op of snapshot.operationalPlans) {
    ensureDay(op.date).operationalPlan = op;
  }

  for (const nt of snapshot.dailyNotes) {
    if (nt.contentMarkdown?.trim()) {
      ensureDay(nt.date).note = nt;
    }
  }

  for (const cf of snapshot.conflicts) {
    ensureDay(cf.date).conflicts.push(cf);
    if (cf.secondaryDate && cf.secondaryDate !== cf.date) {
      ensureDay(cf.secondaryDate).conflicts.push(cf);
    }
  }

  return { byDate, courseById };
}
