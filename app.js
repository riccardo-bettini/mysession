// ==========================================================================
// MySession — Desktop Study Session & Exam Management Application
// Infinite Calendar, Multi-Session Store, AI Assistant & Apple HIG Design
// ==========================================================================

const TODAY = new Date(2026, 9, 26); // 26 Ottobre 2026 (Default reference date)

const MONTH_NAMES = [
  "Gennaio", "Febbraio", "Marzo", "Aprile", "Maggio", "Giugno",
  "Luglio", "Agosto", "Settembre", "Ottobre", "Novembre", "Dicembre"
];

const WEEKDAY_NAMES = [
  "Lunedì", "Martedì", "Mercoledì", "Giovedì", "Venerdì", "Sabato", "Domenica"
];

const DEFAULT_COURSE_COLORS = [
  "var(--mam-border)",
  "var(--tm-border)",
  "var(--mse-border)",
  "var(--dcm-border)",
  "var(--psm-border)",
  "#0284c7",
  "#d97706",
  "#059669",
  "#7c3aed",
  "#dc2626"
];

// Initial Seed Store (Preserves the 5 PoliMi courses & 10 exam dates)
const INITIAL_STORE = {
  activeSessionId: "polimi_inv_2026_27",
  sessions: [
    {
      id: "polimi_inv_2026_27",
      name: "Sessione Invernale 2026/27",
      subtitle: "PoliMi · Ingegneria Meccanica",
      startDate: "2026-10-01",
      endDate: "2027-02-28",
      courses: {
        mam: {
          id: "mam",
          sigla: "MAM",
          name: "Meccanica Applicata alle Macchine",
          code: "054828",
          cfu: 10,
          prof: "Prof. Daniele Rocchi (Titolare), Alberto Zasso, Andrea Collina",
          colorVar: "var(--mam-border)",
          modules: [
            "Mod. 1 · Cinematica del corpo rigido e numeri complessi (manovellismo, quadrilatero, glifo)",
            "Mod. 2 · Dinamica dei sistemi a corpi rigidi, bilancio potenze ed Equazioni di Lagrange",
            "Mod. 3 · Azioni mutue: attrito statico/radente, volvente, usura e fluidodinamica",
            "Mod. 4 · Dinamica della macchina: regime assoluto, periodico (volano) e alternativo",
            "Mod. 5 · Organi delle macchine: ingranaggi, epicicloidali (Willis), cinghie, camme, cuscinetti, freni",
            "Mod. 6 · Elementi di meccanica delle vibrazioni (1 g.d.l. libere, forzate, isolamento)",
            "Mod. 7 · Introduzione alla stabilità e regolazione di velocità delle macchine"
          ]
        },
        tm: {
          id: "tm",
          sigla: "TM",
          name: "Tecnologia Meccanica",
          code: "TM-ING",
          cfu: 9,
          prof: "Dipartimento di Meccanica — PoliMi",
          colorVar: "var(--tm-border)",
          modules: []
        },
        mse: {
          id: "mse",
          sigla: "MSE",
          name: "Macchine e Sistemi Energetici",
          code: "MSE-ING",
          cfu: 9,
          prof: "Dipartimento di Energia — PoliMi",
          colorVar: "var(--mse-border)",
          modules: []
        },
        dcm: {
          id: "dcm",
          sigla: "DCM",
          name: "Dinamica e Controllo delle Macchine",
          code: "DCM-ING",
          cfu: 9,
          prof: "Dipartimento di Meccanica — PoliMi",
          colorVar: "var(--dcm-border)",
          modules: []
        },
        psm: {
          id: "psm",
          sigla: "PSM",
          name: "Progettazione di Sistemi Meccanici",
          code: "PSM-ING",
          cfu: 9,
          prof: "Dipartimento di Meccanica — PoliMi",
          colorVar: "var(--psm-border)",
          modules: []
        }
      },
      exams: [
        {
          id: "mam-1",
          courseId: "mam",
          date: "2027-01-08",
          round: 1,
          title: "1° Appello — Meccanica Applicata alle Macchine",
          details: "Scritto (Sez. 1 numerica + Sez. 2 problema complesso) + Orale."
        },
        {
          id: "tm-1",
          courseId: "tm",
          date: "2027-01-13",
          round: 1,
          title: "1° Appello — Tecnologia Meccanica",
          details: "Primo appello invernale (+5 giorni da MAM)."
        },
        {
          id: "mse-1",
          courseId: "mse",
          date: "2027-01-18",
          round: 1,
          title: "1° Appello — Macchine e Sistemi Energetici",
          details: "Primo appello invernale (+5 giorni da TM)."
        },
        {
          id: "dcm-1",
          courseId: "dcm",
          date: "2027-01-20",
          round: 1,
          title: "1° Appello — Dinamica e Controllo delle Macchine",
          details: "Primo appello invernale (+2 giorni da MSE)."
        },
        {
          id: "psm-1",
          courseId: "psm",
          date: "2027-01-25",
          round: 1,
          title: "1° Appello — Progettazione di Sistemi Meccanici",
          details: "Primo appello invernale (+5 giorni da DCM). Chiude il 1° giro."
        },
        {
          id: "tm-2",
          courseId: "tm",
          date: "2027-02-03",
          round: 2,
          title: "2° Appello — Tecnologia Meccanica",
          details: "Secondo appello (slot di recupero)."
        },
        {
          id: "mam-2",
          courseId: "mam",
          date: "2027-02-08",
          round: 2,
          title: "2° Appello — Meccanica Applicata alle Macchine",
          details: "⚠️ Coincidenza data con il 2° appello di Dinamica e Controllo (DCM)."
        },
        {
          id: "dcm-2",
          courseId: "dcm",
          date: "2027-02-08",
          round: 2,
          title: "2° Appello — Dinamica e Controllo delle Macchine",
          details: "⚠️ Coincidenza data con il 2° appello di Meccanica Applicata (MAM)."
        },
        {
          id: "mse-2",
          courseId: "mse",
          date: "2027-02-10",
          round: 2,
          title: "2° Appello — Macchine e Sistemi Energetici",
          details: "Secondo appello invernale (+2 giorni da MAM/DCM)."
        },
        {
          id: "psm-2",
          courseId: "psm",
          date: "2027-02-15",
          round: 2,
          title: "2° Appello — Progettazione di Sistemi Meccanici",
          details: "Secondo appello invernale (+5 giorni da MSE). Chiusura sessione."
        }
      ],
      genericEvents: [],
      todos: {}
    }
  ]
};

// ==========================================================================
// 1. HOLIDAY ENGINE (Italian National Holidays + Easter Calculation)
// ==========================================================================
function calculateEaster(year) {
  const a = year % 19;
  const b = Math.floor(year / 100);
  const c = year % 100;
  const d = Math.floor(b / 4);
  const e = b % 4;
  const f = Math.floor((b + 8) / 25);
  const g = Math.floor((b - f + 1) / 3);
  const h = (19 * a + b - d - g + 15) % 30;
  const i = Math.floor(c / 4);
  const k = c % 4;
  const l = (32 + 2 * e + 2 * i - h - k) % 7;
  const m = Math.floor((a + 11 * h + 22 * l) / 451);
  const month = Math.floor((h + l - 7 * m + 114) / 31) - 1;
  const day = ((h + l - 7 * m + 114) % 31) + 1;
  return new Date(year, month, day);
}

function getItalianHoliday(dateObj) {
  const y = dateObj.getFullYear();
  const m = dateObj.getMonth() + 1;
  const d = dateObj.getDate();
  const md = `${String(m).padStart(2, "0")}-${String(d).padStart(2, "0")}`;

  const fixed = {
    "01-01": "Capodanno",
    "01-06": "Epifania",
    "04-25": "Liberazione",
    "05-01": "Festa del Lavoro",
    "06-02": "Festa della Repubblica",
    "08-15": "Ferragosto",
    "11-01": "Tutti i Santi",
    "12-08": "Immacolata",
    "12-25": "Natale",
    "12-26": "S. Stefano"
  };

  if (fixed[md]) return fixed[md];

  // Movable Easter & Pasquetta
  const easter = calculateEaster(y);
  const easterMon = new Date(easter);
  easterMon.setDate(easter.getDate() + 1);

  if (dateObj.getMonth() === easter.getMonth() && dateObj.getDate() === easter.getDate()) {
    return "Pasqua";
  }
  if (dateObj.getMonth() === easterMon.getMonth() && dateObj.getDate() === easterMon.getDate()) {
    return "Pasquetta";
  }

  return null;
}

// ==========================================================================
// 2. SESSION STORE MANAGER (Enhanced with To-Do, In-Itinere, Generic Events)
// ==========================================================================
class SessionStore {
  constructor() {
    this.STORAGE_KEY = "mysession_app_store_v2";
    this.data = this.load();
  }

  load() {
    try {
      const raw = localStorage.getItem(this.STORAGE_KEY);
      if (raw) {
        const parsed = JSON.parse(raw);
        if (parsed.sessions && parsed.sessions.length > 0) {
          parsed.sessions.forEach(s => {
            s.genericEvents = s.genericEvents || [];
            s.todos = s.todos || {};
          });
          return parsed;
        }
      }
    } catch (e) {
      console.warn("Caricamento store fallito, ripristino default:", e);
    }
    this.save(INITIAL_STORE);
    return INITIAL_STORE;
  }

  save(data = this.data) {
    this.data = data;
    try {
      localStorage.setItem(this.STORAGE_KEY, JSON.stringify(data));
    } catch (e) {
      console.error("Salvataggio store fallito:", e);
    }
  }

  getActiveSession() {
    const s = this.data.sessions.find(x => x.id === this.data.activeSessionId);
    return s || this.data.sessions[0];
  }

  setActiveSession(id) {
    if (this.data.sessions.some(x => x.id === id)) {
      this.data.activeSessionId = id;
      this.save();
    }
  }

  createSession(newSession) {
    const id = "session_" + Date.now();
    const sessionObj = {
      id,
      name: newSession.name.trim() || "Nuova Sessione",
      subtitle: newSession.subtitle ? newSession.subtitle.trim() : "Sessione di Studio",
      startDate: newSession.startDate,
      endDate: newSession.endDate,
      courses: {},
      exams: [],
      genericEvents: [],
      todos: {}
    };
    this.data.sessions.push(sessionObj);
    this.data.activeSessionId = id;
    this.save();
    return sessionObj;
  }

  deleteSession(id) {
    if (this.data.sessions.length <= 1) {
      alert("Non puoi eliminare l'unica sessione presente.");
      return false;
    }
    this.data.sessions = this.data.sessions.filter(s => s.id !== id);
    if (this.data.activeSessionId === id) {
      this.data.activeSessionId = this.data.sessions[0].id;
    }
    this.save();
    return true;
  }

  addCourse(course) {
    const session = this.getActiveSession();
    const id = course.id || ("c_" + Date.now());
    const cObj = {
      id,
      sigla: (course.sigla || id.toUpperCase()).trim(),
      name: (course.name || course.sigla || "Materia").trim(),
      code: course.code ? course.code.trim() : id.toUpperCase(),
      cfu: parseInt(course.cfu, 10) || 6,
      prof: course.prof ? course.prof.trim() : "Docente",
      colorVar: course.colorVar || DEFAULT_COURSE_COLORS[Object.keys(session.courses).length % DEFAULT_COURSE_COLORS.length],
      modules: course.modules || []
    };
    session.courses[id] = cObj;
    this.save();
    return cObj;
  }

  addExam(exam) {
    const session = this.getActiveSession();
    const id = "exam_" + Date.now();
    const course = session.courses[exam.courseId];
    let title = exam.title;
    if (!title) {
      if (exam.round === "itinere_1") {
        title = `1ª Prova in Itinere — ${course?.sigla || "Esame"}`;
      } else if (exam.round === "itinere_2") {
        title = `2ª Prova in Itinere — ${course?.sigla || "Esame"}`;
      } else {
        title = `${exam.round}° Appello — ${course?.sigla || "Esame"}`;
      }
    }
    const examObj = {
      id,
      courseId: exam.courseId,
      date: exam.date,
      round: exam.round,
      title,
      details: exam.details || "Appello d'esame programmato",
      pairId: exam.pairId || null
    };
    session.exams.push(examObj);
    this.save();
    return examObj;
  }

  addItinereExams({ courseId, date1, date2, details }) {
    const session = this.getActiveSession();
    const pairId = "itinere_pair_" + Date.now();
    const course = session.courses[courseId];

    const ex1 = {
      id: "exam_itin1_" + Date.now(),
      courseId,
      date: date1,
      round: "itinere_1",
      title: `1ª Prova in Itinere — ${course ? course.sigla : "Esame"}`,
      details: details ? `1ª Prova parziale. ${details}` : "1ª Prova in itinere programmata",
      pairId
    };

    const ex2 = {
      id: "exam_itin2_" + (Date.now() + 1),
      courseId,
      date: date2,
      round: "itinere_2",
      title: `2ª Prova in Itinere — ${course ? course.sigla : "Esame"}`,
      details: details ? `2ª Prova parziale. ${details}` : "2ª Prova in itinere programmata",
      pairId
    };

    session.exams.push(ex1, ex2);
    this.save();
    return [ex1, ex2];
  }

  deleteExam(examId) {
    const session = this.getActiveSession();
    session.exams = session.exams.filter(e => e.id !== examId);
    this.save();
  }

  // Generic Events
  addGenericEvent(event) {
    const session = this.getActiveSession();
    session.genericEvents = session.genericEvents || [];
    const id = "genevt_" + Date.now();
    const newEvt = {
      id,
      date: event.date,
      title: event.title.trim(),
      time: event.time || "",
      category: event.category || "studio", // studio | scadenza | lezione | personale
      details: event.details || ""
    };
    session.genericEvents.push(newEvt);
    this.save();
    return newEvt;
  }

  deleteGenericEvent(eventId) {
    const session = this.getActiveSession();
    if (!session.genericEvents) return;
    session.genericEvents = session.genericEvents.filter(e => e.id !== eventId);
    this.save();
  }

  // To-Do List Management
  getTodos(dateStr) {
    const session = this.getActiveSession();
    session.todos = session.todos || {};
    return session.todos[dateStr] || [];
  }

  addTodo(dateStr, text) {
    const session = this.getActiveSession();
    session.todos = session.todos || {};
    session.todos[dateStr] = session.todos[dateStr] || [];
    const newTodo = {
      id: "todo_" + Date.now(),
      text: text.trim(),
      done: false,
      createdAt: Date.now()
    };
    session.todos[dateStr].push(newTodo);
    this.save();
    return newTodo;
  }

  toggleTodo(dateStr, todoId) {
    const session = this.getActiveSession();
    if (!session.todos || !session.todos[dateStr]) return;
    const item = session.todos[dateStr].find(t => t.id === todoId);
    if (item) {
      item.done = !item.done;
      this.save();
    }
  }

  deleteTodo(dateStr, todoId) {
    const session = this.getActiveSession();
    if (!session.todos || !session.todos[dateStr]) return;
    session.todos[dateStr] = session.todos[dateStr].filter(t => t.id !== todoId);
    this.save();
  }
}

const store = new SessionStore();

// ==========================================================================
// 3. APPLICATION STATE & MULTI-FILTER SETUP
// ==========================================================================
let state = {
  currentYear: 2027,
  currentMonth: 0, // Gennaio 2027
  currentView: "continuous", // "continuous" | "month" | "week" | "agenda"
  selectedCourseFilters: new Set(), // Multi-selection support!
  selectedRoundFilter: "all",
  activeDateString: "2027-01-08",
  weekStart: new Date(2027, 0, 4),
  themeChoice: localStorage.getItem("polimi_theme_choice") || "auto",
  sidebarState: (localStorage.getItem("polimi_sidebar_state") === "compact" || localStorage.getItem("polimi_sidebar_state") === "compact-rail") ? "compact" : "expanded",
  userPinnedSidebar: false,
  autoCompacted: false,
  infiniteRange: {
    startYear: 2026,
    startMonth: 8,
    endYear: 2027,
    endMonth: 4
  },
  aiPendingEvent: null, // Temporary holder for AI clarification flow
  speechAccumulatedText: "",
  settings: {
    theme: localStorage.getItem("polimi_theme_choice") || "auto",
    uiScale: parseInt(localStorage.getItem("polimi_ui_scale") || "100", 10),
    accentColor: localStorage.getItem("polimi_accent_color") || "#007AFF",
    holidayColor: localStorage.getItem("polimi_holiday_color") || "#FF3B30",
    sundayColor: localStorage.getItem("polimi_sunday_color") || "#FF9500",
    categoryColors: {
      studio: localStorage.getItem("polimi_cat_color_studio") || "#007AFF",
      scadenza: localStorage.getItem("polimi_cat_color_scadenza") || "#FF9500",
      lezione: localStorage.getItem("polimi_cat_color_lezione") || "#AF52DE",
      personale: localStorage.getItem("polimi_cat_color_personale") || "#34C759"
    },
    courseColors: {
      mam: localStorage.getItem("polimi_course_color_mam") || "#0284C7",
      tm: localStorage.getItem("polimi_course_color_tm") || "#D97706",
      mse: localStorage.getItem("polimi_course_color_mse") || "#059669",
      dcm: localStorage.getItem("polimi_course_color_dcm") || "#7C3AED",
      psm: localStorage.getItem("polimi_course_color_psm") || "#E11D48"
    },
    startDayOfWeek: parseInt(localStorage.getItem("polimi_start_day") || "1", 10), // 1 = Lunedì
    weeklyHolidayDay: parseInt(localStorage.getItem("polimi_weekly_holiday") || "0", 10), // 0 = Domenica
    windowFrameStyle: localStorage.getItem("polimi_window_frame_style") || "custom"
  }
};

let monthObserver = null;
let isInfiniteAppending = false;
let wheelLock = false;
let speechRecognizer = null;

function runWithoutMotion(fn) {
  document.documentElement.classList.add("no-motion");
  fn();
  requestAnimationFrame(() => {
    requestAnimationFrame(() => {
      document.documentElement.classList.remove("no-motion");
    });
  });
}

// ==========================================================================
// 4. INITIALIZATION
// ==========================================================================
document.addEventListener("DOMContentLoaded", () => {
  initThemeEngine();
  initSettingsEngine();
  handleWindowResize();
  applySidebarState();
  initSessionSelectorUI();
  initExamFilters();
  initHorizonRibbon();
  initModals();
  initEventListeners();
  initAiAssistant();
  updateAlertsUI();
  updateTelemetry();
  renderApp();

  requestAnimationFrame(() => {
    scrollToMonthSection(state.currentYear, state.currentMonth, false);
  });
});

// ==========================================================================
// 5. THEME & SETTINGS ENGINE
// ==========================================================================
function initThemeEngine() {
  const mediaQuery = window.matchMedia("(prefers-color-scheme: dark)");
  mediaQuery.addEventListener("change", () => {
    if (state.settings.theme === "auto") {
      applyEffectiveTheme();
    }
  });
  applyEffectiveTheme();
}

function setThemeChoice(choice) {
  state.settings.theme = choice;
  state.themeChoice = choice;
  localStorage.setItem("polimi_theme_choice", choice);
  applyEffectiveTheme();
}

function applyEffectiveTheme() {
  const isSystemDark = window.matchMedia("(prefers-color-scheme: dark)").matches;
  const effective = state.settings.theme === "auto"
    ? (isSystemDark ? "dark" : "light")
    : state.settings.theme;

  document.documentElement.setAttribute("data-theme", effective);

  document.querySelectorAll(".theme-btn, #settingsThemeSwitcher .seg-btn").forEach(btn => {
    const val = btn.dataset.themeChoice || btn.dataset.themeVal;
    const isSelected = val === state.settings.theme;
    btn.classList.toggle("active", isSelected);
    btn.setAttribute("aria-checked", isSelected ? "true" : "false");
  });

  if (state.settings && state.settings.categoryColors) {
    Object.keys(state.settings.categoryColors).forEach(cat => {
      applyCategoryColor(cat, state.settings.categoryColors[cat]);
    });
  }
  if (state.settings && state.settings.courseColors) {
    Object.keys(state.settings.courseColors).forEach(courseKey => {
      applyCourseColor(courseKey, state.settings.courseColors[courseKey]);
    });
  }
}

