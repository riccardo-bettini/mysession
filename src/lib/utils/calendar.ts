import type {
  ConflictItem,
  Course,
  DailyNote,
  Exam,
  GenericEvent,
  HolidayInfo,
  SessionSnapshot,
  StudyUnit,
  TodoItem
} from "../types/domain";

export const MESI_ITALIANI: readonly string[] = [
  "Gennaio",
  "Febbraio",
  "Marzo",
  "Aprile",
  "Maggio",
  "Giugno",
  "Luglio",
  "Agosto",
  "Settembre",
  "Ottobre",
  "Novembre",
  "Dicembre"
] as const;

export const MESI_ITALIANI_SHORT: readonly string[] = [
  "Gen",
  "Feb",
  "Mar",
  "Apr",
  "Mag",
  "Giu",
  "Lug",
  "Ago",
  "Set",
  "Ott",
  "Nov",
  "Dic"
] as const;

// Indici standard JS Date.getDay(): 0 = Dom, 1 = Lun, ..., 6 = Sab
const WEEKDAY_SHORT_BY_JS: readonly string[] = ["Dom", "Lun", "Mar", "Mer", "Gio", "Ven", "Sab"];
const WEEKDAY_LONG_BY_JS: readonly string[] = [
  "Domenica",
  "Lunedì",
  "Martedì",
  "Mercoledì",
  "Giovedì",
  "Venerdì",
  "Sabato"
];

export interface WeekdayHeader {
  jsDay: number;
  shortLabel: string;
  longLabel: string;
  isWeeklyHoliday: boolean;
}

export interface CalendarDayCell {
  isoDate: string;
  year: number;
  month: number; // 1..12
  day: number; // 1..31
  jsWeekday: number; // 0..6
  isCurrentMonth: boolean;
  isToday: boolean;
  isWeeklyHoliday: boolean;
}

export interface VirtualMonthItem {
  index: number;
  key: string; // "YYYY-MM"
  year: number;
  month: number; // 1..12
  title: string; // es. "Gennaio 2027"
  shortTitle: string; // es. "Gen '27"
  rowCount: number; // 5 o 6 righe
}

export interface DayAggregatedData {
  isoDate: string;
  holiday: HolidayInfo | null;
  exams: Exam[];
  genericEvents: GenericEvent[];
  studyUnits: StudyUnit[];
  todos: TodoItem[];
  note: DailyNote | null;
  conflicts: ConflictItem[];
  workloadHours: number;
}

export function toIsoDate(year: number, month1Based: number, day: number): string {
  const y = String(year).padStart(4, "0");
  const m = String(month1Based).padStart(2, "0");
  const d = String(day).padStart(2, "0");
  return `${y}-${m}-${d}`;
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
  const wName = WEEKDAY_LONG_BY_JS[dt.getDay()] ?? "";
  const mName = MESI_ITALIANI[month - 1] ?? "";
  return `${wName} ${day} ${mName} ${year}`;
}

export function formatItalianDateShort(isoDate: string): string {
  const { month, day } = parseIsoDate(isoDate);
  const mShort = MESI_ITALIANI_SHORT[month - 1] ?? "";
  return `${day} ${mShort}`;
}

export function addDaysIso(isoDate: string, deltaDays: number): string {
  const { year, month, day } = parseIsoDate(isoDate);
  const dt = new Date(year, month - 1, day + deltaDays);
  return toIsoDate(dt.getFullYear(), dt.getMonth() + 1, dt.getDate());
}

export function addMonths(
  year: number,
  month1Based: number,
  deltaMonths: number
): { year: number; month: number } {
  const total = year * 12 + (month1Based - 1) + deltaMonths;
  const nextYear = Math.floor(total / 12);
  const nextMonth = (total % 12) + 1;
  return { year: nextYear, month: nextMonth };
}

export function diffDaysIso(fromIso: string, toIso: string): number {
  const a = parseIsoDate(fromIso);
  const b = parseIsoDate(toIso);
  const utcA = Date.UTC(a.year, a.month - 1, a.day);
  const utcB = Date.UTC(b.year, b.month - 1, b.day);
  return Math.round((utcB - utcA) / 86_400_000);
}

export function getWeekdayHeaders(
  startDayOfWeek = 1,
  weeklyHolidayDay = 0
): WeekdayHeader[] {
  const headers: WeekdayHeader[] = [];
  for (let i = 0; i < 7; i++) {
    const jsDay = (startDayOfWeek + i) % 7;
    headers.push({
      jsDay,
      shortLabel: WEEKDAY_SHORT_BY_JS[jsDay],
      longLabel: WEEKDAY_LONG_BY_JS[jsDay],
      isWeeklyHoliday: weeklyHolidayDay >= 0 && jsDay === weeklyHolidayDay
    });
  }
  return headers;
}

export function getMonthRowCount(
  year: number,
  month1Based: number,
  startDayOfWeek = 1
): number {
  const firstDayJs = new Date(year, month1Based - 1, 1).getDay();
  const leadingOffset = (firstDayJs - startDayOfWeek + 7) % 7;
  const daysInMonth = new Date(year, month1Based, 0).getDate();
  return Math.ceil((leadingOffset + daysInMonth) / 7);
}

