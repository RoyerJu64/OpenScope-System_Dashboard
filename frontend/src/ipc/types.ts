/*
 * DTOs de la frontière IPC. Écrits à la main en Phase 0 ;
 * seront générés depuis les types Rust via ts-rs (issue #6).
 */

export type ValueDto =
  | { kind: "gauge"; v: number }
  | { kind: "counter"; v: number }
  | { kind: "text"; v: string };

export interface SampleDto {
  metric: string;
  value: ValueDto;
  labels?: Record<string, string>;
}

/** Payload de l'event `metrics-batch` : un tick d'un collecteur. */
export interface BatchDto {
  source: string;
  ts_ms: number;
  samples: SampleDto[];
}

export interface CollectorCapabilityDto {
  available: boolean;
  reason?: string;
  details?: Record<string, string>;
}

export interface CapabilitiesDto {
  collectors: Record<string, CollectorCapabilityDto>;
}