function cycleTheme() {
  const order = ["light", "dark", "auto"];
  const next = order[(order.indexOf(state.settings.theme) + 1) % order.length];
  setThemeChoice(next);
}

// Settings Engine (Scale, Accent, Holiday, Sunday, Categories, Courses, Calendar Days, Modal UI)
function initSettingsEngine() {
  applyUiScale(state.settings.uiScale);
  applyAccentColor(state.settings.accentColor);
  applyHolidayColor(state.settings.holidayColor);
  applySundayColor(state.settings.sundayColor);
  if (state.settings.categoryColors) {
    Object.keys(state.settings.categoryColors).forEach(cat => {
      applyCategoryColor(cat, state.settings.categoryColors[cat]);
    });
  }
  if (state.settings.courseColors) {
    Object.keys(state.settings.courseColors).forEach(courseKey => {
      applyCourseColor(courseKey, state.settings.courseColors[courseKey]);
    });
  }
  applyWindowFrameStyle(state.settings.windowFrameStyle);
  renderStickyWeekdayHeader();
  initSettingsModal();
  initCustomTitlebarEvents();
}

function applyWindowFrameStyle(style) {
  const cleanStyle = style === "standard" ? "standard" : "custom";
  state.settings.windowFrameStyle = cleanStyle;
  localStorage.setItem("polimi_window_frame_style", cleanStyle);

  const titlebar = document.getElementById("customAppTitlebar");

  if (cleanStyle === "custom") {
    document.body.classList.add("custom-window-frame");
    document.body.classList.remove("standard-window-frame");
    if (titlebar) titlebar.classList.remove("hidden");
  } else {
    document.body.classList.remove("custom-window-frame");
    document.body.classList.add("standard-window-frame");
    if (titlebar) titlebar.classList.add("hidden");
  }

  document.querySelectorAll("#settingsWindowFrameSwitcher .seg-btn").forEach(b => {
    b.classList.toggle("active", b.dataset.frameVal === cleanStyle);
  });

  // Notifica nativamente MySession.exe (C#) per nascondere o ripristinare la barra nativa di Windows
  fetch(`http://127.0.0.1:49271/api/frame?style=${cleanStyle}`).catch(() => {});
}

function updateMaxRestoreIcons(isMax) {
  const maxIcon = document.querySelector("#btnWinMaximize .icon-max");
  const restoreIcon = document.querySelector("#btnWinMaximize .icon-restore");
  if (maxIcon && restoreIcon) {
    maxIcon.classList.toggle("hidden", isMax);
    restoreIcon.classList.toggle("hidden", !isMax);
  }
  const maxBtn = document.getElementById("btnWinMaximize");
  if (maxBtn) maxBtn.title = isMax ? "Ripristina finestra" : "Ingrandisci finestra";
  document.body.classList.toggle("window-maximized", isMax);
}

function initCustomTitlebarEvents() {
  const minBtn = document.getElementById("btnWinMinimize");
  const maxBtn = document.getElementById("btnWinMaximize");
  const closeBtn = document.getElementById("btnWinClose");
  const dragRegion = document.getElementById("customTitlebarDragRegion");

  if (minBtn) {
    minBtn.addEventListener("click", () => {
      fetch("http://127.0.0.1:49271/api/minimize")
        .then(() => {
          showAiToast("Finestra ridotta a icona");
        })
        .catch(() => {
          showAiToast("Finestra ridotta a icona");
          window.blur();
        });
    });
  }

  const toggleMax = () => {
    fetch("http://127.0.0.1:49271/api/maximize")
      .then(res => res.json())
      .then(data => {
        if (typeof data.isMaximized === "boolean") {
          updateMaxRestoreIcons(data.isMaximized);
        }
      })
      .catch(() => {
        if (!document.fullscreenElement) {
          document.documentElement.requestFullscreen().catch(() => {});
        } else {
          document.exitFullscreen().catch(() => {});
        }
      });
  };

  if (maxBtn) maxBtn.addEventListener("click", toggleMax);
  if (dragRegion) dragRegion.addEventListener("dblclick", toggleMax);

  document.addEventListener("fullscreenchange", () => {
    updateMaxRestoreIcons(!!document.fullscreenElement);
  });

  window.addEventListener("resize", () => {
    const isMax = !!document.fullscreenElement || (window.outerWidth >= screen.availWidth - 12 && window.outerHeight >= screen.availHeight - 12);
    updateMaxRestoreIcons(isMax);
  });

  if (closeBtn) {
    closeBtn.addEventListener("click", () => {
      fetch("http://127.0.0.1:49271/api/close")
        .catch(() => {
          window.close();
          setTimeout(() => {
            if (!window.closed) {
              showAiToast("Per uscire premi Alt+F4 o chiudi la finestra");
            }
          }, 250);
        });
    });
  }
}

function applyUiScale(scalePercent) {
  const cleanScale = Math.max(70, Math.min(150, scalePercent));
  state.settings.uiScale = cleanScale;
  localStorage.setItem("polimi_ui_scale", cleanScale);

  // Set zoom on documentElement for WebView2/Chromium
  document.documentElement.style.zoom = (cleanScale / 100);

  // Aggiorna lo stato visivo del segmented control delle opzioni di scala predefinite
  document.querySelectorAll("#settingsUiScaleSwitcher .seg-btn").forEach(btn => {
    btn.classList.toggle("active", parseInt(btn.dataset.scaleVal, 10) === cleanScale);
  });

  // Re-evalua la geometria responsiva dei componenti per la nuova scala
  handleWindowResize();
}

function hexToRgba(hex, alpha) {
  let c = hex.replace("#", "");
  if (c.length === 3) c = c.split("").map(x => x + x).join("");
  const num = parseInt(c, 16);
  const r = (num >> 16) & 255;
  const g = (num >> 8) & 255;
  const b = num & 255;
  return `rgba(${r}, ${g}, ${b}, ${alpha})`;
}

function adjustHexBrightness(hex, percent) {
  let c = hex.replace("#", "");
  if (c.length === 3) c = c.split("").map(x => x + x).join("");
  let num = parseInt(c, 16);
  let r = Math.min(255, Math.max(0, ((num >> 16) & 255) + percent));
  let g = Math.min(255, Math.max(0, ((num >> 8) & 255) + percent));
  let b = Math.min(255, Math.max(0, (num & 255) + percent));
  return `#${((1 << 24) + (r << 16) + (g << 8) + b).toString(16).slice(1)}`;
}

