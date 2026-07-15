import { createMemo, For } from "solid-js";
import { labelledKeys, lastValue, version } from "../stores/metrics";

/**
 * Barres d'utilisation instantanée par cœur, façon btop. Les cœurs sont
 * découverts dynamiquement depuis les séries `cpu.usage{core=N}`.
 */
export function CpuCores() {
  const cores = createMemo(() => {
    version();
    return labelledKeys("cpu.usage", "core").map((c) => ({
      label: c.label,
      value: lastValue(c.key),
    }));
  });

  return (
    <div class="core-grid">
      <For each={cores()}>
        {(core) => (
          <div class="core-row">
            <span class="core-label">{core.label}</span>
            <div class="core-track">
              <div
                class="core-fill"
                style={{ width: `${core.value ?? 0}%` }}
              />
            </div>
            <span class="core-value">
              {core.value == null ? "—" : `${core.value.toFixed(0)} %`}
            </span>
          </div>
        )}
      </For>
    </div>
  );
}
