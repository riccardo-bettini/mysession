import type { AppSettings, TaskCategory } from "../types/domain";

export interface CategoryMeta {
  id: TaskCategory;
  label: string;
  shortLabel: string;
  defaultColor: string;
}

export const TASK_CATEGORIES: readonly CategoryMeta[] = [
  { id: "teoria", label: "Teoria", shortLabel: "Teoria", defaultColor: "#3B82F6" },
  { id: "esercizi", label: "Esercizi", shortLabel: "Esercizi", defaultColor: "#10B981" },
  { id: "temi_esame", label: "Temi d'esame", shortLabel: "Temi d'esame", defaultColor: "#F59E0B" },
  { id: "dimostrazioni", label: "Dimostrazioni", shortLabel: "Dimostr.", defaultColor: "#8B5CF6" },
  { id: "domande_orale", label: "Domande orale", shortLabel: "Orale", defaultColor: "#EC4899" },
  { id: "ripasso", label: "Ripasso", shortLabel: "Ripasso", defaultColor: "#6366F1" },
] as const;

export const CORE_FIVE_CATEGORIES: readonly CategoryMeta[] = TASK_CATEGORIES.slice(0, 5);

export function getCategoryLabel(category: string): string {
  return TASK_CATEGORIES.find((c) => c.id === category)?.label ?? "Teoria";
}

export function getCategoryColor(category: string, settings: AppSettings): string {
  switch (category) {
    case "teoria":
      return settings.catTeoriaColor || "#3B82F6";
    case "esercizi":
      return settings.catEserciziColor || "#10B981";
    case "temi_esame":
      return settings.catTemiEsameColor || "#F59E0B";
    case "dimostrazioni":
      return settings.catDimostrazioniColor || "#8B5CF6";
    case "domande_orale":
      return settings.catDomandeOraleColor || "#EC4899";
    default:
      return settings.accentColor || "#6366F1";
  }
}

export const SECTION_LABELS: Record<string, string> = {
  home: "Home",
  calendar: "Calendario",
  today: "Oggi",
  courses: "Corsi",
  quiz: "Quiz",
  progress: "Progressione",
  ai: "AI",
};