function normalizeHexColor(str) {
  if (!str) return null;
  let s = String(str).trim();
  if (!s.startsWith("#")) s = "#" + s;
  if (/^#[0-9A-Fa-f]{6}$/.test(s)) return s.toUpperCase();
  if (/^#[0-9A-Fa-f]{3}$/.test(s)) {
    return ("#" + s[1] + s[1] + s[2] + s[2] + s[3] + s[3]).toUpperCase();
  }
  return null;
}

function hexToHsl(hex) {
  let c = (hex || "#007AFF").replace("#", "");
  if (c.length === 3) c = c.split("").map(x => x + x).join("");
  const num = parseInt(c, 16);
  if (isNaN(num)) return { h: 211, s: 100, l: 50 };
  const r = ((num >> 16) & 255) / 255;
  const g = ((num >> 8) & 255) / 255;
  const b = (num & 255) / 255;

  const max = Math.max(r, g, b), min = Math.min(r, g, b);
  let h = 0, s = 0, l = (max + min) / 2;

  if (max === min) {
    h = s = 0;
  } else {
    const d = max - min;
    s = l > 0.5 ? d / (2 - max - min) : d / (max + min);
    switch (max) {
      case r: h = (g - b) / d + (g < b ? 6 : 0); break;
      case g: h = (b - r) / d + 2; break;
      case b: h = (r - g) / d + 4; break;
    }
    h /= 6;
  }
  return {
    h: Math.round(h * 360),
    s: Math.round(s * 100),
    l: Math.round(l * 100)
  };
}

function hslToHex(h, s, l) {
  h = ((h % 360) + 360) % 360;
  s = Math.max(0, Math.min(100, s)) / 100;
  l = Math.max(0, Math.min(100, l)) / 100;
  const c = (1 - Math.abs(2 * l - 1)) * s;
  const x = c * (1 - Math.abs(((h / 60) % 2) - 1));
  const m = l - c / 2;
  let r = 0, g = 0, b = 0;
  if (0 <= h && h < 60) { r = c; g = x; b = 0; }
  else if (60 <= h && h < 120) { r = x; g = c; b = 0; }
  else if (120 <= h && h < 180) { r = 0; g = c; b = x; }
  else if (180 <= h && h < 240) { r = 0; g = x; b = c; }
  else if (240 <= h && h < 300) { r = x; g = 0; b = c; }
  else if (300 <= h && h <= 360) { r = c; g = 0; b = x; }
  const rHex = Math.round((r + m) * 255).toString(16).padStart(2, "0");
  const gHex = Math.round((g + m) * 255).toString(16).padStart(2, "0");
  const bHex = Math.round((b + m) * 255).toString(16).padStart(2, "0");
  return `#${rHex}${gHex}${bHex}`.toUpperCase();
}

function copyColorHex(hex, btnEl) {
  if (!hex) return;
  const cleanHex = normalizeHexColor(hex) || hex.toUpperCase();
  if (navigator.clipboard && navigator.clipboard.writeText) {
    navigator.clipboard.writeText(cleanHex).then(() => {
      showCopySuccess(btnEl, cleanHex);
    }).catch(() => {
      fallbackCopyText(cleanHex, btnEl);
    });
  } else {
    fallbackCopyText(cleanHex, btnEl);
  }
}

function fallbackCopyText(text, btnEl) {
  const ta = document.createElement("textarea");
  ta.value = text;
  ta.style.position = "fixed";
  ta.style.opacity = "0";
  document.body.appendChild(ta);
  ta.select();
  try {
    document.execCommand("copy");
    showCopySuccess(btnEl, text);
  } catch {
    showAiToast(`Colore: ${text}`);
  }
  document.body.removeChild(ta);
}

function showCopySuccess(btnEl, text) {
  if (btnEl) {
    const origHtml = btnEl.innerHTML;
    btnEl.classList.add("copied");
    btnEl.innerHTML = `<svg width="13" height="13" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true"><polyline points="20 6 9 17 4 12"/></svg><span>Copiato!</span>`;
    setTimeout(() => {
      btnEl.classList.remove("copied");
      btnEl.innerHTML = origHtml;
    }, 1500);
  }
  showAiToast(`✓ Codice colore ${text} copiato negli appunti`);
}

function applyAccentColor(hex) {
  if (!hex) return;
  const cleanHex = normalizeHexColor(hex) || hex;
  state.settings.accentColor = cleanHex;
  localStorage.setItem("polimi_accent_color", cleanHex);

  const root = document.documentElement;
  root.style.setProperty("--accent", cleanHex);
  root.style.setProperty("--accent-hover", adjustHexBrightness(cleanHex, -20));
  root.style.setProperty("--accent-soft", hexToRgba(cleanHex, 0.12));
  root.style.setProperty("--accent-border", hexToRgba(cleanHex, 0.32));
  root.style.setProperty("--border-focus", hexToRgba(cleanHex, 0.45));
}

function applyHolidayColor(hex) {
  if (!hex) return;
  const cleanHex = normalizeHexColor(hex) || hex;
  state.settings.holidayColor = cleanHex;
  localStorage.setItem("polimi_holiday_color", cleanHex);

  const root = document.documentElement;
  root.style.setProperty("--holiday-border", cleanHex);
  root.style.setProperty("--holiday-text", cleanHex);
  root.style.setProperty("--holiday-bg", hexToRgba(cleanHex, 0.12));
}

function applySundayColor(hex) {
  if (!hex) return;
  const cleanHex = normalizeHexColor(hex) || hex;
  state.settings.sundayColor = cleanHex;
  localStorage.setItem("polimi_sunday_color", cleanHex);

  const root = document.documentElement;
  root.style.setProperty("--sunday-text", cleanHex);
  root.style.setProperty("--sunday-wash", hexToRgba(cleanHex, 0.055));
}

function applyCategoryColor(catKey, hex) {
  if (!hex) return;
  const cleanHex = normalizeHexColor(hex) || hex;
  if (!state.settings.categoryColors) state.settings.categoryColors = {};
  state.settings.categoryColors[catKey] = cleanHex;
  localStorage.setItem("polimi_cat_color_" + catKey, cleanHex);

  const root = document.documentElement;
  const isDark = root.getAttribute("data-theme") === "dark";
  root.style.setProperty(`--cat-${catKey}-color`, cleanHex);
  root.style.setProperty(`--cat-${catKey}-bg`, hexToRgba(cleanHex, isDark ? 0.16 : 0.08));
}

function applyCourseColor(courseKey, hex) {
  if (!hex) return;
  const cleanHex = normalizeHexColor(hex) || hex;
  if (!state.settings.courseColors) state.settings.courseColors = {};
  state.settings.courseColors[courseKey] = cleanHex;
  localStorage.setItem("polimi_course_color_" + courseKey, cleanHex);

  const root = document.documentElement;
  const isDark = root.getAttribute("data-theme") === "dark";
  root.style.setProperty(`--${courseKey}-border`, cleanHex);
  root.style.setProperty(`--${courseKey}-bg`, hexToRgba(cleanHex, isDark ? 0.14 : 0.10));
  root.style.setProperty(`--${courseKey}-bg-hover`, hexToRgba(cleanHex, isDark ? 0.22 : 0.16));
  root.style.setProperty(`--${courseKey}-text`, isDark ? adjustHexBrightness(cleanHex, 60) : adjustHexBrightness(cleanHex, -40));
}

const APP_VERSION = "1.0.00";
const APP_AUTHOR = "Riccardo Bettini";

// Palette colori standard uniforme per TUTTE le categorie
const UNIFIED_STANDARD_PALETTE = [
  "#007AFF", // Blu Accento / Elettrico
  "#0284C7", // Azzurro / Sky
  "#059669", // Smeraldo / Teal
  "#34C759", // Verde Mela / Prato
  "#EAB308", // Oro / Giallo Caldo
  "#FF9500", // Arancione Vivo
  "#FF3B30", // Rosso Corallo / Scarlatto
  "#E11D48", // Carminio / Lampone
  "#AF52DE", // Viola / Orchidea
  "#7C3AED", // Indaco / Ametista
  "#5856D6", // Blu Indaco / Iris
  "#64748B"  // Grigio Ardesia Neutro
];

// Configurazione unificata per tutti i controlli colore delle categorie
const COLOR_CONTROLLER_CONFIGS = [
  // 1. SISTEMA & CALENDARIO
  {
    id: "accent",
    tab: "system",
    getter: () => state.settings.accentColor,
    setter: (hex) => applyAccentColor(hex),
    presets: UNIFIED_STANDARD_PALETTE
  },
  {
    id: "holiday",
    tab: "system",
    getter: () => state.settings.holidayColor,
    setter: (hex) => { applyHolidayColor(hex); renderApp(); },
    presets: UNIFIED_STANDARD_PALETTE
  },
  {
    id: "sunday",
    tab: "system",
    getter: () => state.settings.sundayColor,
    setter: (hex) => { applySundayColor(hex); renderApp(); },
    presets: UNIFIED_STANDARD_PALETTE
  },

  // 2. CATEGORIE IMPEGNI
  {
    id: "cat_studio",
    tab: "events",
    getter: () => (state.settings.categoryColors && state.settings.categoryColors.studio) || "#007AFF",
    setter: (hex) => { applyCategoryColor("studio", hex); renderApp(); },
    presets: UNIFIED_STANDARD_PALETTE
  },
  {
    id: "cat_scadenza",
    tab: "events",
    getter: () => (state.settings.categoryColors && state.settings.categoryColors.scadenza) || "#FF9500",
    setter: (hex) => { applyCategoryColor("scadenza", hex); renderApp(); },
    presets: UNIFIED_STANDARD_PALETTE
  },
  {
    id: "cat_lezione",
    tab: "events",
    getter: () => (state.settings.categoryColors && state.settings.categoryColors.lezione) || "#AF52DE",
    setter: (hex) => { applyCategoryColor("lezione", hex); renderApp(); },
    presets: UNIFIED_STANDARD_PALETTE
  },
  {
    id: "cat_personale",
    tab: "events",
    getter: () => (state.settings.categoryColors && state.settings.categoryColors.personale) || "#34C759",
    setter: (hex) => { applyCategoryColor("personale", hex); renderApp(); },
    presets: UNIFIED_STANDARD_PALETTE
  },

  // 3. CORSI & MATERIE
  {
    id: "course_mam",
    tab: "courses",
    getter: () => (state.settings.courseColors && state.settings.courseColors.mam) || "#0284C7",
    setter: (hex) => { applyCourseColor("mam", hex); renderApp(); },
    presets: UNIFIED_STANDARD_PALETTE
  },
  {
    id: "course_tm",
    tab: "courses",
    getter: () => (state.settings.courseColors && state.settings.courseColors.tm) || "#D97706",
    setter: (hex) => { applyCourseColor("tm", hex); renderApp(); },
    presets: UNIFIED_STANDARD_PALETTE
  },
  {
    id: "course_mse",
    tab: "courses",
    getter: () => (state.settings.courseColors && state.settings.courseColors.mse) || "#059669",
    setter: (hex) => { applyCourseColor("mse", hex); renderApp(); },
    presets: UNIFIED_STANDARD_PALETTE
  },
  {
    id: "course_dcm",
    tab: "courses",
    getter: () => (state.settings.courseColors && state.settings.courseColors.dcm) || "#7C3AED",
    setter: (hex) => { applyCourseColor("dcm", hex); renderApp(); },
    presets: UNIFIED_STANDARD_PALETTE
  },
  {
    id: "course_psm",
    tab: "courses",
    getter: () => (state.settings.courseColors && state.settings.courseColors.psm) || "#E11D48",
    setter: (hex) => { applyCourseColor("psm", hex); renderApp(); },
    presets: UNIFIED_STANDARD_PALETTE
  }
];

function syncColorControl(cfg) {
  const hex = cfg.getter();
  if (!hex) return;
  const cleanHex = normalizeHexColor(hex) || hex;
  const { h } = hexToHsl(cleanHex);

  const slider = document.getElementById(`slider_${cfg.id}`);
  const sliderVal = document.getElementById(`sliderVal_${cfg.id}`);
  const hexInput = document.getElementById(`hex_${cfg.id}`);
  const nativeColor = document.getElementById(`nativeColor_${cfg.id}`);
  const previewInd = document.getElementById(`previewInd_${cfg.id}`);
  const previewText = document.getElementById(`previewText_${cfg.id}`);
  const swatchesContainer = document.getElementById(`swatches_${cfg.id}`);

  if (slider) {
    slider.value = h;
    slider.style.setProperty("--thumb-color", cleanHex);
  }
  if (sliderVal) {
    sliderVal.textContent = `${h}°`;
  }
  if (hexInput && hexInput.value.toUpperCase() !== cleanHex.toUpperCase()) {
    hexInput.value = cleanHex.toUpperCase();
  }
  if (nativeColor && nativeColor.value.toLowerCase() !== cleanHex.toLowerCase()) {
    nativeColor.value = cleanHex;
  }
  if (previewInd) {
    previewInd.style.backgroundColor = cleanHex;
  }
  if (previewText) {
    previewText.textContent = cleanHex.toUpperCase();
  }
  if (swatchesContainer) {
    swatchesContainer.querySelectorAll(".color-swatch-chip").forEach(chip => {
      chip.classList.toggle("active", chip.dataset.color.toUpperCase() === cleanHex.toUpperCase());
    });
  }
}

function renderStickyWeekdayHeader() {
  const bar = document.getElementById("weekdaysBar");
  if (!bar) return;
  bar.innerHTML = "";

  const startDay = (state.settings && typeof state.settings.startDayOfWeek === "number") ? state.settings.startDayOfWeek : 1;
  const weeklyHoliday = (state.settings && typeof state.settings.weeklyHolidayDay === "number") ? state.settings.weeklyHolidayDay : 0;

  for (let i = 0; i < 7; i++) {
    const dayIndex = (startDay + i) % 7; // 0 = Dom, 1 = Lun, etc.
    const nameIdx = dayIndex === 0 ? 6 : dayIndex - 1; // WEEKDAY_NAMES index
    const col = document.createElement("div");
    col.className = "weekday-header-col";
    col.innerHTML = `<span class="weekday-full">${WEEKDAY_NAMES[nameIdx]}</span><span class="weekday-short">${WEEKDAY_NAMES[nameIdx].slice(0, 3)}</span>`;
    if (dayIndex === 6 || dayIndex === 0) col.classList.add("weekend");
    if (weeklyHoliday !== -1 && dayIndex === weeklyHoliday) {
      col.classList.add("sunday-header");
    }
    bar.appendChild(col);
  }
}

function initSettingsModal() {
  const overlay = document.getElementById("modalSettingsOverlay");
  const openBtn = document.getElementById("btnOpenSettings");
  const closeBtn = document.getElementById("btnCloseSettingsModal");
  const doneBtn = document.getElementById("btnSaveCloseSettings");
  const resetBtn = document.getElementById("btnResetSettingsDefault");
  if (!overlay) return;

  // Sidebar Navigation Switcher per le Sezioni delle Impostazioni
  const navItems = document.querySelectorAll(".settings-nav-item");
  const sectionPanels = document.querySelectorAll(".settings-section-panel");

  function switchSettingsSection(targetKey) {
    if (!targetKey) return;
    navItems.forEach(item => {
      const isMatch = item.dataset.sectionTarget === targetKey;
      item.classList.toggle("active", isMatch);
      item.setAttribute("aria-selected", isMatch ? "true" : "false");
    });
    sectionPanels.forEach(panel => {
      const isMatch = panel.id === `settingsSection-${targetKey}`;
      panel.classList.toggle("hidden", !isMatch);
      panel.classList.toggle("active", isMatch);
    });
  }

  navItems.forEach(item => {
    item.addEventListener("click", () => {
      switchSettingsSection(item.dataset.sectionTarget);
    });
  });

  const openSettings = () => {
    overlay.classList.remove("hidden");
    overlay.setAttribute("data-open", "true");
    syncSettingsModalInputs();
    const activeItem = document.querySelector(".settings-nav-item.active");
    if (activeItem) {
      switchSettingsSection(activeItem.dataset.sectionTarget);
    } else {
      switchSettingsSection("appearance");
    }
  };

  const closeSettings = () => {
    overlay.classList.add("hidden");
    overlay.setAttribute("data-open", "false");
  };

  if (openBtn) openBtn.addEventListener("click", openSettings);
  if (closeBtn) closeBtn.addEventListener("click", closeSettings);
  if (doneBtn) doneBtn.addEventListener("click", closeSettings);

  overlay.addEventListener("click", (e) => {
    if (e.target === overlay) closeSettings();
  });

  // Settings theme buttons
  document.querySelectorAll("#settingsThemeSwitcher .seg-btn").forEach(btn => {
    btn.addEventListener("click", () => {
      setThemeChoice(btn.dataset.themeVal);
    });
  });

  // Scale segmented preset options
  document.querySelectorAll("#settingsUiScaleSwitcher .seg-btn").forEach(btn => {
    btn.addEventListener("click", () => {
      applyUiScale(parseInt(btn.dataset.scaleVal, 10));
    });
  });

  // Inizializzazione interattiva di tutti i controller colore (Slider + Palette Standard + Hex + Native Picker)
  COLOR_CONTROLLER_CONFIGS.forEach(cfg => {
    const swatchesContainer = document.getElementById(`swatches_${cfg.id}`);
    const slider = document.getElementById(`slider_${cfg.id}`);
    const hexInput = document.getElementById(`hex_${cfg.id}`);
    const nativeColor = document.getElementById(`nativeColor_${cfg.id}`);
    const copyBtn = document.getElementById(`copyBtn_${cfg.id}`);

    // Generazione chip colori standard
    if (swatchesContainer) {
      swatchesContainer.innerHTML = "";
      cfg.presets.forEach(color => {
        const btn = document.createElement("button");
        btn.type = "button";
        btn.className = "color-swatch-chip";
        btn.dataset.color = color;
        btn.style.backgroundColor = color;
        btn.title = `Colore standard ${color}`;
        btn.addEventListener("click", () => {
          cfg.setter(color);
          syncColorControl(cfg);
        });
        swatchesContainer.appendChild(btn);
      });
    }

    // Selettore a slider (spettro cromatico con aggiornamento istantaneo a 60fps)
    if (slider) {
      slider.addEventListener("input", (e) => {
        const h = parseInt(e.target.value, 10);
        const currentHex = cfg.getter();
        const currentHsl = hexToHsl(currentHex);
        const s = Math.max(75, Math.min(100, currentHsl.s || 88));
        const l = Math.max(42, Math.min(56, currentHsl.l || 50));
        const newHex = hslToHex(h, s, l);
        cfg.setter(newHex);
        syncColorControl(cfg);
      });
    }

    // Casella codice esadecimale con validazione in tempo reale
    if (hexInput) {
      const handleHexChange = () => {
        const normalized = normalizeHexColor(hexInput.value);
        if (normalized) {
          cfg.setter(normalized);
          syncColorControl(cfg);
        } else {
          hexInput.value = (cfg.getter() || "").toUpperCase();
        }
      };
      hexInput.addEventListener("input", () => {
        const normalized = normalizeHexColor(hexInput.value);
        if (normalized) {
          cfg.setter(normalized);
          syncColorControl(cfg);
        }
      });
      hexInput.addEventListener("change", handleHexChange);
      hexInput.addEventListener("blur", handleHexChange);
    }

    // Selettore nativo / Pipetta contagocce
    if (nativeColor) {
      nativeColor.addEventListener("input", (e) => {
        cfg.setter(e.target.value);
        syncColorControl(cfg);
      });
      nativeColor.addEventListener("change", (e) => {
        cfg.setter(e.target.value);
        syncColorControl(cfg);
      });
    }

    // Pulsante Copia Codice Esadecimale
    if (copyBtn) {
      copyBtn.addEventListener("click", () => {
        copyColorHex(cfg.getter(), copyBtn);
      });
    }
  });

  // Giorno Inizio Settimana
  const startDaySelect = document.getElementById("settingsStartDaySelect");
  if (startDaySelect) {
    startDaySelect.addEventListener("change", (e) => {
      state.settings.startDayOfWeek = parseInt(e.target.value, 10);
      localStorage.setItem("polimi_start_day", state.settings.startDayOfWeek);
      renderStickyWeekdayHeader();
      renderApp();
    });
  }

  // Giorno Festivo Settimanale
  const weeklyHolidaySelect = document.getElementById("settingsWeeklyHolidaySelect");
  if (weeklyHolidaySelect) {
    weeklyHolidaySelect.addEventListener("change", (e) => {
      state.settings.weeklyHolidayDay = parseInt(e.target.value, 10);
      localStorage.setItem("polimi_weekly_holiday", state.settings.weeklyHolidayDay);
      renderStickyWeekdayHeader();
      renderApp();
    });
  }

  // Selettore Cornice Finestra Personalizzata / Standard
  const frameBtns = document.querySelectorAll("#settingsWindowFrameSwitcher .seg-btn");
  frameBtns.forEach(btn => {
    btn.addEventListener("click", () => {
      applyWindowFrameStyle(btn.dataset.frameVal);
    });
  });

  // Ripristina Predefiniti
  if (resetBtn) {
    resetBtn.addEventListener("click", () => {
      setThemeChoice("auto");
      applyUiScale(100);
      applyWindowFrameStyle("custom");
      state.settings.startDayOfWeek = 1;
      localStorage.setItem("polimi_start_day", "1");
      state.settings.weeklyHolidayDay = 0;
      localStorage.setItem("polimi_weekly_holiday", "0");

      applyAccentColor("#007AFF");
      applyHolidayColor("#FF3B30");
      applySundayColor("#FF9500");

      applyCategoryColor("studio", "#007AFF");
      applyCategoryColor("scadenza", "#FF9500");
      applyCategoryColor("lezione", "#AF52DE");
      applyCategoryColor("personale", "#34C759");

      applyCourseColor("mam", "#0284C7");
      applyCourseColor("tm", "#D97706");
      applyCourseColor("mse", "#059669");
      applyCourseColor("dcm", "#7C3AED");
      applyCourseColor("psm", "#E11D48");

      syncSettingsModalInputs();
      renderStickyWeekdayHeader();
      renderApp();
      showAiToast("✓ Impostazioni e colori ripristinati ai valori predefiniti");
    });
  }
}

function syncSettingsModalInputs() {
  const startDaySelect = document.getElementById("settingsStartDaySelect");
  const weeklyHolidaySelect = document.getElementById("settingsWeeklyHolidaySelect");

  document.querySelectorAll("#settingsUiScaleSwitcher .seg-btn").forEach(btn => {
    btn.classList.toggle("active", parseInt(btn.dataset.scaleVal, 10) === state.settings.uiScale);
  });
  if (startDaySelect) startDaySelect.value = state.settings.startDayOfWeek;
  if (weeklyHolidaySelect) weeklyHolidaySelect.value = state.settings.weeklyHolidayDay;

  document.querySelectorAll("#settingsWindowFrameSwitcher .seg-btn").forEach(b => {
    b.classList.toggle("active", b.dataset.frameVal === state.settings.windowFrameStyle);
  });

  applyEffectiveTheme();
  COLOR_CONTROLLER_CONFIGS.forEach(cfg => {
    syncColorControl(cfg);
  });
}

// ==========================================================================
// 6. SESSION SELECTOR & MANAGEMENT
// ==========================================================================
function initSessionSelectorUI() {
  const session = store.getActiveSession();
  const titleEl = document.getElementById("sidebarSessionTitle");
  const subEl = document.getElementById("sidebarSessionSubtitle");
  const titlebarSession = document.getElementById("customTitlebarSessionName");
  if (titleEl) titleEl.textContent = session.name;
  if (subEl) subEl.textContent = session.subtitle || "Sessione di Studio";
  if (titlebarSession) titlebarSession.textContent = session.name;

  const listEl = document.getElementById("sessionDropdownList");
  if (!listEl) return;
  listEl.innerHTML = "";

  store.data.sessions.forEach(s => {
    const btn = document.createElement("button");
    btn.type = "button";
    const isActive = s.id === session.id;
    btn.className = `session-item-row ${isActive ? "active" : ""}`;
    btn.innerHTML = `
      <div style="min-width:0;">
        <div style="font-weight:650; white-space:nowrap; overflow:hidden; text-overflow:ellipsis;">${s.name}</div>
        <div style="font-size:0.74rem; color:var(--text-secondary); white-space:nowrap; overflow:hidden; text-overflow:ellipsis;">${s.subtitle || ""}</div>
      </div>
      ${isActive ? `<span style="font-size:0.88rem; color:var(--accent); font-weight:700;">✓</span>` : ""}
    `;
    btn.addEventListener("click", () => {
      store.setActiveSession(s.id);
      closeSessionDropdown();
      onSessionChanged();
    });
    listEl.appendChild(btn);
  });
}

function toggleSessionDropdown() {
  const menu = document.getElementById("sessionDropdownMenu");
  const btn = document.getElementById("btnSessionMenu");
  if (!menu || !btn) return;
  const isOpening = menu.classList.contains("hidden");
  menu.classList.toggle("hidden", !isOpening);
  menu.setAttribute("data-open", isOpening ? "true" : "false");
  btn.setAttribute("aria-expanded", isOpening ? "true" : "false");
}

function closeSessionDropdown() {
  const menu = document.getElementById("sessionDropdownMenu");
  const btn = document.getElementById("btnSessionMenu");
  if (menu) {
    menu.classList.add("hidden");
    menu.setAttribute("data-open", "false");
  }
  if (btn) btn.setAttribute("aria-expanded", "false");
}

function onSessionChanged() {
  const session = store.getActiveSession();
  initSessionSelectorUI();
  initExamFilters();
  initHorizonRibbon();
  updateAlertsUI();
  updateTelemetry();
  state.selectedCourseFilters.clear();

  if (session.exams.length > 0) {
    const firstExDate = parseDateISO(session.exams[0].date);
    state.currentYear = firstExDate.getFullYear();
    state.currentMonth = firstExDate.getMonth();
    state.activeDateString = session.exams[0].date;
  } else if (session.startDate) {
    const sDate = parseDateISO(session.startDate);
    state.currentYear = sDate.getFullYear();
    state.currentMonth = sDate.getMonth();
    state.activeDateString = session.startDate;
  }

  state.infiniteRange.startYear = state.currentYear;
  state.infiniteRange.startMonth = Math.max(0, state.currentMonth - 2);
  state.infiniteRange.endYear = state.currentYear;
  state.infiniteRange.endMonth = Math.min(11, state.currentMonth + 4);

  renderApp();
  scrollToMonthSection(state.currentYear, state.currentMonth, false);
}

// ==========================================================================
// 7. DEDICATED ALERTS & CONFLICTS CONTROLLER
// ==========================================================================
function updateAlertsUI() {
  const session = store.getActiveSession();
  const exams = session.exams || [];
  const alertBtn = document.getElementById("btnToolbarAlert");
  const alertBadge = document.getElementById("alertCountBadge");
  const popoverList = document.getElementById("alertPopoverList");
  const popoverSub = document.getElementById("alertPopoverSubtitle");
  if (!alertBtn || !alertBadge) return;

  const dateCounts = {};
  exams.forEach(e => {
    dateCounts[e.date] = (dateCounts[e.date] || 0) + 1;
  });

  const conflictDates = Object.keys(dateCounts).filter(d => dateCounts[d] > 1);
  const totalConflicts = conflictDates.length;

  alertBadge.textContent = totalConflicts;
  alertBtn.classList.toggle("has-conflicts", totalConflicts > 0);
  alertBtn.title = totalConflicts > 0
    ? `${totalConflicts} coincidenz${totalConflicts === 1 ? "a" : "e"} di appelli rilevat${totalConflicts === 1 ? "a" : "e"}`
    : "Nessun conflitto d'esame rilevato";

  if (popoverSub) {
    popoverSub.textContent = totalConflicts === 0
      ? "Nessuna coincidenza"
      : `${totalConflicts} giornat${totalConflicts === 1 ? "a" : "e"} con più appelli`;
  }

  if (popoverList) {
    popoverList.innerHTML = "";
    if (totalConflicts === 0) {
      popoverList.innerHTML = `<div class="alert-clean-state">✓ Tutti gli appelli sono distribuiti regolarmente senza sovrapposizioni.</div>`;
    } else {
      conflictDates.forEach(dStr => {
        const confExams = exams.filter(e => e.date === dStr);
        const card = document.createElement("div");
        card.className = "alert-item-card";
        const dObj = parseDateISO(dStr);

        card.innerHTML = `
          <div style="display:flex; justify-content:space-between; align-items:center;">
            <strong>⚠️ ${formatDateISOToIT(dStr)} (${formatDateFriendly(dObj)})</strong>
            <span class="mono" style="font-size:0.7rem; font-weight:700;">${confExams.length} Esami</span>
          </div>
          <div>${confExams.map(e => `<b>${session.courses[e.courseId]?.sigla || "Esame"}</b> (${e.title})`).join("<br/>")}</div>
        `;

        card.addEventListener("click", () => {
          closeAlertPopover();
          navigateToDate(dObj, true);
        });

        popoverList.appendChild(card);
      });
    }
  }
}

function toggleAlertPopover() {
  const popover = document.getElementById("alertPopoverMenu");
  const btn = document.getElementById("btnToolbarAlert");
  if (!popover || !btn) return;
  const isHidden = popover.classList.contains("hidden");
  popover.classList.toggle("hidden", !isHidden);
  popover.setAttribute("data-open", isHidden ? "true" : "false");
  btn.setAttribute("aria-expanded", isHidden ? "true" : "false");
}

function closeAlertPopover() {
  const popover = document.getElementById("alertPopoverMenu");
  const btn = document.getElementById("btnToolbarAlert");
  if (popover) {
    popover.classList.add("hidden");
    popover.setAttribute("data-open", "false");
  }
  if (btn) btn.setAttribute("aria-expanded", "false");
}

// ==========================================================================
// 8. TELEMETRY WIDGET
// ==========================================================================
function updateTelemetry() {
  const session = store.getActiveSession();
  const exams = session.exams || [];
  const courses = Object.values(session.courses || {});

  const sortedExams = [...exams].sort((a, b) => a.date.localeCompare(b.date));
  const upcoming = sortedExams.find(e => parseDateISO(e.date) >= TODAY) || sortedExams[0];

  const countdownEl = document.getElementById("telemetryCountdown");
  const targetDescEl = document.getElementById("telemetryTargetDesc");
  const targetCodeEl = document.getElementById("telemetryTargetCode");
  const eyebrowEl = document.getElementById("telemetryEyebrow");

  const telemetryCardEl = document.querySelector(".telemetry-card");

  if (upcoming) {
    const uDate = parseDateISO(upcoming.date);
    const diffDays = Math.ceil((uDate - TODAY) / (1000 * 60 * 60 * 24));
    const c = session.courses[upcoming.courseId];
    if (countdownEl) countdownEl.textContent = `${diffDays} gg`;
    if (targetDescEl) {
      targetDescEl.innerHTML = `<strong>${c ? c.sigla : "Esame"} · ${upcoming.round === 'itinere_1' ? '1ª Itinere' : (upcoming.round === 'itinere_2' ? '2ª Itinere' : `${upcoming.round}° Appello`)}</strong><span class="telemetry-sub-date">${formatDateFriendly(uDate)}</span>`;
    }
    if (targetCodeEl) {
      targetCodeEl.textContent = c ? `${c.code} · ${c.cfu} CFU` : "Prossimo Appello";
    }
    if (eyebrowEl) eyebrowEl.textContent = "Prossimo Target";
    if (telemetryCardEl) {
      telemetryCardEl.title = `Prossimo Target: ${c ? c.sigla : "Esame"} (${upcoming.title}) - ${formatDateFriendly(uDate)} (tra ${diffDays} giorni) - Clicca per visualizzare`;
      telemetryCardEl.onclick = () => navigateToDate(uDate, true);
    }
  } else {
    if (countdownEl) countdownEl.textContent = "--";
    if (targetDescEl) targetDescEl.textContent = "Nessun esame configurato";
    if (targetCodeEl) targetCodeEl.textContent = "Sessione Vuota";
    if (telemetryCardEl) {
      telemetryCardEl.title = "Nessun esame configurato";
      telemetryCardEl.onclick = null;
    }
  }

  const coursesCountEl = document.getElementById("metricCoursesCount");
  const examsCountEl = document.getElementById("metricExamsCount");
  const totalCfuEl = document.getElementById("metricTotalCfu");

  if (coursesCountEl) coursesCountEl.textContent = courses.length;
  if (examsCountEl) examsCountEl.textContent = exams.length;

  const totalCfu = courses.reduce((acc, curr) => acc + (parseInt(curr.cfu, 10) || 0), 0);
  if (totalCfuEl) totalCfuEl.textContent = totalCfu;

  updateQuickMonthsUI();
}

function updateQuickMonthsUI() {
  const container = document.getElementById("quickMonthsList");
  if (!container) return;
  container.innerHTML = "";

  const session = store.getActiveSession();
  const start = session.startDate ? parseDateISO(session.startDate) : new Date(state.currentYear, 0, 1);
  const end = session.endDate ? parseDateISO(session.endDate) : new Date(state.currentYear, 11, 31);

  let cur = new Date(start.getFullYear(), start.getMonth(), 1);
  const stop = new Date(end.getFullYear(), end.getMonth(), 1);

  while (cur <= stop) {
    const y = cur.getFullYear();
    const m = cur.getMonth();
    const pill = document.createElement("button");
    pill.type = "button";
    pill.className = "month-pill";
    pill.dataset.year = y;
    pill.dataset.month = m;
    if (state.currentYear === y && state.currentMonth === m) {
      pill.classList.add("active");
    }

    const exCount = (session.exams || []).filter(e => {
      const d = parseDateISO(e.date);
      return d.getFullYear() === y && d.getMonth() === m;
    }).length;

    pill.innerHTML = `
      <span>${MONTH_NAMES[m]} ${y}</span>
      <span class="month-pill-badge mono">${exCount > 0 ? `${exCount} App.` : "0"}</span>
    `;

    pill.addEventListener("click", () => {
      navigateToMonth(y, m);
    });

    container.appendChild(pill);
    cur.setMonth(cur.getMonth() + 1);
  }
}

// Multi-Selection Course Filters
function initExamFilters() {
  const container = document.getElementById("examFiltersList");
  if (!container) return;
  container.innerHTML = "";

  const session = store.getActiveSession();
  const courses = Object.values(session.courses || {});

  if (courses.length === 0) {
    container.innerHTML = `<span class="empty-state-text" style="padding:6px 10px;">Nessuna materia registrata.</span>`;
    return;
  }

  courses.forEach(course => {
    const btn = document.createElement("button");
    btn.type = "button";
    const isSelected = state.selectedCourseFilters.has(course.id);
    btn.className = `filter-chip ${isSelected ? "active" : ""}`;
    btn.dataset.courseId = course.id;
    btn.style.borderLeftColor = course.colorVar || "var(--accent)";
    btn.title = `${course.sigla} · ${course.name} (${course.cfu} CFU) - Clicca per filtrare`;

    btn.innerHTML = `
      <div class="filter-chip-left">
        <span class="filter-chip-checkbox">${isSelected ? "✓" : ""}</span>
        <span class="course-swatch" style="background: ${course.colorVar}"></span>
        <span class="filter-chip-sigla mono">${course.sigla}</span>
        <span class="filter-chip-name">${course.name}</span>
      </div>
      <span class="filter-chip-count mono">${course.cfu} CFU</span>
    `;

    btn.addEventListener("click", () => {
      if (state.selectedCourseFilters.has(course.id)) {
        state.selectedCourseFilters.delete(course.id);
      } else {
        state.selectedCourseFilters.add(course.id);
      }
      updateFilterChipsUI();
      renderApp();
    });

    container.appendChild(btn);
  });
}

function updateFilterChipsUI() {
  document.querySelectorAll(".filter-chip").forEach(chip => {
    const cId = chip.dataset.courseId;
    const isSelected = state.selectedCourseFilters.has(cId);
    chip.classList.toggle("active", isSelected);
    const cb = chip.querySelector(".filter-chip-checkbox");
    if (cb) cb.textContent = isSelected ? "✓" : "";
  });
}

// ==========================================================================
// 9. DYNAMIC HORIZON TIMELINE RIBBON
// ==========================================================================
function initHorizonRibbon() {
  const track = document.getElementById("horizonTrack");
  const labelSessionName = document.getElementById("horizonLabelSessionName");
  const session = store.getActiveSession();

  if (labelSessionName) labelSessionName.textContent = session.name;
  if (!track) return;
  track.innerHTML = "";

  let startMs = session.startDate ? parseDateISO(session.startDate).getTime() : new Date(state.currentYear, 0, 1).getTime();
  let endMs = session.endDate ? parseDateISO(session.endDate).getTime() : new Date(state.currentYear, 11, 31).getTime();

  (session.exams || []).forEach(e => {
    const t = parseDateISO(e.date).getTime();
    if (t < startMs) startMs = t - (86400000 * 5);
    if (t > endMs) endMs = t + (86400000 * 5);
  });

  const totalSpan = Math.max(86400000 * 15, endMs - startMs);

  const pctForDate = (dObj) => {
    const clamped = Math.max(startMs, Math.min(endMs, dObj.getTime()));
    return ((clamped - startMs) / totalSpan) * 100;
  };

  const ticksWrap = document.createElement("div");
  ticksWrap.className = "horizon-month-ticks";

  let cursor = new Date(startMs);
  cursor.setDate(15);
  const endCursor = new Date(endMs);

  while (cursor <= endCursor) {
    const y = cursor.getFullYear();
    const m = cursor.getMonth();
    const tick = document.createElement("button");
    tick.type = "button";
    tick.className = "horizon-tick mono";
    tick.style.left = `${pctForDate(new Date(y, m, 15))}%`;
    tick.textContent = MONTH_NAMES[m].slice(0, 3).toUpperCase();
    tick.title = `Vai a ${MONTH_NAMES[m]} ${y}`;
    tick.addEventListener("click", () => navigateToMonth(y, m));
    ticksWrap.appendChild(tick);
    cursor.setMonth(cursor.getMonth() + 1);
  }
  track.appendChild(ticksWrap);

  if (TODAY.getTime() >= startMs && TODAY.getTime() <= endMs) {
    const todayMarker = document.createElement("div");
    todayMarker.className = "horizon-today-marker";
    todayMarker.style.left = `${pctForDate(TODAY)}%`;
    todayMarker.title = "Oggi: 26 Ottobre 2026";
    track.appendChild(todayMarker);
  }

  const dateCounts = {};
  (session.exams || []).forEach(e => {
    dateCounts[e.date] = (dateCounts[e.date] || 0) + 1;
  });

  (session.exams || []).forEach(evt => {
    const course = session.courses[evt.courseId];
    const dObj = parseDateISO(evt.date);
    const pin = document.createElement("button");
    pin.type = "button";
    const isConflict = dateCounts[evt.date] > 1;
    pin.className = `horizon-pin ${isConflict ? "conflict" : ""}`;
    pin.style.left = `${pctForDate(dObj)}%`;
    pin.style.background = course ? course.colorVar : "var(--accent)";
    pin.title = `${formatDateISOToIT(evt.date)} · ${course ? course.sigla : "Esame"} (${evt.round === 'itinere_1' ? '1ª Itinere' : (evt.round === 'itinere_2' ? '2ª Itinere' : `${evt.round}° Appello`)})`;
    pin.setAttribute("aria-label", pin.title);

    pin.addEventListener("click", () => {
      navigateToDate(dObj, true);
    });
    track.appendChild(pin);
  });
}

// ==========================================================================
// 10. EVENT LISTENERS & MODALS
// ==========================================================================
function initEventListeners() {
  const btnSessionMenu = document.getElementById("btnSessionMenu");
  if (btnSessionMenu) {
    btnSessionMenu.addEventListener("click", (e) => {
      e.stopPropagation();
      toggleSessionDropdown();
    });
  }

  // Alert Toolbar Button
  const btnAlert = document.getElementById("btnToolbarAlert");
  if (btnAlert) {
    btnAlert.addEventListener("click", (e) => {
      e.stopPropagation();
      toggleAlertPopover();
    });
  }

  document.addEventListener("click", (e) => {
    if (!e.target.closest("#sessionDropdownMenu") && !e.target.closest("#btnSessionMenu")) {
      closeSessionDropdown();
    }
    if (!e.target.closest("#alertPopoverMenu") && !e.target.closest("#btnToolbarAlert")) {
      closeAlertPopover();
    }
  });

  // Sidebar Toggle (Full -> Compact Rail -> Collapsed)
  document.getElementById("btnToggleSidebar").addEventListener("click", () => cycleSidebarState(false));

  // Stepper
  document.getElementById("btnPrevMonth").addEventListener("click", () => stepMonth(-1));
  document.getElementById("btnNextMonth").addEventListener("click", () => stepMonth(1));
  document.getElementById("btnToday").addEventListener("click", () => {
    navigateToDate(TODAY, false);
  });

  // Jump Month
  const btnJumpMonth = document.getElementById("btnJumpMonth");
  const hiddenMonthPicker = document.getElementById("hiddenMonthPicker");
  if (btnJumpMonth && hiddenMonthPicker) {
    btnJumpMonth.addEventListener("click", () => {
      hiddenMonthPicker.value = `${state.currentYear}-${String(state.currentMonth + 1).padStart(2, "0")}`;
      hiddenMonthPicker.showPicker ? hiddenMonthPicker.showPicker() : hiddenMonthPicker.click();
    });
    hiddenMonthPicker.addEventListener("change", (e) => {
      if (e.target.value) {
        const [y, m] = e.target.value.split("-").map(Number);
        navigateToMonth(y, m - 1);
      }
    });
  }

  // Reset Multi-Filters
  document.getElementById("btnResetFilters").addEventListener("click", () => {
    state.selectedCourseFilters.clear();
    state.selectedRoundFilter = "all";
    document.querySelectorAll(".pill-round").forEach(p => p.classList.remove("active"));
    document.querySelector('.pill-round[data-round="all"]').classList.add("active");
    updateFilterChipsUI();
    renderApp();
  });

  // Views
  ["continuous", "month", "week", "agenda"].forEach(viewName => {
    const btn = document.querySelector(`.seg-btn[data-view="${viewName}"]`);
    if (btn) {
      btn.addEventListener("click", () => switchView(viewName));
    }
  });

  // Rounds
  document.querySelectorAll(".pill-round").forEach(pill => {
    pill.addEventListener("click", () => {
      document.querySelectorAll(".pill-round").forEach(p => p.classList.remove("active"));
      pill.classList.add("active");
      state.selectedRoundFilter = pill.dataset.round;
      renderApp();
    });
  });

  // Theme
  document.querySelectorAll(".theme-btn").forEach(btn => {
    btn.addEventListener("click", () => setThemeChoice(btn.dataset.themeChoice));
  });

  // Week
  document.getElementById("btnPrevWeek").addEventListener("click", () => {
    state.weekStart.setDate(state.weekStart.getDate() - 7);
    renderWeekView();
  });
  document.getElementById("btnNextWeek").addEventListener("click", () => {
    state.weekStart.setDate(state.weekStart.getDate() + 7);
    renderWeekView();
  });

  // Drawer
  document.getElementById("btnCloseDrawer").addEventListener("click", closeDrawer);
  document.getElementById("drawerOverlay").addEventListener("click", closeDrawer);
  document.getElementById("btnSaveNotes").addEventListener("click", saveUserNotes);

  const btnDrawerAdd = document.getElementById("btnDrawerAddExamOnDate");
  if (btnDrawerAdd) {
    btnDrawerAdd.addEventListener("click", () => {
      openAddExamModal(state.activeDateString);
    });
  }

  const btnDrawerAddGeneric = document.getElementById("btnDrawerAddGenericEvent");
  if (btnDrawerAddGeneric) {
    btnDrawerAddGeneric.addEventListener("click", () => {
      openAddGenericEventModal(state.activeDateString);
    });
  }

  // To-Do in Drawer
  const btnAddTodo = document.getElementById("btnAddTodo");
  const inputNewTodo = document.getElementById("inputNewTodo");
  if (btnAddTodo && inputNewTodo) {
    const handleAddTodo = () => {
      const val = inputNewTodo.value.trim();
      if (!val || !state.activeDateString) return;
      store.addTodo(state.activeDateString, val);
      inputNewTodo.value = "";
      renderDrawerTodoList(state.activeDateString);
      renderApp();
    };
    btnAddTodo.addEventListener("click", handleAddTodo);
    inputNewTodo.addEventListener("keydown", (e) => {
      if (e.key === "Enter") {
        e.preventDefault();
        handleAddTodo();
      }
    });
  }

  // Mouse Wheel
  const viewport = document.getElementById("calendarViewContainer");
  viewport.addEventListener("wheel", (e) => {
    if (state.currentView === "month") {
      if (Math.abs(e.deltaY) < 18) return;
      e.preventDefault();
      if (wheelLock) return;
      wheelLock = true;
      stepMonth(e.deltaY > 0 ? 1 : -1);
      setTimeout(() => { wheelLock = false; }, 260);
    } else if (state.currentView === "week") {
      if (Math.abs(e.deltaY) < 22) return;
      e.preventDefault();
      if (wheelLock) return;
      wheelLock = true;
      state.weekStart.setDate(state.weekStart.getDate() + (e.deltaY > 0 ? 7 : -7));
      renderWeekView();
      setTimeout(() => { wheelLock = false; }, 240);
    }
  }, { passive: false });

  viewport.addEventListener("scroll", handleInfiniteScroll);

  // Keyboard Shortcuts (0ms Latency)
  document.addEventListener("keydown", (e) => {
    const tag = document.activeElement ? document.activeElement.tagName : "";
    if (tag === "TEXTAREA" || tag === "INPUT" || tag === "SELECT") {
      if (e.key === "Escape") {
        closeDrawer();
        closeAllModals();
        closeAlertPopover();
      }
      return;
    }

    if (e.key === "Escape") {
      closeDrawer();
      closeAllModals();
      closeSessionDropdown();
      closeAlertPopover();
      closeAiClarifySheet();
    } else if (e.altKey && (e.key === "t" || e.key === "T")) {
      e.preventDefault();
      runWithoutMotion(() => cycleTheme());
    } else if (e.altKey && (e.key === "n" || e.key === "N")) {
      e.preventDefault();
      openNewSessionModal();
    } else if (e.key === "t" || e.key === "T") {
      e.preventDefault();
      navigateToDate(TODAY, false);
    } else if (e.key === "s" || e.key === "S") {
      e.preventDefault();
      cycleSidebarState(true);
    } else if ((e.ctrlKey || e.altKey) && e.key === ",") {
      e.preventDefault();
      document.getElementById("btnOpenSettings")?.click();
    } else if (e.key === "PageDown" || (e.key === "ArrowDown" && state.currentView === "month")) {
      e.preventDefault();
      stepMonth(1);
    } else if (e.key === "PageUp" || (e.key === "ArrowUp" && state.currentView === "month")) {
      e.preventDefault();
      stepMonth(-1);
    }
  });

  // Responsive Window Auto-Adaptation Listener
  let windowResizeTimer = null;
  window.addEventListener("resize", () => {
    clearTimeout(windowResizeTimer);
    windowResizeTimer = setTimeout(() => {
      handleWindowResize();
    }, 90);
  });
}

function handleWindowResize() {
  const width = window.innerWidth;
  // If window is narrow and sidebar is expanded without manual user lock, auto-compact to rail
  if (width < 1180 && state.sidebarState === "expanded" && !state.userPinnedSidebar) {
    state.sidebarState = "compact";
    state.autoCompacted = true;
    applySidebarState();
  } else if (width >= 1180 && state.sidebarState === "compact" && state.autoCompacted) {
    state.sidebarState = "expanded";
    state.autoCompacted = false;
    applySidebarState();
  }
}

function cycleSidebarState() {
  state.sidebarState = state.sidebarState === "expanded" ? "compact" : "expanded";
  state.userPinnedSidebar = true;
  state.autoCompacted = false;
  applySidebarState();
}

function applySidebarState() {
  const sb = document.getElementById("appSidebar");
  const btn = document.getElementById("btnToggleSidebar");
  if (!sb) return;

  sb.classList.remove("compact", "compact-rail", "collapsed");
  if (state.sidebarState === "compact") {
    sb.classList.add("compact");
    sb.classList.add("compact-rail");
    if (btn) {
      btn.classList.remove("active");
      btn.title = "Espandi Barra Laterale (S)";
    }
  } else {
    state.sidebarState = "expanded";
    if (btn) {
      btn.classList.add("active");
      btn.title = "Riduci Barra Laterale (S)";
    }
  }
  localStorage.setItem("polimi_sidebar_state", state.sidebarState);
}

// ==========================================================================
// 11. MODALS & FORMS (Prove in Itinere & Eventi Generici)
// ==========================================================================
function setModalOpenState(modalEl, isOpen) {
  if (!modalEl) return;
  modalEl.classList.toggle("hidden", !isOpen);
  modalEl.setAttribute("data-open", isOpen ? "true" : "false");
}

function initModals() {
  document.querySelectorAll(".modal-overlay").forEach(overlay => {
    overlay.addEventListener("click", (e) => {
      if (e.target === overlay) {
        setModalOpenState(overlay, false);
      }
    });
  });

  // 1. Nuova Sessione
  const btnOpenNew = document.getElementById("btnOpenNewSessionModal");
  const modalNew = document.getElementById("modalNewSessionOverlay");
  const btnCloseNew = document.getElementById("btnCloseNewSessionModal");
  const btnCancelNew = document.getElementById("btnCancelNewSession");
  const formNew = document.getElementById("formNewSession");

  if (btnOpenNew) btnOpenNew.addEventListener("click", () => { closeSessionDropdown(); openNewSessionModal(); });
  if (btnCloseNew) btnCloseNew.addEventListener("click", () => setModalOpenState(modalNew, false));
  if (btnCancelNew) btnCancelNew.addEventListener("click", () => setModalOpenState(modalNew, false));
  if (formNew) {
    formNew.addEventListener("submit", (e) => {
      e.preventDefault();
      const name = document.getElementById("inputSessionName").value;
      const subtitle = document.getElementById("inputSessionSub").value;
      const startDate = document.getElementById("inputSessionStart").value;
      const endDate = document.getElementById("inputSessionEnd").value;
      store.createSession({ name, subtitle, startDate, endDate });
      setModalOpenState(modalNew, false);
      formNew.reset();
      onSessionChanged();
    });
  }

  // 2. Aggiungi Esame / Prove in Itinere
  const btnOpenAdd1 = document.getElementById("btnOpenAddExamModal");
  const btnOpenAdd2 = document.getElementById("btnTopAddExam");
  const modalAdd = document.getElementById("modalAddExamOverlay");
  const btnCloseAdd = document.getElementById("btnCloseAddExamModal");
  const btnCancelAdd = document.getElementById("btnCancelAddExam");
  const formAdd = document.getElementById("formAddExam");
  const selectCourse = document.getElementById("selectExamCourse");
  const newCourseRow = document.getElementById("newCourseRow");
  const selectRound = document.getElementById("selectExamRound");
  const singleDateRow = document.getElementById("singleDateRow");
  const itinereDatesRow = document.getElementById("itinereDatesRow");
  const inputSingleDate = document.getElementById("inputExamDate");
  const inputItin1 = document.getElementById("inputExamDateItinere1");
  const inputItin2 = document.getElementById("inputExamDateItinere2");

  if (btnOpenAdd1) btnOpenAdd1.addEventListener("click", () => openAddExamModal());
  if (btnOpenAdd2) btnOpenAdd2.addEventListener("click", () => openAddExamModal());
  if (btnCloseAdd) btnCloseAdd.addEventListener("click", () => setModalOpenState(modalAdd, false));
  if (btnCancelAdd) btnCancelAdd.addEventListener("click", () => setModalOpenState(modalAdd, false));

  if (selectCourse) {
    selectCourse.addEventListener("change", () => {
      newCourseRow.style.display = selectCourse.value === "__new__" ? "flex" : "none";
    });
  }

  if (selectRound) {
    selectRound.addEventListener("change", () => {
      const isItinere = selectRound.value === "itinere";
      singleDateRow.style.display = isItinere ? "none" : "flex";
      itinereDatesRow.style.display = isItinere ? "flex" : "none";
      inputSingleDate.required = !isItinere;
      inputItin1.required = isItinere;
      inputItin2.required = isItinere;
    });
  }

  if (formAdd) {
    formAdd.addEventListener("submit", (e) => {
      e.preventDefault();
      let courseId = selectCourse.value;
      if (courseId === "__new__") {
        const sigla = document.getElementById("inputNewCourseSigla").value || "NUOVO";
        const name = document.getElementById("inputNewCourseName").value || sigla;
        const newC = store.addCourse({ sigla, name, cfu: 6 });
        courseId = newC.id;
      }

      const roundVal = selectRound.value;
      const details = document.getElementById("inputExamDetails").value;

      if (roundVal === "itinere") {
        const d1 = inputItin1.value;
        const d2 = inputItin2.value;
        store.addItinereExams({ courseId, date1: d1, date2: d2, details });
        navigateToDate(parseDateISO(d1), true);
      } else {
        const date = inputSingleDate.value;
        const roundNum = parseInt(roundVal, 10) || 1;
        store.addExam({ courseId, date, round: roundNum, details });
        navigateToDate(parseDateISO(date), true);
      }

      setModalOpenState(modalAdd, false);
      formAdd.reset();
      newCourseRow.style.display = "none";
      singleDateRow.style.display = "flex";
      itinereDatesRow.style.display = "none";

      initExamFilters();
      initHorizonRibbon();
      updateAlertsUI();
      updateTelemetry();
      renderApp();
    });
  }

  // 3. Aggiungi Evento Generico
  const modalGeneric = document.getElementById("modalAddGenericEventOverlay");
  const btnCloseGeneric = document.getElementById("btnCloseAddGenericEventModal");
  const btnCancelGeneric = document.getElementById("btnCancelAddGenericEvent");
  const formGeneric = document.getElementById("formAddGenericEvent");

  if (btnCloseGeneric) btnCloseGeneric.addEventListener("click", () => setModalOpenState(modalGeneric, false));
  if (btnCancelGeneric) btnCancelGeneric.addEventListener("click", () => setModalOpenState(modalGeneric, false));
  if (formGeneric) {
    formGeneric.addEventListener("submit", (e) => {
      e.preventDefault();
      const title = document.getElementById("inputGenericEventTitle").value;
      const date = document.getElementById("inputGenericEventDate").value;
      const time = document.getElementById("inputGenericEventTime").value;
      const category = document.getElementById("selectGenericEventCategory").value;
      const details = document.getElementById("inputGenericEventDetails").value;

      store.addGenericEvent({ date, title, time, category, details });
      setModalOpenState(modalGeneric, false);
      formGeneric.reset();
      renderApp();
      openDrawer(date, parseDateISO(date));
    });
  }

  // 4. Gestisci Sessioni
  const btnOpenManage = document.getElementById("btnOpenManageSessionsModal");
  const modalManage = document.getElementById("modalManageSessionsOverlay");
  const btnCloseManage = document.getElementById("btnCloseManageSessionsModal");
  const btnCloseManage2 = document.getElementById("btnCloseManageSessions");

  if (btnOpenManage) btnOpenManage.addEventListener("click", () => { closeSessionDropdown(); openManageSessionsModal(); });
  if (btnCloseManage) btnCloseManage.addEventListener("click", () => setModalOpenState(modalManage, false));
  if (btnCloseManage2) btnCloseManage2.addEventListener("click", () => setModalOpenState(modalManage, false));
}

function openNewSessionModal() {
  const modal = document.getElementById("modalNewSessionOverlay");
  if (modal) {
    setModalOpenState(modal, true);
    const dStart = document.getElementById("inputSessionStart");
    const dEnd = document.getElementById("inputSessionEnd");
    if (dStart) dStart.value = "2027-06-01";
    if (dEnd) dEnd.value = "2027-07-31";
    document.getElementById("inputSessionName").focus();
  }
}

function openAddExamModal(prefillDate = null) {
  const modal = document.getElementById("modalAddExamOverlay");
  const select = document.getElementById("selectExamCourse");
  const dateInput = document.getElementById("inputExamDate");
  const newCourseRow = document.getElementById("newCourseRow");
  if (!modal || !select) return;

  const session = store.getActiveSession();
  const courses = Object.values(session.courses || {});

  select.innerHTML = "";
  courses.forEach(c => {
    const opt = document.createElement("option");
    opt.value = c.id;
    opt.textContent = `${c.sigla} — ${c.name} (${c.cfu} CFU)`;
    select.appendChild(opt);
  });

  const optNew = document.createElement("option");
  optNew.value = "__new__";
  optNew.textContent = "+ Crea Nuova Materia...";
  select.appendChild(optNew);

  newCourseRow.style.display = courses.length === 0 ? "flex" : "none";
  if (courses.length === 0) select.value = "__new__";

  if (dateInput) {
    dateInput.value = prefillDate || state.activeDateString || formatDateISO(TODAY);
  }

  setModalOpenState(modal, true);
}

function openAddGenericEventModal(prefillDate = null) {
  const modal = document.getElementById("modalAddGenericEventOverlay");
  const dateInput = document.getElementById("inputGenericEventDate");
  if (!modal) return;
  if (dateInput) {
    dateInput.value = prefillDate || state.activeDateString || formatDateISO(TODAY);
  }
  setModalOpenState(modal, true);
  document.getElementById("inputGenericEventTitle").focus();
}

function openManageSessionsModal() {
  const modal = document.getElementById("modalManageSessionsOverlay");
  const list = document.getElementById("manageSessionsList");
  if (!modal || !list) return;

  list.innerHTML = "";
  store.data.sessions.forEach(s => {
    const card = document.createElement("div");
    card.className = "manage-session-card";
    const isActive = s.id === store.data.activeSessionId;

    card.innerHTML = `
      <div>
        <div style="font-weight:700;">${s.name} ${isActive ? `<span style="font-size:0.75rem; color:var(--accent); margin-left:6px;">(Attiva)</span>` : ""}</div>
        <div style="font-size:0.78rem; color:var(--text-secondary);">${s.subtitle || ""} · ${(s.exams || []).length} appelli</div>
        <div class="mono" style="font-size:0.72rem; color:var(--text-tertiary); margin-top:2px;">${s.startDate || "N/A"} → ${s.endDate || "N/A"}</div>
      </div>
      <div style="display:flex; align-items:center; gap:6px;">
        ${!isActive ? `<button type="button" class="btn-primary" style="height:28px; padding:0 10px; font-size:0.76rem;" data-action="switch" data-id="${s.id}">Attiva</button>` : ""}
        ${store.data.sessions.length > 1 ? `<button type="button" class="btn-secondary" style="height:28px; padding:0 10px; font-size:0.76rem; color:var(--danger-text);" data-action="delete" data-id="${s.id}">Elimina</button>` : ""}
      </div>
    `;

    card.querySelectorAll("button").forEach(b => {
      b.addEventListener("click", () => {
        const action = b.dataset.action;
        const id = b.dataset.id;
        if (action === "switch") {
          store.setActiveSession(id);
          setModalOpenState(modal, false);
          onSessionChanged();
        } else if (action === "delete") {
          if (confirm(`Sei sicuro di voler eliminare definitivamente "${s.name}"?`)) {
            store.deleteSession(id);
            openManageSessionsModal();
            onSessionChanged();
          }
        }
      });
    });

    list.appendChild(card);
  });

  setModalOpenState(modal, true);
}

function closeAllModals() {
  document.querySelectorAll(".modal-overlay").forEach(m => setModalOpenState(m, false));
}

// ==========================================================================
// 12. AI FLOATING ISLAND & WEB SPEECH ASSISTANT
// ==========================================================================
function initAiAssistant() {
  const micBtn = document.getElementById("btnAiIslandMic");
  const submitBtn = document.getElementById("btnAiIslandSubmit");
  const inputEl = document.getElementById("inputAiIslandText");
  const waveEl = document.getElementById("aiIslandListeningWave");
  const closeClarifyBtn = document.getElementById("btnCloseAiClarify");

  if (closeClarifyBtn) {
    closeClarifyBtn.addEventListener("click", closeAiClarifySheet);
  }

  // Web Speech API Initialization
  const SpeechRecognition = window.SpeechRecognition || window.webkitSpeechRecognition;
  let isListening = false;

  if (SpeechRecognition && micBtn) {
    speechRecognizer = new SpeechRecognition();
    speechRecognizer.lang = "it-IT";
    speechRecognizer.continuous = false;
    speechRecognizer.interimResults = true;

    speechRecognizer.onstart = () => {
      isListening = true;
      state.speechAccumulatedText = "";
      micBtn.classList.add("listening");
      if (waveEl) waveEl.classList.remove("hidden");
      if (inputEl) inputEl.placeholder = "Ti ascolto... detta pure l'evento";
    };

    speechRecognizer.onresult = (event) => {
      let currentTranscript = "";
      for (let i = 0; i < event.results.length; ++i) {
        currentTranscript += event.results[i][0].transcript;
      }
      if (inputEl) inputEl.value = currentTranscript;
      state.speechAccumulatedText = currentTranscript;
      // Do NOT trigger questions or submissions here! Wait until dictation is complete.
    };

    speechRecognizer.onerror = (e) => {
      console.warn("Speech recognition notice:", e.error);
      stopListening();
    };

    speechRecognizer.onend = () => {
      stopListening();
      if (inputEl) inputEl.placeholder = "Scrivi o detta un evento... (es. Secondo appello MAM l'8 febbraio)";
      // Process submission only now, when speech has fully concluded
      const finalRecorded = (state.speechAccumulatedText || (inputEl ? inputEl.value : "")).trim();
      state.speechAccumulatedText = "";
      if (finalRecorded) {
        handleAiInputSubmission(finalRecorded);
      }
    };

    micBtn.addEventListener("click", () => {
      if (isListening) {
        speechRecognizer.stop();
      } else {
        try {
          speechRecognizer.start();
        } catch (err) {
          console.warn("Speech already active:", err);
        }
      }
    });
  } else if (micBtn) {
    micBtn.title = "Dettatura vocale supportata nativamente in Edge/Chrome";
  }

  function stopListening() {
    isListening = false;
    if (micBtn) micBtn.classList.remove("listening");
    if (waveEl) waveEl.classList.add("hidden");
  }

  if (submitBtn && inputEl) {
    submitBtn.addEventListener("click", () => {
      if (inputEl.value.trim()) {
        handleAiInputSubmission(inputEl.value.trim());
      }
    });

    inputEl.addEventListener("keydown", (e) => {
      if (e.key === "Enter" && inputEl.value.trim()) {
        e.preventDefault();
        handleAiInputSubmission(inputEl.value.trim());
      }
    });
  }
}

let aiToastTimer = null;
function showAiToast(message) {
  const toast = document.getElementById("aiIslandToast");
  if (!toast) return;
  toast.textContent = message;
  toast.classList.remove("hidden");
  void toast.offsetWidth;
  toast.classList.add("visible");

  clearTimeout(aiToastTimer);
  aiToastTimer = setTimeout(() => {
    toast.classList.remove("visible");
    setTimeout(() => {
      toast.classList.add("hidden");
    }, 240);
  }, 2600);
}

function clearAiInput() {
  const inputEl = document.getElementById("inputAiIslandText");
  if (inputEl) inputEl.value = "";
  closeAiClarifySheet();
}

function parseTodoFromNaturalLanguage(text, session) {
  const lower = text.toLowerCase();
  let date = formatDateISO(TODAY);

  if (lower.includes("domani")) {
    const d = new Date(TODAY.getTime() + 86400000);
    date = formatDateISO(d);
  } else if (lower.includes("oggi")) {
    date = formatDateISO(TODAY);
  } else {
    const isoMatch = lower.match(/\b(202[6-9])-(\d{1,2})-(\d{1,2})\b/);
    if (isoMatch) {
      date = `${isoMatch[1]}-${String(isoMatch[2]).padStart(2, "0")}-${String(isoMatch[3]).padStart(2, "0")}`;
    } else {
      const monthsMap = {
        gennaio: "01", febbraio: "02", marzo: "03", aprile: "04", maggio: "05", giugno: "06",
        luglio: "07", agosto: "08", settembre: "09", ottobre: "10", novembre: "11", dicembre: "12",
        gen: "01", feb: "02", mar: "03", apr: "04", mag: "05", giu: "06",
        lug: "07", ago: "08", set: "09", ott: "10", nov: "11", dic: "12"
      };
      const dayMonthRegex = /\b(\d{1,2})\s+(gennaio|febbraio|marzo|aprile|maggio|giugno|luglio|agosto|settembre|ottobre|novembre|dicembre|gen|feb|mar|apr|mag|giu|lug|ago|set|ott|nov|dic)\b(?:\s+(202[6-9]))?/i;
      const dmMatch = lower.match(dayMonthRegex);
      if (dmMatch) {
        const day = String(dmMatch[1]).padStart(2, "0");
        const month = monthsMap[dmMatch[2].toLowerCase()];
        const year = dmMatch[3] || (parseInt(month, 10) <= 2 ? "2027" : "2026");
        date = `${year}-${month}-${day}`;
      } else if (state.activeDateString) {
        date = state.activeDateString;
      }
    }
  }

  let cleaned = text
    .replace(/aggiungi\s+(?:un\s+)?(?:to-do|todo|compito|promemoria|attività|nota)/gi, "")
    .replace(/(?:to-do|todo|compito|promemoria|da\s+fare)/gi, "")
    .replace(/per\s+(?:il\s+)?\d{1,2}\s+(?:gennaio|febbraio|marzo|aprile|maggio|giugno|luglio|agosto|settembre|ottobre|novembre|dicembre|gen|feb|mar|apr|mag|giu|lug|ago|set|ott|nov|dic)(?:\s+202[6-9])?/gi, "")
    .replace(/per\s+(?:il\s+)?202[6-9]-\d{1,2}-\d{1,2}/gi, "")
    .replace(/per\s+domani/gi, "")
    .replace(/per\s+oggi/gi, "")
    .replace(/^[\s:;,-]+/, "")
    .trim();

  if (!cleaned) cleaned = "Promemoria di studio";
  return { date, text: cleaned };
}

function getUpcomingExamInfo(session) {
  const exams = session.exams || [];
  const sortedExams = [...exams].sort((a, b) => a.date.localeCompare(b.date));
  const upcoming = sortedExams.find(e => parseDateISO(e.date) >= TODAY) || sortedExams[0];
  if (!upcoming) return null;
  const uDate = parseDateISO(upcoming.date);
  const diffDays = Math.ceil((uDate - TODAY) / (1000 * 60 * 60 * 24));
  const c = session.courses[upcoming.courseId];
  return {
    courseName: c ? c.name : "Esame",
    sigla: c ? c.sigla : "Esame",
    dateFormatted: formatDateFriendly(uDate),
    daysLeft: diffDays,
    dateObj: uDate
  };
}

function handleAiInputSubmission(rawText) {
  if (!rawText || !rawText.trim()) return;
  const text = rawText.trim();
  const lower = text.toLowerCase();
  const session = store.getActiveSession();

  // 1. TEMA (THEME)
  if (lower.includes("tema scuro") || lower.includes("dark mode") || lower.includes("modalità scura") || lower.includes("sfondo scuro") || lower.includes("metti scuro") || lower.includes("passa a scuro")) {
    setThemeChoice("dark");
    showAiToast("✓ Tema scuro applicato");
    clearAiInput();
    return;
  }
  if (lower.includes("tema chiaro") || lower.includes("light mode") || lower.includes("modalità chiara") || lower.includes("sfondo bianco") || lower.includes("metti chiaro") || lower.includes("passa a chiaro")) {
    setThemeChoice("light");
    showAiToast("✓ Tema chiaro applicato");
    clearAiInput();
    return;
  }
  if (lower.includes("tema auto") || lower.includes("tema automatico") || lower.includes("segui sistema") || lower.includes("tema sistema") || lower.includes("tema predefinito")) {
    setThemeChoice("auto");
    showAiToast("✓ Tema sincronizzato con il sistema");
    clearAiInput();
    return;
  }

  // 2. DIMENSIONI & ZOOM (UI SCALE PRESETS)
  const UI_SCALE_PRESETS = [85, 90, 100, 115, 125];
  const scaleMatch = lower.match(/(?:zoom|scala|dimensione|dimensioni|grandezza).*?(\d{2,3})\s*%?/i) || lower.match(/(\d{2,3})\s*%/);
  if (scaleMatch && (lower.includes("zoom") || lower.includes("scala") || lower.includes("dimensione") || lower.includes("grandezza") || lower.includes("%"))) {
    const val = parseInt(scaleMatch[1], 10);
    if (val >= 70 && val <= 150) {
      // Trova l'opzione predefinita più vicina o applica il valore esatto
      const closest = UI_SCALE_PRESETS.reduce((prev, curr) => Math.abs(curr - val) < Math.abs(prev - val) ? curr : prev);
      applyUiScale(closest);
      syncSettingsModalInputs();
      showAiToast(`✓ Dimensioni impostate su opzione predefinita: ${closest}%`);
      clearAiInput();
      return;
    }
  }
  if (lower.includes("ingrandisci") || lower.includes("fai più grande") || lower.includes("aumenta dimensioni") || lower.includes("zoom in")) {
    const cur = state.settings.uiScale || 100;
    const nextVal = UI_SCALE_PRESETS.find(p => p > cur) || 125;
    applyUiScale(nextVal);
    syncSettingsModalInputs();
    showAiToast(`✓ Ingrandito all'opzione predefinita: ${nextVal}%`);
    clearAiInput();
    return;
  }
  if (lower.includes("rimpicciolisci") || lower.includes("fai più piccolo") || lower.includes("riduci dimensioni") || lower.includes("zoom out")) {
    const cur = state.settings.uiScale || 100;
    const prevVal = [...UI_SCALE_PRESETS].reverse().find(p => p < cur) || 85;
    applyUiScale(prevVal);
    syncSettingsModalInputs();
    showAiToast(`✓ Rimpicciolito all'opzione predefinita: ${prevVal}%`);
    clearAiInput();
    return;
  }
  if (lower.includes("zoom compatto") || lower.includes("scala compatta")) {
    applyUiScale(85);
    syncSettingsModalInputs();
    showAiToast("✓ Dimensioni impostate su: Compatto (85%)");
    clearAiInput();
    return;
  }
  if (lower.includes("zoom ridotto") || lower.includes("scala ridotta")) {
    applyUiScale(90);
    syncSettingsModalInputs();
    showAiToast("✓ Dimensioni impostate su: Ridotto (90%)");
    clearAiInput();
    return;
  }
  if (lower.includes("zoom medio") || lower.includes("scala media")) {
    applyUiScale(115);
    syncSettingsModalInputs();
    showAiToast("✓ Dimensioni impostate su: Medio (115%)");
    clearAiInput();
    return;
  }
  if (lower.includes("zoom grande") || lower.includes("scala grande")) {
    applyUiScale(125);
    syncSettingsModalInputs();
    showAiToast("✓ Dimensioni impostate su: Grande (125%)");
    clearAiInput();
    return;
  }
  if (lower.includes("resetta dimensioni") || lower.includes("dimensione normale") || lower.includes("zoom normale") || lower.includes("zoom standard") || lower.includes("zoom 100")) {
    applyUiScale(100);
    syncSettingsModalInputs();
    showAiToast("✓ Dimensioni ripristinate su Standard (100%)");
    clearAiInput();
    return;
  }

  // 3. COLORI: ACCENTO, FESTIVITÀ & DOMENICA (CON SUPPORTO CODICI HEX E COPIA)
  const isHolidayMention = lower.includes("festivita") || lower.includes("festività") || lower.includes("festa") || lower.includes("vacanze");
  const isSundayColorMention = lower.includes("domenica") || lower.includes("riposo") || lower.includes("fine settimana");
  const explicitHexMatch = text.match(/#([0-9a-fA-F]{6}|[0-9a-fA-F]{3})\b/);

  // 3a. Comandi Copia Codice Colore
  if (lower.includes("copia") && (lower.includes("colore") || lower.includes("hex") || lower.includes("esadecimale"))) {
    if (isSundayColorMention) {
      copyColorHex(state.settings.sundayColor, document.getElementById("copyBtn_sunday") || document.getElementById("btnCopySundayHex"));
      clearAiInput();
      return;
    }
    if (isHolidayMention) {
      copyColorHex(state.settings.holidayColor, document.getElementById("copyBtn_holiday") || document.getElementById("btnCopyHolidayHex"));
      clearAiInput();
      return;
    }
    copyColorHex(state.settings.accentColor, document.getElementById("copyBtn_accent") || document.getElementById("btnCopyAccentHex"));
    clearAiInput();
    return;
  }

  // 3b. Mappa Colori Nominali
  const colorMap = {
    "blu": "#007AFF",
    "azzurro": "#30B0C7",
    "viola": "#5856D6",
    "purple": "#5856D6",
    "rosa": "#FF2D55",
    "pink": "#FF2D55",
    "arancione": "#FF9500",
    "orange": "#FF9500",
    "ambra": "#F59E0B",
    "giallo": "#EAB308",
    "verde": "#34C759",
    "green": "#34C759",
    "smeraldo": "#10B981",
    "teal": "#30B0C7",
    "rosso": "#FF3B30",
    "corallo": "#FF6482",
    "red": "#FF3B30"
  };

  // Se è presente un codice esadecimale esplicito (es: #FF9500)
  if (explicitHexMatch) {
    const rawHex = explicitHexMatch[0];
    const cleanHex = normalizeHexColor(rawHex);
    if (cleanHex) {
      if (isSundayColorMention) {
        applySundayColor(cleanHex);
        syncSettingsModalInputs();
        renderApp();
        showAiToast(`✓ Colore domenica impostato su ${cleanHex}`);
        clearAiInput();
        return;
      }
      if (isHolidayMention) {
        applyHolidayColor(cleanHex);
        syncSettingsModalInputs();
        renderApp();
        showAiToast(`✓ Colore festività impostato su ${cleanHex}`);
        clearAiInput();
        return;
      }
      applyAccentColor(cleanHex);
      syncSettingsModalInputs();
      showAiToast(`✓ Colore accento impostato su ${cleanHex}`);
      clearAiInput();
      return;
    }
  }

  // Colore Domenica / Riposo per nome
  if (isSundayColorMention && (lower.includes("colore") || lower.includes("cambia") || lower.includes("imposta") || lower.includes("metti"))) {
    for (const [name, hex] of Object.entries(colorMap)) {
      if (lower.includes(name)) {
        applySundayColor(hex);
        syncSettingsModalInputs();
        renderApp();
        showAiToast(`✓ Colore evidenziazione domenica impostato su ${name}`);
        clearAiInput();
        return;
      }
    }
  }

  // Colore Festività per nome
  if (isHolidayMention && (lower.includes("colore") || lower.includes("cambia") || lower.includes("imposta") || lower.includes("metti"))) {
    for (const [name, hex] of Object.entries(colorMap)) {
      if (lower.includes(name)) {
        applyHolidayColor(hex);
        syncSettingsModalInputs();
        renderApp();
        showAiToast(`✓ Colore festività impostato su ${name}`);
        clearAiInput();
        return;
      }
    }
  }

  // Colore Accento per nome
  if (!isHolidayMention && !isSundayColorMention && (lower.includes("colore accento") || lower.includes("colore principale") || lower.includes("colore app") || lower.includes("cambia colore in") || lower.includes("imposta colore"))) {
    for (const [name, hex] of Object.entries(colorMap)) {
      if (lower.includes(name)) {
        applyAccentColor(hex);
        syncSettingsModalInputs();
        showAiToast(`✓ Colore principale impostato su ${name}`);
        clearAiInput();
        return;
      }
    }
  }

  // 4. GIORNO INIZIO SETTIMANA
  if (lower.includes("inizio settimana") || lower.includes("settimana inizia") || lower.includes("primo giorno della settimana") || lower.includes("inizia la settimana")) {
    let day = 1;
    let dayName = "Lunedì";
    if (lower.includes("domenica")) {
      day = 0; dayName = "Domenica";
    } else if (lower.includes("sabato")) {
      day = 6; dayName = "Sabato";
    }
    state.settings.startDayOfWeek = day;
    localStorage.setItem("polimi_start_day", day);
    renderStickyWeekdayHeader();
    syncSettingsModalInputs();
    renderApp();
    showAiToast(`✓ Inizio settimana impostato su ${dayName}`);
    clearAiInput();
    return;
  }

  // 5. GIORNO FESTIVO DELLA SETTIMANA
  if (lower.includes("festivo della settimana") || lower.includes("giorno festivo settimanale") || lower.includes("evidenzia come festivo") || lower.includes("festivo settimanale")) {
    let day = 0;
    let dayName = "Domenica";
    if (lower.includes("nessuno") || lower.includes("togli") || lower.includes("disattiva")) {
      day = -1; dayName = "Nessuno";
    } else if (lower.includes("sabato")) {
      day = 6; dayName = "Sabato";
    } else if (lower.includes("venerdì") || lower.includes("venerdi")) {
      day = 5; dayName = "Venerdì";
    }
    state.settings.weeklyHolidayDay = day;
    localStorage.setItem("polimi_weekly_holiday", day);
    renderStickyWeekdayHeader();
    syncSettingsModalInputs();
    renderApp();
    showAiToast(`✓ Giorno festivo settimanale: ${dayName}`);
    clearAiInput();
    return;
  }

  // 5b. STILE CORNICE & BORDI FINESTRA
  if (lower.includes("bordo personalizzato") || lower.includes("cornice personalizzata") || lower.includes("cornice custom") || lower.includes("rimuovi bordi standard") || lower.includes("metti cornice crafted") || lower.includes("togli bordi windows") || lower.includes("bordo crafted") || lower.includes("nascondi cornice windows") || lower.includes("nascondi quella predefinita") || lower.includes("togli cornice predefinita") || lower.includes("nascondi barra windows")) {
    applyWindowFrameStyle("custom");
    showAiToast("✓ Cornice custom attivata (cornice predefinita di Windows nascosta)");
    clearAiInput();
    return;
  }
  if (lower.includes("bordo standard") || lower.includes("cornice standard") || lower.includes("cornice predefinita") || lower.includes("bordi di windows") || lower.includes("barra standard") || lower.includes("bordo nativo") || lower.includes("mostra cornice windows") || lower.includes("ripristina cornice predefinita")) {
    applyWindowFrameStyle("standard");
    showAiToast("✓ Cornice standard di Windows ripristinata");
    clearAiInput();
    return;
  }

  // 6. APERTURA / CHIUSURA IMPOSTAZIONI
  if (lower.includes("apri impostazioni") || lower.includes("mostra impostazioni") || lower.includes("pannello impostazioni") || lower.includes("apri preferenze") || lower.includes("menu impostazioni")) {
    openSettings();
    showAiToast("✓ Pannello impostazioni aperto");
    clearAiInput();
    return;
  }
  if (lower.includes("chiudi impostazioni") || lower.includes("salva impostazioni") || lower.includes("nascondi impostazioni")) {
    closeSettings();
    showAiToast("✓ Impostazioni chiuse");
    clearAiInput();
    return;
  }
  if (lower.includes("resetta impostazioni") || lower.includes("ripristina impostazioni") || lower.includes("impostazioni default") || lower.includes("impostazioni predefinite")) {
    document.getElementById("btnResetSettingsDefaults")?.click();
    showAiToast("✓ Impostazioni ripristinate ai valori predefiniti");
    clearAiInput();
    return;
  }

  // 7. FILTRI CORSI / COSA VISUALIZZARE O NO
  if (lower.includes("mostra solo") || lower.includes("filtra per") || lower.includes("visualizza solo") || lower.includes("fai vedere solo")) {
    const matchedCourseIds = [];
    const courses = Object.values(session.courses || {});
    courses.forEach(c => {
      const s = c.sigla.toLowerCase();
      const n = c.name.toLowerCase();
      if (lower.includes(s) || lower.includes(n) || (s === "mam" && (lower.includes("meccanica") || lower.includes("rocchi"))) || (s === "tm" && lower.includes("tecnologia")) || (s === "mse" && lower.includes("macchine")) || (s === "dcm" && lower.includes("dinamica")) || (s === "psm" && lower.includes("progettazione"))) {
        matchedCourseIds.push(c.id);
      }
    });

    if (matchedCourseIds.length > 0) {
      state.selectedCourseFilters.clear();
      matchedCourseIds.forEach(id => state.selectedCourseFilters.add(id));
      updateFilterChipsUI();
      renderApp();
      const sigle = matchedCourseIds.map(id => session.courses[id]?.sigla || id).join(", ");
      showAiToast(`✓ Visualizzazione filtrata: solo ${sigle}`);
      clearAiInput();
      return;
    }
  }

  if (lower.includes("nascondi corso") || lower.includes("togli dal filtro") || lower.includes("deseleziona")) {
    const courses = Object.values(session.courses || {});
    let hiddenSigla = "";
    courses.forEach(c => {
      if (lower.includes(c.sigla.toLowerCase()) || lower.includes(c.name.toLowerCase())) {
        state.selectedCourseFilters.delete(c.id);
        hiddenSigla = c.sigla;
      }
    });
    if (hiddenSigla) {
      updateFilterChipsUI();
      renderApp();
      showAiToast(`✓ Rimosso dal filtro: ${hiddenSigla}`);
      clearAiInput();
      return;
    }
  }

  if (lower.includes("mostra tutti i corsi") || lower.includes("mostra tutte le materie") || lower.includes("mostra tutti") || lower.includes("resetta filtri") || lower.includes("togli filtri") || lower.includes("mostra tutto")) {
    state.selectedCourseFilters.clear();
    state.selectedRoundFilter = "all";
    document.querySelectorAll(".pill-round").forEach(p => p.classList.remove("active"));
    document.querySelector('.pill-round[data-round="all"]')?.classList.add("active");
    updateFilterChipsUI();
    renderApp();
    showAiToast("✓ Mostrati tutti i corsi e gli appelli");
    clearAiInput();
    return;
  }

  // 8. FILTRO GIRI D'APPELLO
  if (lower.includes("primo giro") || lower.includes("1° giro") || lower.includes("primo appello") || lower.includes("solo 1°")) {
    state.selectedRoundFilter = "1";
    document.querySelectorAll(".pill-round").forEach(p => p.classList.toggle("active", p.dataset.round === "1"));
    renderApp();
    showAiToast("✓ Visualizzazione filtrata: solo 1° Giro");
    clearAiInput();
    return;
  }
  if (lower.includes("secondo giro") || lower.includes("2° giro") || lower.includes("secondo appello") || lower.includes("solo 2°")) {
    state.selectedRoundFilter = "2";
    document.querySelectorAll(".pill-round").forEach(p => p.classList.toggle("active", p.dataset.round === "2"));
    renderApp();
    showAiToast("✓ Visualizzazione filtrata: solo 2° Giro");
    clearAiInput();
    return;
  }

  // 9. MODALITÀ DI VISUALIZZAZIONE
  if (lower.includes("vista continuo") || lower.includes("scorrimento continuo") || lower.includes("vista continua") || lower.includes("calendario continuo") || lower.includes("scroll infinito")) {
    switchView("continuous");
    showAiToast("✓ Vista Continuo attivata");
    clearAiInput();
    return;
  }
  if (lower.includes("vista mese") || lower.includes("mese singolo") || lower.includes("mostra mese")) {
    switchView("month");
    showAiToast("✓ Vista Mese singolo attivata");
    clearAiInput();
    return;
  }
  if (lower.includes("vista settimana") || lower.includes("mostra settimana") || lower.includes("settimanale")) {
    switchView("week");
    showAiToast("✓ Vista Settimana attivata");
    clearAiInput();
    return;
  }
  if (lower.includes("vista appelli") || lower.includes("elenco appelli") || lower.includes("vista agenda") || lower.includes("mostra agenda") || lower.includes("lista esami")) {
    switchView("agenda");
    showAiToast("✓ Vista Appelli (Agenda) attivata");
    clearAiInput();
    return;
  }

  // 10. SIDEBAR (COMPATTA / ESPANDI)
  if (lower.includes("compatta sidebar") || lower.includes("chiudi sidebar") || lower.includes("riduci sidebar") || lower.includes("compatta menu") || lower.includes("riduci menu")) {
    state.sidebarState = "compact";
    state.userPinnedSidebar = true;
    applySidebarState();
    showAiToast("✓ Sidebar compattata");
    clearAiInput();
    return;
  }
  if (lower.includes("espandi sidebar") || lower.includes("apri sidebar") || lower.includes("ingrandisci sidebar") || lower.includes("espandi menu") || lower.includes("apri menu")) {
    state.sidebarState = "expanded";
    state.userPinnedSidebar = true;
    applySidebarState();
    showAiToast("✓ Sidebar espansa");
    clearAiInput();
    return;
  }

  // 11. NAVIGAZIONE (OGGI / MESI)
  if (lower.includes("vai a oggi") || lower.includes("torna a oggi") || lower.includes("mostra oggi") || lower === "oggi") {
    navigateToDate(TODAY, false);
    showAiToast("✓ Navigato a Oggi");
    clearAiInput();
    return;
  }
  if (lower.includes("prossimo mese") || lower.includes("mese successivo") || lower.includes("vai avanti di un mese")) {
    stepMonth(1);
    showAiToast("✓ Mese successivo");
    clearAiInput();
    return;
  }
  if (lower.includes("mese precedente") || lower.includes("mese prima") || lower.includes("torna indietro di un mese")) {
    stepMonth(-1);
    showAiToast("✓ Mese precedente");
    clearAiInput();
    return;
  }

  const monthMatch = lower.match(/\b(?:vai\s+a|mostra|apri)?\s*(gennaio|febbraio|marzo|aprile|maggio|giugno|luglio|agosto|settembre|ottobre|novembre|dicembre)\b(?:\s+(202[6-9]))?/i);
  if (monthMatch && (lower.includes("vai") || lower.includes("mostra") || lower.includes("apri") || monthMatch[2])) {
    const monthsMap = {
      gennaio: 0, febbraio: 1, marzo: 2, aprile: 3, maggio: 4, giugno: 5,
      luglio: 6, agosto: 7, settembre: 8, ottobre: 9, novembre: 10, dicembre: 11
    };
    const m = monthsMap[monthMatch[1].toLowerCase()];
    const y = monthMatch[2] ? parseInt(monthMatch[2], 10) : (m <= 2 ? 2027 : 2026);
    navigateToMonth(y, m);
    showAiToast(`✓ Navigato a ${MONTH_NAMES[m]} ${y}`);
    clearAiInput();
    return;
  }

  // 12. TO-DO & COMPITI
  if (lower.includes("to-do") || lower.includes("todo") || lower.includes("compito") || lower.includes("promemoria") || lower.includes("da fare")) {
    const todoResult = parseTodoFromNaturalLanguage(text, session);
    if (todoResult) {
      store.addTodo(todoResult.date, todoResult.text);
      renderApp();
      updateTelemetry();
      const dObj = parseDateISO(todoResult.date);
      openDrawer(todoResult.date, dObj);
      showAiToast(`✓ To-do aggiunto al ${formatDateISOToIT(todoResult.date)}`);
      clearAiInput();
      return;
    }
  }

  // 13. DOMANDE & STATISTICHE SESSIONE
  if (lower.includes("prossimo esame") || lower.includes("quanti giorni mancano") || lower.includes("quando ho l'esame") || lower.includes("primo target")) {
    const upcoming = getUpcomingExamInfo(session);
    if (upcoming) {
      showAiClarification({
        prompt: `🎯 **Prossimo Target:** ${upcoming.courseName} (${upcoming.sigla})\n📅 **Data:** ${upcoming.dateFormatted} (tra **${upcoming.daysLeft} giorni**)`,
        type: "info",
        actionBtn: { label: "Visualizza nel Calendario", onClick: () => navigateToDate(upcoming.dateObj, true) }
      });
      showAiToast(`Prossimo: ${upcoming.sigla} tra ${upcoming.daysLeft} gg`);
    } else {
      showAiToast("Nessun appello futuro programmato");
    }
    clearAiInput();
    return;
  }

  if (lower.includes("quanti cfu") || lower.includes("totale cfu") || lower.includes("quanti esami")) {
    const coursesCount = Object.keys(session.courses || {}).length;
    const examsCount = (session.exams || []).length;
    let totalCfu = 0;
    Object.values(session.courses || {}).forEach(c => totalCfu += (c.cfu || 0));
    showAiClarification({
      prompt: `📊 **Riepilogo Sessione Attiva:**\n• **${coursesCount}** Corsi registrati\n• **${examsCount}** Date di appello\n• **${totalCfu}** CFU totali in programma`,
      type: "info"
    });
    showAiToast(`Sessione: ${coursesCount} corsi, ${totalCfu} CFU`);
    clearAiInput();
    return;
  }

  if (lower.includes("alert") || lower.includes("conflitti") || lower.includes("coincidenze") || lower.includes("errori")) {
    const btn = document.getElementById("btnToolbarAlert");
    btn?.click();
    showAiToast("✓ Centro Segnalazioni aperto");
    clearAiInput();
    return;
  }

  if (lower.includes("nuova sessione") || lower.includes("crea sessione")) {
    openNewSessionModal();
    showAiToast("✓ Modale Nuova Sessione aperta");
    clearAiInput();
    return;
  }

  if (lower.includes("gestisci sessioni") || lower.includes("elimina sessione") || lower.includes("elenco sessioni")) {
    openManageSessionsModal();
    showAiToast("✓ Gestione Sessioni aperta");
    clearAiInput();
    return;
  }

  // 14. INSERIMENTO EVENTO / ESAME
  const parsed = parseEventFromNaturalLanguage(rawText, session);

  // Check for critical missing parameters:
  // 1. Missing Course
  if (!parsed.courseId && parsed.isExam) {
    state.aiPendingEvent = parsed;
    showAiClarification({
      prompt: "Per quale materia desideri registrare l'esame?",
      type: "course",
      options: Object.values(session.courses).map(c => ({ id: c.id, label: `${c.sigla} (${c.name})` }))
    });
    return;
  }

  // 2. Missing Date
  if (!parsed.date) {
    state.aiPendingEvent = parsed;
    showAiClarification({
      prompt: "In quale data si terrà l'evento?",
      type: "date"
    });
    return;
  }

  commitAiEvent(parsed);
}

function commitAiEvent(eventData) {
  const session = store.getActiveSession();

  if (eventData.isExam) {
    if (eventData.isItinere && eventData.date2) {
      store.addItinereExams({
        courseId: eventData.courseId,
        date1: eventData.date,
        date2: eventData.date2,
        details: eventData.details || "Aggiunto via Assistente AI"
      });
    } else {
      store.addExam({
        courseId: eventData.courseId,
        date: eventData.date,
        round: eventData.round || 1,
        details: eventData.details || "Aggiunto via Assistente AI"
      });
    }
  } else {
    store.addGenericEvent({
      date: eventData.date,
      title: eventData.title || "Impegno di studio",
      time: eventData.time || "",
      category: eventData.category || "studio",
      details: eventData.details || "Aggiunto via Assistente AI"
    });
  }

  // Clear input & sheet
  const inputEl = document.getElementById("inputAiIslandText");
  if (inputEl) inputEl.value = "";
  closeAiClarifySheet();

  initHorizonRibbon();
  updateAlertsUI();
  updateTelemetry();
  renderApp();

  const dObj = parseDateISO(eventData.date);
  navigateToDate(dObj, true);
}

function showAiClarification(config) {
  const { prompt, type, options, actionBtn } = config;
  const sheet = document.getElementById("aiClarifySheet");
  const promptEl = document.getElementById("aiClarifyPrompt");
  const optionsEl = document.getElementById("aiClarifyOptions");
  if (!sheet || !promptEl || !optionsEl) return;

  promptEl.innerHTML = prompt.replace(/\n/g, "<br />").replace(/\*\*(.*?)\*\*/g, "<strong>$1</strong>");
  optionsEl.innerHTML = "";

  if (type === "course" && options) {
    options.forEach(opt => {
      const btn = document.createElement("button");
      btn.type = "button";
      btn.className = "ai-clarify-btn";
      btn.textContent = opt.label;
      btn.addEventListener("click", () => {
        if (state.aiPendingEvent) {
          state.aiPendingEvent.courseId = opt.id;
          handleAiInputSubmissionAfterClarify();
        }
      });
      optionsEl.appendChild(btn);
    });
  } else if (type === "date") {
    const dateInput = document.createElement("input");
    dateInput.type = "date";
    dateInput.className = "form-input";
    dateInput.value = state.activeDateString || formatDateISO(TODAY);

    const confirmBtn = document.createElement("button");
    confirmBtn.type = "button";
    confirmBtn.className = "btn-primary";
    confirmBtn.style.height = "34px";
    confirmBtn.textContent = "Conferma Data";

    confirmBtn.addEventListener("click", () => {
      if (state.aiPendingEvent && dateInput.value) {
        state.aiPendingEvent.date = dateInput.value;
        handleAiInputSubmissionAfterClarify();
      }
    });

    optionsEl.appendChild(dateInput);
    optionsEl.appendChild(confirmBtn);
  } else if (type === "info" && actionBtn) {
    const actBtn = document.createElement("button");
    actBtn.type = "button";
    actBtn.className = "btn-primary";
    actBtn.style.height = "34px";
    actBtn.textContent = actionBtn.label;
    actBtn.addEventListener("click", () => {
      closeAiClarifySheet();
      actionBtn.onClick?.();
    });
    optionsEl.appendChild(actBtn);
  }

  sheet.classList.remove("hidden");
  sheet.setAttribute("data-open", "true");
}

function handleAiInputSubmissionAfterClarify() {
  if (state.aiPendingEvent) {
    const copy = { ...state.aiPendingEvent };
    state.aiPendingEvent = null;
    commitAiEvent(copy);
  }
}

function closeAiClarifySheet() {
  const sheet = document.getElementById("aiClarifySheet");
  if (sheet) {
    sheet.classList.add("hidden");
    sheet.setAttribute("data-open", "false");
  }
}

// Local NLP Parsing with date/course heuristics
function parseEventFromNaturalLanguage(text, session) {
  const lower = text.toLowerCase();
  const res = {
    isExam: true,
    courseId: null,
    date: null,
    date2: null,
    round: 1,
    isItinere: false,
    title: text,
    details: text
  };

  // Check if generic event rather than exam
  if (lower.includes("lezione") || lower.includes("laboratorio")) {
    res.isExam = false;
    res.category = "lezione";
  } else if (lower.includes("scadenza") || lower.includes("consegna") || lower.includes("progetto")) {
    res.isExam = false;
    res.category = "scadenza";
  } else if (lower.includes("studio") || lower.includes("ripasso") || lower.includes("esercitazione")) {
    res.isExam = false;
    res.category = "studio";
  }

  // Detect Course
  const courses = Object.values(session.courses || {});
  for (const c of courses) {
    const sLow = c.sigla.toLowerCase();
    const nLow = c.name.toLowerCase();
    if (lower.includes(sLow) || lower.includes(nLow)) {
      res.courseId = c.id;
      break;
    }
  }

  // Course aliases (PoliMi context)
  if (!res.courseId) {
    if (lower.includes("meccanica") || lower.includes("mam") || lower.includes("rocchi")) res.courseId = "mam";
    else if (lower.includes("tecnologia") || lower.includes("tm")) res.courseId = "tm";
    else if (lower.includes("macchine") || lower.includes("sistemi energetici") || lower.includes("mse")) res.courseId = "mse";
    else if (lower.includes("dinamica") || lower.includes("controllo") || lower.includes("dcm")) res.courseId = "dcm";
    else if (lower.includes("progettazione") || lower.includes("psm")) res.courseId = "psm";
  }

  // Detect Round / Turno
  if (lower.includes("secondo") || lower.includes("2°") || lower.includes("2o") || lower.includes("recupero")) {
    res.round = 2;
  } else if (lower.includes("straordinario") || lower.includes("3°")) {
    res.round = 3;
  } else if (lower.includes("itinere") || lower.includes("parziale") || lower.includes("parziali")) {
    res.isItinere = true;
    res.round = "itinere_1";
  }

  // Detect Date (Explicit ISO or Italian Month Pattern: "15 gennaio", "8 febbraio 2027")
  const isoMatch = lower.match(/\b(202[6-9])-(\d{1,2})-(\d{1,2})\b/);
  if (isoMatch) {
    res.date = `${isoMatch[1]}-${String(isoMatch[2]).padStart(2, "0")}-${String(isoMatch[3]).padStart(2, "0")}`;
  } else {
    const monthsMap = {
      gennaio: "01", febbraio: "02", marzo: "03", aprile: "04", maggio: "05", giugno: "06",
      luglio: "07", agosto: "08", settembre: "09", ottobre: "10", novembre: "11", dicembre: "12",
      gen: "01", feb: "02", mar: "03", apr: "04", mag: "05", giu: "06",
      lug: "07", ago: "08", set: "09", ott: "10", nov: "11", dic: "12"
    };

    const dayMonthRegex = /\b(\d{1,2})\s+(gennaio|febbraio|marzo|aprile|maggio|giugno|luglio|agosto|settembre|ottobre|novembre|dicembre|gen|feb|mar|apr|mag|giu|lug|ago|set|ott|nov|dic)\b(?:\s+(202[6-9]))?/i;
    const dmMatch = lower.match(dayMonthRegex);
    if (dmMatch) {
      const day = String(dmMatch[1]).padStart(2, "0");
      const month = monthsMap[dmMatch[2].toLowerCase()];
      const year = dmMatch[3] || (parseInt(month, 10) <= 2 ? "2027" : "2026");
      res.date = `${year}-${month}-${day}`;
    }
  }

  return res;
}

// ==========================================================================
// 13. INFINITE SCROLL CALENDAR ENGINE
// ==========================================================================
function handleInfiniteScroll() {
  if (state.currentView !== "continuous" || isInfiniteAppending) return;

  const container = document.getElementById("calendarViewContainer");
  const scrollTop = container.scrollTop;
  const scrollHeight = container.scrollHeight;
  const clientHeight = container.clientHeight;

  if (scrollTop < 200) {
    isInfiniteAppending = true;
    prependEarlierMonths(2);
    setTimeout(() => { isInfiniteAppending = false; }, 150);
  } else if (scrollHeight - (scrollTop + clientHeight) < 300) {
    isInfiniteAppending = true;
    appendLaterMonths(2);
    setTimeout(() => { isInfiniteAppending = false; }, 150);
  }
}

function prependEarlierMonths(count = 2) {
  const stack = document.getElementById("monthsScrollStack");
  const container = document.getElementById("calendarViewContainer");
  const oldScrollHeight = container.scrollHeight;

  for (let i = 0; i < count; i++) {
    state.infiniteRange.startMonth--;
    if (state.infiniteRange.startMonth < 0) {
      state.infiniteRange.startMonth = 11;
      state.infiniteRange.startYear--;
    }
    const block = buildMonthSectionBlock(state.infiniteRange.startYear, state.infiniteRange.startMonth, "Calendario Continuo", true);
    stack.insertBefore(block, stack.firstChild);
    if (monthObserver) monthObserver.observe(block);
  }

  const newScrollHeight = container.scrollHeight;
  container.scrollTop += (newScrollHeight - oldScrollHeight);
}

function appendLaterMonths(count = 2) {
  const stack = document.getElementById("monthsScrollStack");

  for (let i = 0; i < count; i++) {
    state.infiniteRange.endMonth++;
    if (state.infiniteRange.endMonth > 11) {
      state.infiniteRange.endMonth = 0;
      state.infiniteRange.endYear++;
    }
    const block = buildMonthSectionBlock(state.infiniteRange.endYear, state.infiniteRange.endMonth, "Calendario Continuo", true);
    stack.appendChild(block);
    if (monthObserver) monthObserver.observe(block);
  }
}

function switchView(viewName) {
  if (state.currentView === viewName) return;
  state.currentView = viewName;

  document.querySelectorAll(".seg-btn[data-view]").forEach(btn => {
    const active = btn.dataset.view === viewName;
    btn.classList.toggle("active", active);
    btn.setAttribute("aria-selected", active ? "true" : "false");
  });

  const container = document.getElementById("calendarViewContainer");
  if (container) container.classList.add("view-switching");

  setTimeout(() => {
    const isMonthGrid = viewName === "continuous" || viewName === "month";
    document.getElementById("weekdaysBar").classList.toggle("hidden", !isMonthGrid);
    document.getElementById("monthsScrollStack").classList.toggle("hidden", !isMonthGrid);
    document.getElementById("weekView").classList.toggle("hidden", viewName !== "week");
    document.getElementById("agendaView").classList.toggle("hidden", viewName !== "agenda");

    renderApp();

    if (viewName === "continuous") {
      requestAnimationFrame(() => {
        scrollToMonthSection(state.currentYear, state.currentMonth, false);
      });
    }
    if (container) container.classList.remove("view-switching");
  }, 85);
}

function stepMonth(delta) {
  let y = state.currentYear;
  let m = state.currentMonth + delta;
  if (m < 0) { m = 11; y--; }
  if (m > 11) { m = 0; y++; }
  navigateToMonth(y, m);
}

function navigateToMonth(year, month) {
  state.currentYear = year;
  state.currentMonth = month;
  updateHeaderAndSidebarMonthUI();

  if (state.currentView === "continuous") {
    let section = document.getElementById(`month-sec-${year}-${month}`);
    if (!section) {
      state.infiniteRange.startYear = Math.min(state.infiniteRange.startYear, year);
      state.infiniteRange.startMonth = Math.min(state.infiniteRange.startMonth, month);
      state.infiniteRange.endYear = Math.max(state.infiniteRange.endYear, year);
      state.infiniteRange.endMonth = Math.max(state.infiniteRange.endMonth, month);
      renderContinuousMonthsView();
    }
    scrollToMonthSection(year, month, true);
  } else {
    renderApp();
  }
}

function navigateToDate(dateObj, shouldOpenDrawer = false) {
  const year = dateObj.getFullYear();
  const month = dateObj.getMonth();
  const dateStr = formatDateISO(dateObj);
  state.activeDateString = dateStr;
  state.currentYear = year;
  state.currentMonth = month;
  state.weekStart = new Date(dateObj);

  if (state.currentView === "continuous") {
    updateHeaderAndSidebarMonthUI();
    highlightSelectedCell(dateStr);
    let cell = document.querySelector(`.day-cell[data-date="${dateStr}"]`);
    if (!cell) {
      navigateToMonth(year, month);
      cell = document.querySelector(`.day-cell[data-date="${dateStr}"]`);
    }
    if (cell) {
      cell.scrollIntoView({ behavior: "smooth", block: "center" });
    }
  } else {
    renderApp();
  }

  if (shouldOpenDrawer) {
    openDrawer(dateStr, dateObj);
  }
}

function scrollToMonthSection(year, month, smooth = true) {
  const section = document.getElementById(`month-sec-${year}-${month}`);
  const container = document.getElementById("calendarViewContainer");
  if (section && container) {
    const topPos = section.offsetTop - 44;
    container.scrollTo({
      top: Math.max(0, topPos),
      behavior: smooth ? "smooth" : "auto"
    });
  }
}

function updateHeaderAndSidebarMonthUI() {
  document.getElementById("currentMonthDisplay").textContent =
    `${MONTH_NAMES[state.currentMonth]} ${state.currentYear}`;

  document.querySelectorAll(".month-pill").forEach(pill => {
    const y = parseInt(pill.dataset.year, 10);
    const m = parseInt(pill.dataset.month, 10);
    pill.classList.toggle("active", y === state.currentYear && m === state.currentMonth);
  });
}

function highlightSelectedCell(dateStr) {
  document.querySelectorAll(".day-cell.selected").forEach(el => el.classList.remove("selected"));
  document.querySelectorAll(`.day-cell[data-date="${dateStr}"]`).forEach(el => el.classList.add("selected"));
}

// ==========================================================================
// 14. VIEWS RENDERING & HOLIDAY/SUNDAY ENHANCEMENTS
// ==========================================================================
function getFilteredEvents() {
  const session = store.getActiveSession();
  const exams = session.exams || [];
  return exams.filter(evt => {
    // Multi-course filter check
    if (state.selectedCourseFilters.size > 0 && !state.selectedCourseFilters.has(evt.courseId)) {
      return false;
    }
    // Round filter check
    if (state.selectedRoundFilter !== "all" && String(evt.round) !== state.selectedRoundFilter) {
      return false;
    }
    return true;
  });
}

function getFilteredGenericEvents() {
  const session = store.getActiveSession();
  return session.genericEvents || [];
}

function renderApp() {
  updateHeaderAndSidebarMonthUI();

  if (state.currentView === "continuous") {
    renderContinuousMonthsView();
  } else if (state.currentView === "month") {
    renderSingleMonthView();
  } else if (state.currentView === "week") {
    renderWeekView();
  } else if (state.currentView === "agenda") {
    renderAgendaView();
  }
}

// Continuous Multi-Month Stack
function renderContinuousMonthsView() {
  const stack = document.getElementById("monthsScrollStack");
  stack.innerHTML = "";

  if (monthObserver) monthObserver.disconnect();

  const session = store.getActiveSession();

  let curY = state.infiniteRange.startYear;
  let curM = state.infiniteRange.startMonth;
  const endY = state.infiniteRange.endYear;
  const endM = state.infiniteRange.endMonth;

  while (curY < endY || (curY === endY && curM <= endM)) {
    const isSessionNominal = session.startDate && session.endDate &&
      (curY > parseDateISO(session.startDate).getFullYear() || (curY === parseDateISO(session.startDate).getFullYear() && curM >= parseDateISO(session.startDate).getMonth())) &&
      (curY < parseDateISO(session.endDate).getFullYear() || (curY === parseDateISO(session.endDate).getFullYear() && curM <= parseDateISO(session.endDate).getMonth()));

    const phaseLabel = isSessionNominal ? `${session.name} · Attiva` : "Calendario Infinito";
    const block = buildMonthSectionBlock(curY, curM, phaseLabel, true);
    stack.appendChild(block);

    curM++;
    if (curM > 11) {
      curM = 0;
      curY++;
    }
  }

  const container = document.getElementById("calendarViewContainer");
  monthObserver = new IntersectionObserver((entries) => {
    if (state.currentView !== "continuous") return;
    const visible = entries
      .filter(e => e.isIntersecting)
      .sort((a, b) => b.intersectionRatio - a.intersectionRatio);

    if (visible.length > 0) {
      const topEl = visible[0].target;
      const y = parseInt(topEl.dataset.year, 10);
      const m = parseInt(topEl.dataset.month, 10);
      if (state.currentYear !== y || state.currentMonth !== m) {
        state.currentYear = y;
        state.currentMonth = m;
        updateHeaderAndSidebarMonthUI();
      }
    }
  }, {
    root: container,
    threshold: [0.25, 0.55]
  });

  stack.querySelectorAll(".month-section-block").forEach(sec => monthObserver.observe(sec));
}

// Single Month View
function renderSingleMonthView() {
  if (monthObserver) monthObserver.disconnect();
  const stack = document.getElementById("monthsScrollStack");
  stack.innerHTML = "";

  const block = buildMonthSectionBlock(state.currentYear, state.currentMonth, "Scorrimento Mese Singolo", false);
  stack.appendChild(block);
}

function buildMonthSectionBlock(year, month, subtitleText, isContinuousMode) {
  const section = document.createElement("section");
  section.className = "month-section-block";
  section.id = `month-sec-${year}-${month}`;
  section.dataset.year = year;
  section.dataset.month = month;

  const monthEventsCount = getFilteredEvents().filter(e => {
    const d = parseDateISO(e.date);
    return d.getFullYear() === year && d.getMonth() === month;
  }).length;

  section.innerHTML = `
    <div class="month-section-header">
      <div class="month-section-title">
        <span>${MONTH_NAMES[month]} ${year}</span>
      </div>
      <div class="month-section-meta">
        ${subtitleText} ${monthEventsCount > 0 ? ` · <strong>${monthEventsCount} appell${monthEventsCount === 1 ? "o" : "i"}</strong>` : ""}
      </div>
    </div>
    <div class="days-grid"></div>
  `;

  const grid = section.querySelector(".days-grid");
  const firstDay = new Date(year, month, 1);
  const lastDay = new Date(year, month + 1, 0);

  const startDay = (state.settings && typeof state.settings.startDayOfWeek === "number") ? state.settings.startDayOfWeek : 1;
  const startOffset = (firstDay.getDay() - startDay + 7) % 7;

  const totalDays = lastDay.getDate();
  const prevMonthLastDate = new Date(year, month, 0).getDate();

  for (let i = startOffset - 1; i >= 0; i--) {
    const d = new Date(year, month - 1, prevMonthLastDate - i);
    grid.appendChild(createDayCell(d, true));
  }

  for (let day = 1; day <= totalDays; day++) {
    const d = new Date(year, month, day);
    grid.appendChild(createDayCell(d, false));
  }

  const totalCellsSoFar = startOffset + totalDays;
  const trailing = (7 - (totalCellsSoFar % 7)) % 7;
  for (let n = 1; n <= trailing; n++) {
    const d = new Date(year, month + 1, n);
    grid.appendChild(createDayCell(d, true));
  }

  return section;
}

function createDayCell(dateObj, isOtherMonth) {
  const session = store.getActiveSession();
  const dateStr = formatDateISO(dateObj);
  const cell = document.createElement("div");
  cell.className = `day-cell ${isOtherMonth ? "other-month" : ""}`;
  cell.dataset.date = dateStr;
  cell.setAttribute("role", "button");
  cell.setAttribute("tabindex", isOtherMonth ? "-1" : "0");

  const isToday = isSameDate(dateObj, TODAY);
  if (isToday) cell.classList.add("today");
  if (state.activeDateString === dateStr && !isOtherMonth) cell.classList.add("selected");

  // Differentiate weekly holiday day (configured in settings)
  const weeklyHoliday = (state.settings && typeof state.settings.weeklyHolidayDay === "number") ? state.settings.weeklyHolidayDay : 0;
  if (weeklyHoliday !== -1 && dateObj.getDay() === weeklyHoliday) {
    cell.classList.add("is-sunday");
  }

  // Check Italian Holiday
  const holiday = getItalianHoliday(dateObj);
  if (holiday) {
    cell.classList.add("holiday-day");
  }

  // To-Do Count
  const todos = store.getTodos(dateStr);
  const doneTodos = todos.filter(t => t.done).length;
  const hasSavedNote = Boolean((localStorage.getItem(`study_notes_${session.id}_${dateStr}`) || localStorage.getItem(`study_notes_${dateStr}`) || "").trim());

  const header = document.createElement("div");
  header.className = "day-header";
  header.innerHTML = `
    <span class="day-number">${dateObj.getDate()}</span>
    <div class="day-Indicators">
      ${isToday ? `<span class="today-tag">Oggi</span>` : ""}
      ${todos.length > 0 ? `<span class="todo-pill-badge" title="${doneTodos}/${todos.length} attività completate">✓ ${doneTodos}/${todos.length}</span>` : ""}
      ${hasSavedNote ? `<span class="note-dot" title="Nota personale presente"></span>` : ""}
    </div>
  `;
  cell.appendChild(header);

  // Holiday pill tag if applicable
  if (holiday) {
    const hEl = document.createElement("div");
    hEl.className = "holiday-pill mono";
    hEl.textContent = `🏛️ ${holiday}`;
    hEl.title = `Festività: ${holiday}`;
    cell.appendChild(hEl);
  }

  const eventsContainer = document.createElement("div");
  eventsContainer.className = "day-events";

  // Exam Badges
  const matchingEvents = getFilteredEvents().filter(e => e.date === dateStr);
  matchingEvents.forEach(evt => {
    const course = session.courses[evt.courseId];
    const badge = document.createElement("div");
    badge.className = `event-badge ${evt.courseId}`;
    if (course && course.colorVar) {
      badge.style.borderLeftColor = course.colorVar;
    }

    let roundLabel = `${evt.round}° App.`;
    let roundIcon = "🎯";
    if (evt.round === "itinere_1") {
      roundLabel = "1ª Itinere";
      roundIcon = "📝";
    } else if (evt.round === "itinere_2") {
      roundLabel = "2ª Itinere";
      roundIcon = "📝";
    } else if (evt.round === 2) {
      roundIcon = "🔄";
    }

    badge.innerHTML = `
      <span class="event-sigla-wrap">
        <span>${roundIcon}</span>
        <span>${course ? course.sigla : "ESAME"}</span>
      </span>
      <span class="event-round-tag mono">${roundLabel}</span>
    `;
    badge.title = `${course ? course.name : "Esame"} — ${roundLabel}`;
    eventsContainer.appendChild(badge);
  });

  // Generic Events Badges
  const matchingGeneric = getFilteredGenericEvents().filter(e => e.date === dateStr);
  matchingGeneric.forEach(gEvt => {
    const gBadge = document.createElement("div");
    gBadge.className = `generic-badge ${gEvt.category}`;
    gBadge.innerHTML = `
      <span style="overflow:hidden; text-overflow:ellipsis; white-space:nowrap;">${gEvt.title}</span>
      ${gEvt.time ? `<span class="mono" style="opacity:0.8; font-size:0.65rem;">${gEvt.time}</span>` : ""}
    `;
    gBadge.title = `${gEvt.title} (${gEvt.category}) ${gEvt.details ? `— ${gEvt.details}` : ""}`;
    eventsContainer.appendChild(gBadge);
  });

  cell.appendChild(eventsContainer);

  const labelParts = [`${dateObj.getDate()} ${MONTH_NAMES[dateObj.getMonth()]} ${dateObj.getFullYear()}`];
  if (holiday) labelParts.push(`Festività: ${holiday}`);
  if (matchingEvents.length > 0) labelParts.push(`${matchingEvents.length} esami previsti`);
  cell.setAttribute("aria-label", labelParts.join(", "));

  const activateCell = () => {
    state.activeDateString = dateStr;
    highlightSelectedCell(dateStr);
    openDrawer(dateStr, dateObj);
  };

  cell.addEventListener("click", activateCell);
  cell.addEventListener("keydown", (e) => {
    if (e.key === "Enter" || e.key === " ") {
      e.preventDefault();
      activateCell();
    }
  });

  return cell;
}

// Week View
function renderWeekView() {
  const container = document.getElementById("weekGrid");
  container.innerHTML = "";

  const session = store.getActiveSession();
  const d = new Date(state.weekStart);
  const startDay = (state.settings && typeof state.settings.startDayOfWeek === "number") ? state.settings.startDayOfWeek : 1;
  const weeklyHoliday = (state.settings && typeof state.settings.weeklyHolidayDay === "number") ? state.settings.weeklyHolidayDay : 0;

  const dayOfWeek = d.getDay();
  const diff = (dayOfWeek - startDay + 7) % 7;
  const weekStartDay = new Date(d);
  weekStartDay.setDate(d.getDate() - diff);
  const weekEndDay = new Date(weekStartDay);
  weekEndDay.setDate(weekStartDay.getDate() + 6);

  document.getElementById("weekRangeLabel").textContent =
    `${weekStartDay.getDate()} ${MONTH_NAMES[weekStartDay.getMonth()]} — ${weekEndDay.getDate()} ${MONTH_NAMES[weekEndDay.getMonth()]} ${weekEndDay.getFullYear()}`;

  for (let i = 0; i < 7; i++) {
    const currentDay = new Date(weekStartDay);
    currentDay.setDate(weekStartDay.getDate() + i);
    const dateStr = formatDateISO(currentDay);
    const isHoliday = (weeklyHoliday !== -1 && currentDay.getDay() === weeklyHoliday);
    const nameIdx = currentDay.getDay() === 0 ? 6 : currentDay.getDay() - 1;

    const col = document.createElement("div");
    col.className = `week-col ${isSameDate(currentDay, TODAY) ? "today" : ""} ${isHoliday ? "is-sunday" : ""}`;
    col.innerHTML = `
      <div class="week-col-header">
        <div class="week-col-name" style="${isHoliday ? 'color:var(--sunday-text); font-weight:700;' : ''}">${WEEKDAY_NAMES[nameIdx]}</div>
        <div class="week-col-date">${currentDay.getDate()}</div>
      </div>
      <div class="week-col-body"></div>
    `;

    const body = col.querySelector(".week-col-body");
    const matchingEvents = getFilteredEvents().filter(e => e.date === dateStr);
    const matchingGeneric = getFilteredGenericEvents().filter(e => e.date === dateStr);

    if (matchingEvents.length === 0 && matchingGeneric.length === 0) {
      body.innerHTML = `<span class="empty-state-text">Nessun impegno</span>`;
    } else {
      matchingEvents.forEach(evt => {
        const course = session.courses[evt.courseId];
        const badge = document.createElement("div");
        badge.className = `event-badge ${evt.courseId}`;
        badge.style.padding = "7px";
        if (course && course.colorVar) badge.style.borderLeftColor = course.colorVar;
        badge.innerHTML = `
          <div>
            <strong>${course ? course.sigla : "Esame"} · ${evt.round === 'itinere_1' ? '1ª Itinere' : (evt.round === 'itinere_2' ? '2ª Itinere' : `${evt.round}° App.`)}</strong>
            <div style="font-size:0.74rem; margin-top:2px;">${course ? course.name : ""}</div>
          </div>
        `;
        body.appendChild(badge);
      });

      matchingGeneric.forEach(gEvt => {
        const gBadge = document.createElement("div");
        gBadge.className = `generic-badge ${gEvt.category}`;
        gBadge.innerHTML = `<span>${gEvt.title}</span>`;
        body.appendChild(gBadge);
      });
    }

    col.addEventListener("click", () => openDrawer(dateStr, currentDay));
    container.appendChild(col);
  }
}

// Agenda View
function renderAgendaView() {
  const container = document.getElementById("agendaList");
  container.innerHTML = "";

  const session = store.getActiveSession();
  const events = getFilteredEvents().sort((a, b) => a.date.localeCompare(b.date));

  if (events.length === 0) {
    container.innerHTML = `
      <div class="agenda-group">
        <p class="empty-state-text">Nessun appello d'esame programmato per questa sessione con i filtri correnti.</p>
        <button type="button" class="btn-primary" style="margin-top:12px;" onclick="openAddExamModal()">+ Aggiungi il primo appello</button>
      </div>`;
    return;
  }

  const rounds = [
    { round: 1, title: "1° Giro di Appelli", subtitle: "Tentativi Principali" },
    { round: 2, title: "2° Giro di Appelli", subtitle: "Finestra di Recupero" },
    { round: "itinere_1", title: "Prove in Itinere (1° Parziale)", subtitle: "Valutazioni Intermedie" },
    { round: "itinere_2", title: "Prove in Itinere (2° Parziale)", subtitle: "Chiusura Prove Intermedie" },
    { round: 3, title: "Appelli Straordinari / Altri", subtitle: "Sessioni Speciali" }
  ];

  rounds.forEach(rGroup => {
    const groupEvts = events.filter(e => e.round === rGroup.round);
    if (groupEvts.length === 0) return;

    const groupEl = document.createElement("section");
    groupEl.className = "agenda-group";
    groupEl.innerHTML = `
      <div class="agenda-group-header">
        <h2>${rGroup.title}</h2>
        <span>${rGroup.subtitle}</span>
      </div>
      <div class="agenda-list"></div>
    `;

    const listEl = groupEl.querySelector(".agenda-list");
    let prevDate = null;

    groupEvts.forEach(evt => {
      const course = session.courses[evt.courseId];
      const evtDate = parseDateISO(evt.date);
      const diffDays = Math.ceil((evtDate - TODAY) / (1000 * 60 * 60 * 24));

      let deltaText = "Apertura giro";
      if (prevDate) {
        const deltaDays = Math.round((evtDate - prevDate) / (1000 * 60 * 60 * 24));
        deltaText = deltaDays === 0 ? "⚠️ Stesso giorno!" : `+${deltaDays} gg dal prec.`;
      }
      prevDate = evtDate;

      const card = document.createElement("div");
      card.className = "agenda-card";
      card.setAttribute("role", "button");
      card.setAttribute("tabindex", "0");
      card.innerHTML = `
        <div class="agenda-card-left">
          <div class="agenda-date-box">
            <div class="agenda-date-day">${String(evtDate.getDate()).padStart(2, "0")}</div>
            <div class="agenda-date-month">${MONTH_NAMES[evtDate.getMonth()].slice(0, 3)}</div>
          </div>
          <div class="agenda-info">
            <h3>
              <span class="course-swatch" style="background: ${course ? course.colorVar : 'var(--accent)'}"></span>
              <span>${course ? course.name : "Esame"}</span>
              <span class="event-round-tag mono">${evt.round === 'itinere_1' ? '1ª Itinere' : (evt.round === 'itinere_2' ? '2ª Itinere' : `${evt.round}° Appello`)} · ${course ? course.cfu : '6'} CFU</span>
            </h3>
            <p>${evt.details}</p>
          </div>
        </div>
        <div class="agenda-card-right">
          <span class="delta-badge">${deltaText}</span>
          <span class="countdown-pill">${diffDays >= 0 ? `Tra ${diffDays} gg` : "Concluso"}</span>
        </div>
      `;

      card.addEventListener("click", () => openDrawer(evt.date, evtDate));
      listEl.appendChild(card);
    });

    container.appendChild(groupEl);
  });
}

// ==========================================================================
// 15. INSPECTOR SHEET (To-Do List, Eventi Generici, Note)
// ==========================================================================
function openDrawer(dateStr, dateObj) {
  state.activeDateString = dateStr;
  const session = store.getActiveSession();

  const weekdayName = WEEKDAY_NAMES[dateObj.getDay() === 0 ? 6 : dateObj.getDay() - 1];
  document.getElementById("drawerDateSubtitle").textContent = weekdayName;
  document.getElementById("drawerDateTitle").textContent =
    `${dateObj.getDate()} ${MONTH_NAMES[dateObj.getMonth()]} ${dateObj.getFullYear()}`;

  // Holiday banner in drawer
  const holiday = getItalianHoliday(dateObj);
  const hBanner = document.getElementById("drawerHolidayNotice");
  if (hBanner) {
    if (holiday) {
      hBanner.textContent = `🏛️ Festività Nazionale: ${holiday}`;
      hBanner.classList.remove("hidden");
    } else {
      hBanner.classList.add("hidden");
    }
  }

  // To-Do List Render
  renderDrawerTodoList(dateStr);

  // Exam List Render
  const examContainer = document.getElementById("drawerExamList");
  const dayExams = (session.exams || []).filter(e => e.date === dateStr);

  if (dayExams.length === 0) {
    examContainer.innerHTML = `<p class="empty-state-text">Nessun appello d'esame in questa data.</p>`;
  } else {
    examContainer.innerHTML = "";
    dayExams.forEach(evt => {
      const course = session.courses[evt.courseId];
      const card = document.createElement("div");
      card.className = `drawer-exam-card ${evt.courseId}`;
      if (course && course.colorVar) card.style.borderLeftColor = course.colorVar;
      const roundLabel = evt.round === 'itinere_1' ? '1ª Prova in Itinere' : (evt.round === 'itinere_2' ? '2ª Prova in Itinere' : `${evt.round}° Appello`);

      card.innerHTML = `
        <div class="drawer-exam-header">
          <div class="drawer-exam-title">${course ? course.name : "Esame"}</div>
          <div style="display:flex; align-items:center; gap:6px;">
            <span class="event-round-tag mono">${roundLabel}</span>
            <button type="button" class="btn-text-small" style="color:var(--danger-text);" data-action="delete-exam" data-id="${evt.id}" title="Rimuovi questo appello">✕</button>
          </div>
        </div>
        <div class="drawer-exam-details">
          <div><strong>Codice:</strong> <span class="mono">${course ? course.code : "N/A"}</span> (${course ? course.cfu : "6"} CFU)</div>
          <div><strong>Docenti:</strong> ${course ? course.prof : "N/A"}</div>
          <div style="margin-top:4px;">${evt.details}</div>
        </div>
      `;

      card.querySelector('[data-action="delete-exam"]').addEventListener("click", (e) => {
        e.stopPropagation();
        if (confirm("Rimuovere questo appello d'esame dalla sessione?")) {
          store.deleteExam(evt.id);
          initHorizonRibbon();
          updateAlertsUI();
          updateTelemetry();
          renderApp();
          openDrawer(dateStr, dateObj);
        }
      });

      examContainer.appendChild(card);
    });
  }

  // Generic Events List Render
  const genericContainer = document.getElementById("drawerGenericEventsList");
  const dayGeneric = (session.genericEvents || []).filter(e => e.date === dateStr);

  if (!dayGeneric || dayGeneric.length === 0) {
    genericContainer.innerHTML = `<p class="empty-state-text">Nessun altro impegno registrato.</p>`;
  } else {
    genericContainer.innerHTML = "";
    dayGeneric.forEach(gEvt => {
      const card = document.createElement("div");
      card.className = `generic-badge ${gEvt.category}`;
      card.style.cssText = "padding:8px 10px; margin-bottom:6px; display:flex; justify-content:space-between; align-items:center;";
      card.innerHTML = `
        <div>
          <div style="font-weight:700;">${gEvt.title} ${gEvt.time ? `<span class="mono" style="font-weight:normal; font-size:0.75rem;">(${gEvt.time})</span>` : ""}</div>
          ${gEvt.details ? `<div style="font-size:0.75rem; color:var(--text-secondary);">${gEvt.details}</div>` : ""}
        </div>
        <button type="button" class="btn-text-small" style="color:var(--danger-text);" data-action="delete-genevt" data-id="${gEvt.id}">✕</button>
      `;

      card.querySelector('[data-action="delete-genevt"]').addEventListener("click", (e) => {
        e.stopPropagation();
        store.deleteGenericEvent(gEvt.id);
        openDrawer(dateStr, dateObj);
        renderApp();
      });

      genericContainer.appendChild(card);
    });
  }

  // Modules preview
  const modulesContainer = document.getElementById("drawerCourseModules");
  const hasModules = Object.values(session.courses).find(c => c.modules && c.modules.length > 0);
  if (hasModules) {
    modulesContainer.innerHTML = hasModules.modules
      .map(m => `<div class="module-pill">${m}</div>`)
      .join("");
  } else {
    modulesContainer.innerHTML = `<div class="empty-state-text">Pronto per l'aggiunta delle unità didattiche e argomenti.</div>`;
  }

  // Notes
  const savedNotes = localStorage.getItem(`study_notes_${session.id}_${dateStr}`) ||
                     localStorage.getItem(`study_notes_${dateStr}`) || "";
  document.getElementById("drawerUserNotes").value = savedNotes;
  document.getElementById("noteSavedStatus").textContent = "";

  document.getElementById("drawerOverlay").classList.add("open");
  document.getElementById("dayDrawer").classList.add("open");
}

function renderDrawerTodoList(dateStr) {
  const container = document.getElementById("drawerTodoList");
  const progressLabel = document.getElementById("drawerTodoProgress");
  if (!container) return;

  const todos = store.getTodos(dateStr);
  const doneCount = todos.filter(t => t.done).length;

  if (progressLabel) {
    progressLabel.textContent = `${doneCount}/${todos.length}`;
  }

  container.innerHTML = "";
  if (todos.length === 0) {
    container.innerHTML = `<p class="empty-state-text">Nessun task per oggi. Aggiungi il primo!</p>`;
    return;
  }

  todos.forEach(t => {
    const item = document.createElement("div");
    item.className = `todo-item ${t.done ? "done" : ""}`;
    item.innerHTML = `
      <div class="todo-checkbox" data-action="toggle"></div>
      <span class="todo-text" data-action="toggle">${t.text}</span>
      <button type="button" class="todo-delete-btn" data-action="delete" title="Elimina task">✕</button>
    `;

    item.querySelectorAll('[data-action="toggle"]').forEach(el => {
      el.addEventListener("click", () => {
        store.toggleTodo(dateStr, t.id);
        renderDrawerTodoList(dateStr);
        renderApp();
      });
    });

    item.querySelector('[data-action="delete"]').addEventListener("click", () => {
      store.deleteTodo(dateStr, t.id);
      renderDrawerTodoList(dateStr);
      renderApp();
    });

    container.appendChild(item);
  });
}

function closeDrawer() {
  document.getElementById("drawerOverlay").classList.remove("open");
  document.getElementById("dayDrawer").classList.remove("open");
}

function saveUserNotes() {
  if (!state.activeDateString) return;
  const session = store.getActiveSession();
  const text = document.getElementById("drawerUserNotes").value;
  localStorage.setItem(`study_notes_${session.id}_${state.activeDateString}`, text);

  const statusEl = document.getElementById("noteSavedStatus");
  statusEl.textContent = "✓ Nota salvata";
  renderApp();
  setTimeout(() => { statusEl.textContent = ""; }, 2200);
}

// ==========================================================================
// 16. DATE FORMATTING HELPERS
// ==========================================================================
function formatDateISO(d) {
  const year = d.getFullYear();
  const month = String(d.getMonth() + 1).padStart(2, "0");
  const day = String(d.getDate()).padStart(2, "0");
  return `${year}-${month}-${day}`;
}

function parseDateISO(str) {
  if (!str) return new Date();
  const [y, m, d] = str.split("-").map(Number);
  return new Date(y, m - 1, d);
}

function isSameDate(d1, d2) {
  return (
    d1.getFullYear() === d2.getFullYear() &&
    d1.getMonth() === d2.getMonth() &&
    d1.getDate() === d2.getDate()
  );
}

function formatDateFriendly(d) {
  const wd = WEEKDAY_NAMES[d.getDay() === 0 ? 6 : d.getDay() - 1].slice(0, 3);
  return `${wd} ${String(d.getDate()).padStart(2, "0")} ${MONTH_NAMES[d.getMonth()].slice(0, 3)} ${d.getFullYear()}`;
}

function formatDateISOToIT(isoStr) {
  const [y, m, d] = isoStr.split("-");
  return `${d}/${m}/${y}`;
}
