import { invoke } from "@tauri-apps/api/core";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import type { BatchDto, Capabilities } from "./types";

export function onMetricsBatch(
  handler: (batch: BatchDto) => void,
): Promise<UnlistenFn> {
  return listen<BatchDto>("metrics-batch", (event) => handler(event.payload));
}

export function getCapabilities(): Promise<Capabilities> {
  return invoke<Capabilities>("get_capabilities");
}
