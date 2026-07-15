import { createMemo } from "solid-js";
import { TimeSeries } from "../charts/TimeSeries";
import { CpuCores } from "../widgets/CpuCores";
import { labelledKeys, lastValue, version } from "../stores/metrics";

/**
 * Dashboard — widgets CPU, mémoire, disque, réseau (issues #12–#16).
 * La grille drag & drop arrive avec l'issue #23, les pages dédiées
 * avec les issues #22 et suivantes.
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

  // Détails optionnels selon le matériel (température, fréquence
  // moyenne, consommation RAPL) — absents, ils ne s'affichent pas.
  const cpuDetails = createMemo(() => {
    version();
    const parts: string[] = [`${coreCount()} cœurs`];
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

  const gib = (v: number | null) =>
    v == null ? "—" : `${(v / 2 ** 30).toFixed(1)} Gio`;

  const memNow = createMemo(() => {
    version();
    return lastValue("mem.used_pct");
  });

  const memDetails = createMemo(() => {
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
    <main class="main">
      <h1 class="page-title">Vue d'ensemble</h1>
      <div class="widget-grid">
        <section class="card" style={{ "grid-column": "span 2" }}>
          <h2 class="card-title">CPU — utilisation</h2>
          <TimeSeries
            series={[
              { seriesKey: "cpu.usage", label: "CPU", colorToken: "--series-1" },
            ]}
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
          <div class="stat-sub">{cpuDetails()}</div>
        </section>
        <section class="card" style={{ "grid-column": "span 2" }}>
          <h2 class="card-title">CPU — cœurs</h2>
          <CpuCores />
        </section>
        <section class="card" style={{ "grid-column": "span 2" }}>
          <h2 class="card-title">Mémoire — utilisation</h2>
          <TimeSeries
            series={[
              {
                seriesKey: "mem.used_pct",
                label: "Mémoire",
                colorToken: "--series-5",
              },
            ]}
            unit="%"
            range={[0, 100]}
            height={180}
          />
        </section>
        <section class="card">
          <h2 class="card-title">Mémoire — instantané</h2>
          <div class="stat-value">
            {memNow() == null ? "—" : `${memNow()!.toFixed(1)} %`}
          </div>
          <div class="stat-sub">{memDetails()}</div>
        </section>
        <section class="card" style={{ "grid-column": "span 2" }}>
          <h2 class="card-title">Disque — débits</h2>
          <TimeSeries
            series={[
              { seriesKey: "disk.read_bps", label: "Lecture" },
              { seriesKey: "disk.write_bps", label: "Écriture" },
            ]}
            format="bytes"
            height={180}
          />
        </section>
        <section class="card" style={{ "grid-column": "span 2" }}>
          <h2 class="card-title">Réseau — débits</h2>
          <TimeSeries
            series={[
              { seriesKey: "net.rx_bps", label: "Réception" },
              { seriesKey: "net.tx_bps", label: "Émission" },
            ]}
            format="bytes"
            height={180}
          />
        </section>
      </div>
    </main>
  );
}
