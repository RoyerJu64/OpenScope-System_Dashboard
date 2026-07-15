import { createMemo } from "solid-js";
import { TimeSeries } from "../charts/TimeSeries";
import { lastValue, seriesFor, version } from "../stores/metrics";

/**
 * Dashboard de la Phase 0 : valide la chaîne collecteur → bus → IPC →
 * graphe avec le collecteur factice. La grille drag & drop et les vrais
 * widgets arrivent en Phase 1 (issues #23–#26).
 */
export function Dashboard() {
  const sineNow = createMemo(() => {
    version();
    return lastValue("mock.sine");
  });

  const pointCount = createMemo(() => {
    version();
    return seriesFor("mock.sine").ts.length;
  });

  return (
    <main class="main">
      <h1 class="page-title">Vue d'ensemble</h1>
      <div class="widget-grid">
        <section class="card" style={{ "grid-column": "span 2" }}>
          <h2 class="card-title">Signal de démonstration — mock.sine</h2>
          <TimeSeries
            seriesKey="mock.sine"
            label="mock.sine"
            colorToken="--series-1"
            unit="%"
            range={[0, 100]}
            height={220}
          />
        </section>
        <section class="card">
          <h2 class="card-title">Valeur instantanée</h2>
          <div class="stat-value">
            {sineNow() == null ? "—" : `${sineNow()!.toFixed(1)} %`}
          </div>
          <div class="stat-sub">
            {pointCount()} points en fenêtre · collecteur « mock » à 2 Hz
          </div>
        </section>
      </div>
    </main>
  );
}
