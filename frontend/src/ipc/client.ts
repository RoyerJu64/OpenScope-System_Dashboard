import { invoke } from "@tauri-apps/api/core";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import type {
  ActionOutcome,
  BatchDto,
  Capabilities,
  ProcessRowDto,
} from "./types";

export function onMetricsBatch(
  handler: (batch: BatchDto) => void,
): Promise<UnlistenFn> {
  return listen<BatchDto>("metrics-batch", (event) => handler(event.payload));
}

export function getCapabilities(): Promise<Capabilities> {
  return invoke<Capabilities>("get_capabilities");
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
