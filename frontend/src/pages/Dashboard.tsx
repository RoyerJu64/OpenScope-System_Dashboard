import { createMemo } from "solid-js";
import { TimeSeries } from "../charts/TimeSeries";
import { CpuCores } from "../widgets/CpuCores";
import { labelledKeys, lastValue, version } from "../stores/metrics";

/**
 * Dashboard — widgets CPU (issue #12). La grille drag & drop et les
 * widgets mémoire/disque/réseau arrivent avec les issues #14–#26.
 */
export function Dashboard() {
  const cpuNow = createMemo(() => {
    version();
    return lastValue("cpu.usage");
  });

  const coreCount = createMemo(() => {
    version();
    return labelledKeys("cpu.usage", "core").length;
  });

  return (
    <main class="main">
      <h1 class="page-title">Vue d'ensemble</h1>
      <div class="widget-grid">
        <section class="card" style={{ "grid-column": "span 2" }}>
          <h2 class="card-title">CPU — utilisation</h2>
          <TimeSeries
            seriesKey="cpu.usage"
            label="CPU"
            colorToken="--series-1"
            unit="%"
            range={[0, 100]}
            height={220}
          />
        </section>
        <section class="card">
          <h2 class="card-title">CPU — instantané</h2>
          <div class="stat-value">
            {cpuNow() == null ? "—" : `${cpuNow()!.toFixed(1)} %`}
          </div>
          <div class="stat-sub">{coreCount()} cœurs · 1 Hz</div>
        </section>
        <section class="card" style={{ "grid-column": "span 2" }}>
          <h2 class="card-title">CPU — cœurs</h2>
          <CpuCores />
        </section>
      </div>
    </main>
  );
}
