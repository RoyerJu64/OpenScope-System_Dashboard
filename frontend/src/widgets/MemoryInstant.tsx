import { createMemo } from "solid-js";
import { lastValue, version } from "../stores/metrics";
import { gib, pct } from "../utils/format";

export function MemoryInstant() {
  const now = createMemo(() => {
    version();
    return lastValue("mem.used_pct");
  });

  const details = createMemo(() => {
    version();
    const used = lastValue("mem.used_bytes");
    const total = lastValue("mem.total_bytes");
    const cached = lastValue("mem.cached_bytes");
    const swapUsed = lastValue("swap.used_bytes");
    const parts = [`${gib(used)} / ${gib(total)}`];
    if (cached != null) parts.push(`cache ${gib(cached)}`);
    if (swapUsed != null) parts.push(`swap ${gib(swapUsed)}`);
    return parts.join(" · ");
  });

  return (
    <div>
      <div class="stat-value">{pct(now())}</div>
      <div class="stat-sub">{details()}</div>
    </div>
  );
}
