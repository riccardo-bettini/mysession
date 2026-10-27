/**
 * Apple HIG Fluid Interface Spring Physics Engine (Svelte 5 Runes)
 * Implementa il modello a due parametri di Apple WWDC ("Designing Fluid Interfaces"):
 * - dampingRatio: 1.0 = smorzamento critico (nessun rimbalzo), < 1.0 = sottocritico (es. 0.8)
 * - response: velocità di risposta in secondi (es. 0.30s – 0.40s)
 * Supporta interruzione mid-flight con conservazione della velocità istantanea,
 * proiezione del momento (projectMomentum) e resistenza progressiva ai bordi (rubberband).
 */

export interface AppleSpringConfig {
  dampingRatio?: number;
  response?: number;
  precision?: number;
}

export const SPRING_PRESETS = {
  /** Default UI: smorzamento critico, zero overshoot (WWDC default) */
  default: { dampingRatio: 1.0, response: 0.34, precision: 0.001 },
  /** Drawer / Sheet / Pannelli laterali: lieve elasticità fisica */
  sheet: { dampingRatio: 0.84, response: 0.32, precision: 0.001 },
  /** Interazioni con rilascio di velocità / flick */
  momentum: { dampingRatio: 0.78, response: 0.38, precision: 0.001 },
  /** Micro-feedback rapido per controlli compatti */
  snappy: { dampingRatio: 0.92, response: 0.22, precision: 0.001 }
} as const;

/**
 * Proiezione esatta del punto di arresto usata da Apple in UIScrollView / Fluid Interfaces:
 * decelerationRate ≈ 0.998 (normale) oppure 0.990 (rapido)
 */
export function projectMomentum(initialVelocityPxPerSec: number, decelerationRate = 0.998): number {
  return ((initialVelocityPxPerSec / 1000) * decelerationRate) / (1 - decelerationRate);
}

/**
 * Formula Apple di rubber-banding per resistenza progressiva oltre i limiti (costante 0.55)
 */
export function rubberband(overshoot: number, dimension: number, constant = 0.55): number {
  if (dimension <= 0) return 0;
  return (overshoot * dimension * constant) / (dimension + constant * Math.abs(overshoot));
}

export class SpringScalar {
  current = $state<number>(0);
  target = $state<number>(0);
  velocity = $state<number>(0);
  isAnimating = $state<boolean>(false);

  private dampingRatio: number;
  private response: number;
  private precision: number;
  private rafId: number | null = null;
  private lastTimestamp: number | null = null;

  constructor(initialValue = 0, config: AppleSpringConfig = SPRING_PRESETS.default) {
    this.current = initialValue;
    this.target = initialValue;
    this.dampingRatio = config.dampingRatio ?? 1.0;
    this.response = Math.max(0.05, config.response ?? 0.34);
    this.precision = config.precision ?? 0.001;
  }

  /**
   * Imposta un nuovo target mantenendo la posizione e la velocità correnti (interrompibilità pura)
   */
  setTarget(newTarget: number, initialVelocityDelta = 0, config?: AppleSpringConfig): void {
    if (config?.dampingRatio !== undefined) this.dampingRatio = config.dampingRatio;
    if (config?.response !== undefined) this.response = Math.max(0.05, config.response);

    this.target = newTarget;
    if (initialVelocityDelta !== 0) {
      this.velocity += initialVelocityDelta;
    }

    if (
      typeof window !== "undefined" &&
      window.matchMedia?.("(prefers-reduced-motion: reduce)").matches
    ) {
      this.snap(newTarget);
      return;
    }

    if (!this.isAnimating) {
      this.isAnimating = true;
      this.lastTimestamp = null;
      this.scheduleFrame();
    }
  }

  /**
   * Posiziona istantaneamente il valore (es. durante tracking 1:1 del puntatore)
   */
  snap(value: number, velocity = 0): void {
    if (this.rafId !== null && typeof cancelAnimationFrame !== "undefined") {
      cancelAnimationFrame(this.rafId);
      this.rafId = null;
    }
    this.current = value;
    this.target = value;
    this.velocity = velocity;
    this.isAnimating = false;
    this.lastTimestamp = null;
  }

  private scheduleFrame(): void {
    if (typeof requestAnimationFrame === "undefined") {
      this.snap(this.target);
      return;
    }
    this.rafId = requestAnimationFrame((now) => this.step(now));
  }

  private step(now: number): void {
    if (this.lastTimestamp === null) {
      this.lastTimestamp = now;
      this.scheduleFrame();
      return;
    }

    // Supporta nativamente 60Hz, 120Hz, 180Hz+ con clamping di sicurezza per tab in background
    const dt = Math.min(0.032, Math.max(0.001, (now - this.lastTimestamp) / 1000));
    this.lastTimestamp = now;

    // Conversione parametri Apple (dampingRatio, response) -> (stiffness k, damping c, mass = 1)
    const omega = (2 * Math.PI) / this.response;
    const k = omega * omega;
    const c = 2 * this.dampingRatio * omega;

    const displacement = this.current - this.target;
    const springForce = -k * displacement;
    const dampingForce = -c * this.velocity;
    const acceleration = springForce + dampingForce;

    this.velocity += acceleration * dt;
    this.current += this.velocity * dt;

    if (
      Math.abs(this.target - this.current) <= this.precision &&
      Math.abs(this.velocity) <= this.precision * 10
    ) {
      this.snap(this.target, 0);
      return;
    }

    this.scheduleFrame();
  }
}
