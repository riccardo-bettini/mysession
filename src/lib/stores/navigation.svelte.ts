import type { PendingPlanNotification, SectionId } from "../types/domain";
import { getTodayIso } from "../utils/calendar";

class NavigationStore {
  activeSection = $state<SectionId>("home");
  isMorphing = $state<boolean>(false);

  // Stato sub-viste contestuali
  selectedCourseId = $state<string | null>(null);
  todayTargetDate = $state<string>(getTodayIso());
  calendarSelectedDate = $state<string>(getTodayIso());
  activeQuizId = $state<string | null>(null);

  // Overlays / Modali
  isSettingsOpen = $state<boolean>(false);
  settingsInitialTab = $state<"appearance" | "sidebar" | "sessions" | "ai">("appearance");
  isSessionPopoverOpen = $state<boolean>(false);
  isTimetableModalOpen = $state<boolean>(false);
  isNewSessionModalOpen = $state<boolean>(false);
  pendingPlanModal = $state<PendingPlanNotification | null>(null);

  toast = $state<{ message: string; tone: "info" | "success" | "error" } | null>(null);

  private morphTimer: number | null = null;
  private toastTimer: number | null = null;

  navigate(section: SectionId, options?: { courseId?: string | null; date?: string; skipMorph?: boolean }): void {
    if (options?.date) {
      this.todayTargetDate = options.date;
      this.calendarSelectedDate = options.date;
    }
    if (options?.courseId !== undefined) {
      this.selectedCourseId = options.courseId;
    } else if (section !== "courses") {
      this.selectedCourseId = null;
    }

    this.isSessionPopoverOpen = false;

    if (this.activeSection === section && !options?.skipMorph) {
      return;
    }

    if (this.morphTimer !== null && typeof window !== "undefined") {
      window.clearTimeout(this.morphTimer);
      this.morphTimer = null;
    }

    const reducedMotion =
      typeof window !== "undefined" &&
      window.matchMedia?.("(prefers-reduced-motion: reduce)").matches;

    if (reducedMotion || options?.skipMorph) {
      this.activeSection = section;
      this.isMorphing = false;
      return;
    }

    this.activeSection = section;
    this.isMorphing = true;
    this.morphTimer = window.setTimeout(() => {
      this.isMorphing = false;
      this.morphTimer = null;
    }, 150);
  }

  openCourseDetail(courseId: string): void {
    this.selectedCourseId = courseId;
    this.navigate("courses", { courseId, skipMorph: this.activeSection === "courses" });
  }

  openDayInTodayView(isoDate: string): void {
    this.todayTargetDate = isoDate;
    this.navigate("today", { date: isoDate });
  }

  openSettings(tab: "appearance" | "sidebar" | "sessions" | "ai" = "appearance"): void {
    this.settingsInitialTab = tab;
    this.isSettingsOpen = true;
    this.isSessionPopoverOpen = false;
  }

  showToast(message: string, tone: "info" | "success" | "error" = "info"): void {
    this.toast = { message, tone };
    if (this.toastTimer !== null && typeof window !== "undefined") {
      window.clearTimeout(this.toastTimer);
    }
    if (typeof window !== "undefined") {
      this.toastTimer = window.setTimeout(() => {
        this.toast = null;
      }, 3600);
    }
  }
}

export const navStore = new NavigationStore();