export function buildMonthGrid(
  year: number,
  month1Based: number,
  startDayOfWeek = 1,
  weeklyHolidayDay = 0,
  todayIso = getTodayIso()
): CalendarDayCell[] {
  const firstOfMonth = new Date(year, month1Based - 1, 1);
  const firstDayJs = firstOfMonth.getDay();
  const leadingOffset = (firstDayJs - startDayOfWeek + 7) % 7;
  const daysInMonth = new Date(year, month1Based, 0).getDate();
  const rowCount = Math.ceil((leadingOffset + daysInMonth) / 7);
  const totalCells = rowCount * 7;

  const cells: CalendarDayCell[] = [];
  for (let i = 0; i < totalCells; i++) {
    const dayOffset = i - leadingOffset + 1;
    const cellDate = new Date(year, month1Based - 1, dayOffset);
    const cYear = cellDate.getFullYear();
    const cMonth = cellDate.getMonth() + 1;
    const cDay = cellDate.getDate();
    const jsWeekday = cellDate.getDay();
    const isoDate = toIsoDate(cYear, cMonth, cDay);

    cells.push({
      isoDate,
      year: cYear,
      month: cMonth,
      day: cDay,
      jsWeekday,
      isCurrentMonth: cYear === year && cMonth === month1Based,
      isToday: isoDate === todayIso,
      isWeeklyHoliday: weeklyHolidayDay >= 0 && jsWeekday === weeklyHolidayDay
    });
  }
  return cells;
}

export function buildWeekGrid(
  anchorIso: string,
  startDayOfWeek = 1,
  weeklyHolidayDay = 0,
  todayIso = getTodayIso()
): CalendarDayCell[] {
  const { year, month, day } = parseIsoDate(anchorIso);
  const anchorDate = new Date(year, month - 1, day);
  const anchorJsDay = anchorDate.getDay();
  const offsetToStart = (anchorJsDay - startDayOfWeek + 7) % 7;

  const cells: CalendarDayCell[] = [];
  for (let i = 0; i < 7; i++) {
    const cellDate = new Date(year, month - 1, day - offsetToStart + i);
    const cYear = cellDate.getFullYear();
    const cMonth = cellDate.getMonth() + 1;
    const cDay = cellDate.getDate();
    const jsWeekday = cellDate.getDay();
    const isoDate = toIsoDate(cYear, cMonth, cDay);

    cells.push({
      isoDate,
      year: cYear,
      month: cMonth,
      day: cDay,
      jsWeekday,
      isCurrentMonth: cMonth === month,
      isToday: isoDate === todayIso,
      isWeeklyHoliday: weeklyHolidayDay >= 0 && jsWeekday === weeklyHolidayDay
    });
  }
  return cells;
}

/**
 * Costruisce una finestra virtualizzata continua di mesi (es. 96 mesi = 8 anni)
 * centrata attorno all'anno/mese di riferimento, per lo scorrimento infinito a 120/180 Hz.
 */
export function buildVirtualMonthList(
  anchorYear: number,
  anchorMonth1Based: number,
  monthsBefore = 36,
  monthsAfter = 60,
  startDayOfWeek = 1
): VirtualMonthItem[] {
  const list: VirtualMonthItem[] = [];
  let idx = 0;
  for (let delta = -monthsBefore; delta <= monthsAfter; delta++) {
    const { year, month } = addMonths(anchorYear, anchorMonth1Based, delta);
    const key = `${year}-${String(month).padStart(2, "0")}`;
    const mName = MESI_ITALIANI[month - 1] ?? "";
    const mShort = MESI_ITALIANI_SHORT[month - 1] ?? "";
    const rowCount = getMonthRowCount(year, month, startDayOfWeek);

    list.push({
      index: idx++,
      key,
      year,
      month,
      title: `${mName} ${year}`,
      shortTitle: `${mShort} '${String(year).slice(-2)}`,
      rowCount
    });
  }
  return list;
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
        genericEvents: [],
        studyUnits: [],
        todos: [],
        note: null,
        conflicts: [],
        workloadHours: 0
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

  for (const ev of snapshot.genericEvents) {
    ensureDay(ev.date).genericEvents.push(ev);
  }

  for (const unit of snapshot.studyUnits) {
    if (unit.assignedDate) {
      ensureDay(unit.assignedDate).studyUnits.push(unit);
    }
  }

  for (const td of snapshot.todos) {
    ensureDay(td.date).todos.push(td);
  }

  for (const nt of snapshot.dailyNotes) {
    if (nt.contentMarkdown && nt.contentMarkdown.trim().length > 0) {
      ensureDay(nt.date).note = nt;
    }
  }

  for (const cf of snapshot.conflicts) {
    ensureDay(cf.date).conflicts.push(cf);
    if (cf.secondaryDate && cf.secondaryDate !== cf.date) {
      ensureDay(cf.secondaryDate).conflicts.push(cf);
    }
  }

  const workloadMap = snapshot.telemetry?.dailyWorkloadMap ?? {};
  for (const [iso, hours] of Object.entries(workloadMap)) {
    ensureDay(iso).workloadHours = hours;
  }

  return { byDate, courseById };
}

export function formatExamRoundLabel(round: string): string {
  switch (round) {
    case "1":
    case "1° Appello":
      return "1° Appello";
    case "2":
    case "2° Appello":
      return "2° Appello";
    case "3":
    case "3° Appello":
      return "3° Appello";
    case "itinere_1":
      return "1ª Prova in Itinere";
    case "itinere_2":
      return "2ª Prova in Itinere";
    default:
      return round || "Appello";
  }
}
