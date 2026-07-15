import { createMemo } from "solid-js";
import { labelledKeys, lastValue, version } from "../stores/metrics";
import { pct } from "../utils/format";

export function CpuInstant() {
  const now = createMemo(() => {
    version();
    return lastValue("cpu.usage");
  });

  // Détails optionnels selon le matériel (température, fréquence
  // moyenne, consommation RAPL) — absents, ils ne s'affichent pas.
  const details = createMemo(() => {
    version();
    const parts: string[] = [`${labelledKeys("cpu.usage", "core").length} cœurs`];
    const temp = lastValue("cpu.temp_c");
    if (temp != null) parts.push(`${temp.toFixed(0)} °C`);
    const freqs = labelledKeys("cpu.freq_mhz", "core")
      .map((f) => lastValue(f.key))
      .filter((v): v is number => v != null);
    if (freqs.length > 0) {
      const avg = freqs.reduce((a, b) => a + b, 0) / freqs.length;
      parts.push(`${(avg / 1000).toFixed(2)} GHz`);
    }
    const watts = lastValue("cpu.power_w");
    if (watts != null) parts.push(`${watts.toFixed(1)} W`);
    return parts.join(" · ");
  });

  return (
    <div>
      <div class="stat-value">{pct(now())}</div>
      <div class="stat-sub">{details()}</div>
    </div>
  );
}
