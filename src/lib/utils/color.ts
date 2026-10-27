/**
 * Sistema Unificato di Gestione Colori & Contrasto Dinamico (WCAG AA/AAA)
 * Condiviso da tutti i selettori colore, card corsi, pillole appelli e categorie.
 */

export const UNIFIED_STANDARD_PALETTE: readonly string[] = [
  "#007AFF",
  "#0284C7",
  "#059669",
  "#34C759",
  "#EAB308",
  "#FF9500",
  "#FF3B30",
  "#E11D48",
  "#AF52DE",
  "#7C3AED",
  "#5856D6",
  "#64748B"
] as const;

export interface RgbColor {
  r: number;
  g: number;
  b: number;
}

export interface HslColor {
  h: number; // 0..360
  s: number; // 0..100
  l: number; // 0..100
}

export interface CourseColorTokens {
  solid: string;
  bgTint: string;
  bgHoverTint: string;
  borderTint: string;
  textContrast: string;
  badgeText: string;
  contrastRatio: number;
}

export function normalizeHex(hex: string, fallback = "#007AFF"): string {
  const clean = (hex || "").trim().replace(/^#/, "");
  if (/^[0-9a-fA-F]{3}$/.test(clean)) {
    const r = clean[0] + clean[0];
    const g = clean[1] + clean[1];
    const b = clean[2] + clean[2];
    return `#${(r + g + b).toUpperCase()}`;
  }
  if (/^[0-9a-fA-F]{6}$/.test(clean)) {
    return `#${clean.toUpperCase()}`;
  }
  return fallback;
}

export function hexToRgb(hex: string): RgbColor {
  const norm = normalizeHex(hex).slice(1);
  return {
    r: parseInt(norm.slice(0, 2), 16),
    g: parseInt(norm.slice(2, 4), 16),
    b: parseInt(norm.slice(4, 6), 16)
  };
}

export function rgbToHex({ r, g, b }: RgbColor): string {
  const toByte = (v: number) => Math.max(0, Math.min(255, Math.round(v))).toString(16).padStart(2, "0");
  return `#${toByte(r)}${toByte(g)}${toByte(b)}`.toUpperCase();
}

export function rgbToHsl({ r, g, b }: RgbColor): HslColor {
  const rn = r / 255;
  const gn = g / 255;
  const bn = b / 255;
  const max = Math.max(rn, gn, bn);
  const min = Math.min(rn, gn, bn);
  let h = 0;
  let s = 0;
  const l = (max + min) / 2;

  if (max !== min) {
    const d = max - min;
    s = l > 0.5 ? d / (2 - max - min) : d / (max + min);
    switch (max) {
      case rn:
        h = (gn - bn) / d + (gn < bn ? 6 : 0);
        break;
      case gn:
        h = (bn - rn) / d + 2;
        break;
      case bn:
        h = (rn - gn) / d + 4;
        break;
    }
    h /= 6;
  }

  return {
    h: Math.round(h * 360),
    s: Math.round(s * 100),
    l: Math.round(l * 100)
  };
}

export function hslToRgb({ h, s, l }: HslColor): RgbColor {
  const hn = ((h % 360) + 360) % 360 / 360;
  const sn = Math.max(0, Math.min(100, s)) / 100;
  const ln = Math.max(0, Math.min(100, l)) / 100;

  if (sn === 0) {
    const gray = Math.round(ln * 255);
    return { r: gray, g: gray, b: gray };
  }

  const hue2rgb = (p: number, q: number, t: number) => {
    let tn = t;
    if (tn < 0) tn += 1;
    if (tn > 1) tn -= 1;
    if (tn < 1 / 6) return p + (q - p) * 6 * tn;
    if (tn < 1 / 2) return q;
    if (tn < 2 / 3) return p + (q - p) * (2 / 3 - tn) * 6;
    return p;
  };

  const q = ln < 0.5 ? ln * (1 + sn) : ln + sn - ln * sn;
  const p = 2 * ln - q;

  return {
    r: Math.round(hue2rgb(p, q, hn + 1 / 3) * 255),
    g: Math.round(hue2rgb(p, q, hn) * 255),
    b: Math.round(hue2rgb(p, q, hn - 1 / 3) * 255)
  };
}

export function hexToHsl(hex: string): HslColor {
  return rgbToHsl(hexToRgb(hex));
}

export function hslToHex(hsl: HslColor): string {
  return rgbToHex(hslToRgb(hsl));
}

export function hueToHex(hue: number, saturation = 82, lightness = 52): string {
  return hslToHex({ h: hue, s: saturation, l: lightness });
}

/**
 * Calcolo luminanza relativa WCAG 2.1
 */
export function getRelativeLuminance({ r, g, b }: RgbColor): number {
  const transform = (c: number) => {
    const srgb = c / 255;
    return srgb <= 0.03928 ? srgb / 12.92 : Math.pow((srgb + 0.055) / 1.055, 2.4);
  };
  return 0.2126 * transform(r) + 0.7152 * transform(g) + 0.0722 * transform(b);
}

/**
 * Rapporto di contrasto WCAG 2.1 tra due colori (1.0 .. 21.0)
 */
export function getContrastRatio(rgb1: RgbColor, rgb2: RgbColor): number {
  const l1 = getRelativeLuminance(rgb1);
  const l2 = getRelativeLuminance(rgb2);
  const lighter = Math.max(l1, l2);
  const darker = Math.min(l1, l2);
  return (lighter + 0.05) / (darker + 0.05);
}

/**
 * Genera automaticamente tinte pastello/vibranti ad alto contrasto (WCAG AA/AAA)
 * ottimizzate per il tema attivo (Dark o Light).
 */
export function generateCourseColorTokens(baseHex: string, isDark: boolean): CourseColorTokens {
  const norm = normalizeHex(baseHex);
  const rgb = hexToRgb(norm);
  const hsl = rgbToHsl(rgb);

  // Calcola il colore del testo ad alto contrasto (AA/AAA) rispetto allo sfondo della pillola
  const surfaceRef: RgbColor = isDark ? { r: 28, g: 28, b: 36 } : { r: 248, g: 249, b: 252 };

  const targetLightness = isDark
    ? Math.max(74, Math.min(90, hsl.l + 28))
    : Math.max(22, Math.min(36, hsl.l - 24));

  const targetSat = isDark ? Math.max(68, Math.min(95, hsl.s + 8)) : Math.max(65, Math.min(92, hsl.s));

  const textRgb = hslToRgb({ h: hsl.h, s: targetSat, l: targetLightness });
  const textContrast = rgbToHex(textRgb);
  const contrastRatio = getContrastRatio(textRgb, surfaceRef);

  const whiteContrast = getContrastRatio({ r: 255, g: 255, b: 255 }, rgb);
  const badgeText = whiteContrast >= 4.2 ? "#FFFFFF" : "#111827";

  return {
    solid: norm,
    bgTint: isDark
      ? `rgba(${rgb.r}, ${rgb.g}, ${rgb.b}, 0.19)`
      : `rgba(${rgb.r}, ${rgb.g}, ${rgb.b}, 0.13)`,
    bgHoverTint: isDark
      ? `rgba(${rgb.r}, ${rgb.g}, ${rgb.b}, 0.28)`
      : `rgba(${rgb.r}, ${rgb.g}, ${rgb.b}, 0.21)`,
    borderTint: isDark
      ? `rgba(${rgb.r}, ${rgb.g}, ${rgb.b}, 0.46)`
      : `rgba(${rgb.r}, ${rgb.g}, ${rgb.b}, 0.36)`,
    textContrast,
    badgeText,
    contrastRatio: Math.round(contrastRatio * 100) / 100
  };
}
