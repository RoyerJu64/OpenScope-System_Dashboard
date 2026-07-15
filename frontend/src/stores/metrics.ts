import { createSignal } from "solid-js";
import { getHotWindow, onMetricsBatch } from "../ipc/client";
import type { BatchDto } from "../ipc/types";

/** Métriques pré-remplies au démarrage depuis la fenêtre chaude backend. */
const PREFILL_METRICS = [
  "cpu.usage",
  "mem.used_pct",
  "disk.read_bps",
  "disk.write_bps",
  "net.rx_bps",
  "net.tx_bps",
];

/**
 * Fenêtre visible côté client. Le backend garde la fenêtre chaude
 * complète ; ici on ne conserve que ce que les graphes affichent.
 */
const WINDOW_POINTS = 1200;

export interface SeriesBuffer {
  /** secondes epoch (format attendu par uPlot) */
  ts: number[];
  v: (number | null)[];
}

const buffers = new Map<string, SeriesBuffer>();

/** Signal de version : incrémenté à chaque batch, les graphes s'y abonnent. */
const [version, setVersion] = createSignal(0);
export { version };

/** Clé de série : métrique + labels triés (deux cœurs CPU = deux séries). */
function seriesKey(metric: string, labels?: Record<string, string>): string {
  if (!labels || Object.keys(labels).length === 0) return metric;
  const suffix = Object.entries(labels)
    .sort(([a], [b]) => a.localeCompare(b))
    .map(([k, v]) => `${k}=${v}`)
    .join(",");
  return `${metric}{${suffix}}`;
}

export function seriesFor(key: string): SeriesBuffer {
  let buf = buffers.get(key);
  if (!buf) {
    buf = { ts: [], v: [] };
    buffers.set(key, buf);
  }
  return buf;
}

/** Dernière valeur numérique connue d'une série (pour les stat tiles). */
export function lastValue(key: string): number | null {
  const buf = buffers.get(key);
  const v = buf?.v[buf.v.length - 1];
  return v ?? null;
}

/**
 * Séries d'une métrique déclinée par label — ex. `cpu.usage` par `core`
 * donne [{label: "0", key: "cpu.usage{core=0}"}, …], triées numériquement.
 * À lire dans un contexte réactif avec `version()` pour suivre l'arrivée
 * de nouvelles séries.
 */
export function labelledKeys(
  metric: string,
  labelKey: string,
): { label: string; key: string }[] {
  const prefix = `${metric}{`;
  const out: { label: string; key: string }[] = [];
  for (const key of buffers.keys()) {
    if (!key.startsWith(prefix) || !key.endsWith("}")) continue;
    const pair = key
      .slice(prefix.length, -1)
      .split(",")
      .find((p) => p.startsWith(`${labelKey}=`));
    if (pair) out.push({ label: pair.slice(labelKey.length + 1), key });
  }
  out.sort(
    (a, b) => Number(a.label) - Number(b.label) || a.label.localeCompare(b.label),
  );
  return out;
}

function ingest(batch: BatchDto) {
  const ts = batch.ts_ms / 1000;
  for (const sample of batch.samples) {
    if (sample.value.kind === "text") continue;
    const buf = seriesFor(seriesKey(sample.metric, sample.labels));
    buf.ts.push(ts);
    buf.v.push(sample.value.v);
    if (buf.ts.length > WINDOW_POINTS) {
      buf.ts.splice(0, buf.ts.length - WINDOW_POINTS);
      buf.v.splice(0, buf.v.length - WINDOW_POINTS);
    }
  }
  setVersion((n) => n + 1);
}

let started = false;

/**
 * Branche le store sur l'event IPC `metrics-batch`, puis pré-remplit les
 * graphes avec la fenêtre chaude backend (issue #20) : à la réouverture
 * de l'app ou d'une page, l'historique récent est déjà là. Idempotent.
 */
export async function startMetricsListener(): Promise<void> {
  if (started) return;
  started = true;
  await onMetricsBatch(ingest);

  try {
    const hot = await getHotWindow(PREFILL_METRICS);
    for (const s of hot) {
      const buf = seriesFor(seriesKey(s.metric, s.labels));
      // On ne préfixe que les points antérieurs au premier point live,
      // pour ne jamais dupliquer ce que le listener a déjà reçu.
      const firstLive = buf.ts[0] ?? Infinity;
      const ts: number[] = [];
      const v: (number | null)[] = [];
      for (let i = 0; i < s.ts_ms.length; i += 1) {
        const t = s.ts_ms[i] / 1000;
        if (t < firstLive) {
          ts.push(t);
          v.push(s.values[i]);
        }
      }
      if (ts.length > 0) {
        buf.ts.unshift(...ts);
        buf.v.unshift(...v);
        if (buf.ts.length > WINDOW_POINTS) {
          buf.ts.splice(0, buf.ts.length - WINDOW_POINTS);
          buf.v.splice(0, buf.v.length - WINDOW_POINTS);
        }
      }
    }
    setVersion((n) => n + 1);
  } catch {
    // Fenêtre chaude indisponible : les graphes se remplissent en live.
  }
}
