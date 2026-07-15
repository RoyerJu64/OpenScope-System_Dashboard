import { invoke } from "@tauri-apps/api/core";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import type {
  ActionOutcome,
  BatchDto,
  Capabilities,
  HotSeriesDto,
  ProcessRowDto,
} from "./types";

/**
 * Le backend coalesce les ticks par fenêtres de 100 ms : chaque event
 * porte un tableau de batches (issue #21).
 */
export function onMetricsBatch(
  handler: (batches: BatchDto[]) => void,
): Promise<UnlistenFn> {
  return listen<BatchDto[]>("metrics-batch", (event) => handler(event.payload));
}

export function getCapabilities(): Promise<Capabilities> {
  return invoke<Capabilities>("get_capabilities");
}

/** Fenêtre chaude (~10 min) des métriques demandées, tous labels confondus. */
export function getHotWindow(metrics: string[]): Promise<HotSeriesDto[]> {
  return invoke<HotSeriesDto[]>("get_hot_window", { metrics });
}

export function listProcesses(): Promise<ProcessRowDto[]> {
  return invoke<ProcessRowDto[]>("list_processes");
}

/** signal : SIGTERM=15 (défaut), SIGKILL=9. */
export function killProcess(
  pid: number,
  signal?: number,
): Promise<ActionOutcome> {
  return invoke<ActionOutcome>("kill_process", { pid, signal });
}

export function setPriority(
  pid: number,
  nice: number,
): Promise<ActionOutcome> {
  return invoke<ActionOutcome>("set_priority", { pid, nice });
}

/** Disposition sauvegardée d'une page (schéma possédé par le frontend). */
export function getLayout(page: string): Promise<unknown | null> {
  return invoke<unknown | null>("get_layout", { page });
}

export function saveLayout(page: string, layout: unknown): Promise<void> {
  return invoke<void>("save_layout", { page, layout });
}
