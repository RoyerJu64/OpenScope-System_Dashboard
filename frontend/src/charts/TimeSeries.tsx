import { createEffect, For, onCleanup, onMount, Show } from "solid-js";
import uPlot from "uplot";
import "uplot/dist/uPlot.min.css";
import { seriesFor, version } from "../stores/metrics";

export interface SeriesSpec {
  /** Clé de série du store (métrique, éventuellement + labels). */
  seriesKey: string;
  label: string;
  /** Token CSS de couleur ; à défaut, l'ordre catégoriel --series-N. */
  colorToken?: string;
}

interface Props {
  series: SeriesSpec[];
  /** Suffixe simple pour les valeurs (ex. "%"). */
  unit?: string;
  /** "bytes" : valeurs humanisées en o/s → Gio/s (axe et tooltip). */
  format?: "bytes";
  /** Hauteur fixe en px ; absente, le graphe remplit son conteneur. */
  height?: number;
  /** Bornes fixes de l'axe Y (ex. 0–100 pour des %). */
  range?: [number, number];
}

function cssVar(name: string): string {
  return getComputedStyle(document.documentElement)
    .getPropertyValue(name)
    .trim();
}

function fmtBytesPerSec(v: number): string {
  const units = ["o/s", "Kio/s", "Mio/s", "Gio/s"];
  let x = Math.abs(v);
  let i = 0;
  while (x >= 1024 && i < units.length - 1) {
    x /= 1024;
    i += 1;
  }
  return `${x >= 100 ? x.toFixed(0) : x.toFixed(1)} ${units[i]}`;
}

/**
 * Graphe temporel uPlot thémé : lignes 2px, grille discrète, crosshair +
 * tooltip. Légende affichée dès deux séries (l'identité n'est jamais
 * portée par la couleur seule).
 */
export function TimeSeries(props: Props) {
  let container!: HTMLDivElement;
  let tooltip!: HTMLDivElement;
  let plot: uPlot | undefined;
  const currentHeight = () => props.height ?? (container.clientHeight || 200);

  const color = (spec: SeriesSpec, index: number) =>
    cssVar(spec.colorToken ?? `--series-${index + 1}`);

  const fmtValue = (v: number) =>
    props.format === "bytes"
      ? fmtBytesPerSec(v)
      : `${v.toFixed(1)}${props.unit ? ` ${props.unit}` : ""}`;

  onMount(() => {
    const grid = cssVar("--gridline");
    const muted = cssVar("--ink-muted");
    const axisFont = `11px ${cssVar("--font") || "system-ui"}`;

    const opts: uPlot.Options = {
      width: container.clientWidth || 300,
      height: currentHeight(),
      legend: { show: false },
      cursor: { y: false, points: { size: 8 } },
      scales: {
        y: {
          range: props.range
            ? () => props.range as [number, number]
            : undefined,
        },
      },
      axes: [
        {
          stroke: muted,
          font: axisFont,
          grid: { show: false },
          ticks: { show: false },
        },
        {
          stroke: muted,
          font: axisFont,
          size: props.format === "bytes" ? 68 : 46,
          grid: { stroke: grid, width: 1 },
          ticks: { show: false },
          values:
            props.format === "bytes"
              ? (_u, ticks) => ticks.map(fmtBytesPerSec)
              : undefined,
        },
      ],
      series: [
        {},
        ...props.series.map((spec, i) => ({
          label: spec.label,
          stroke: color(spec, i),
          width: 2,
          fill: props.series.length === 1 ? `${color(spec, i)}1f` : undefined,
          points: { show: false },
        })),
      ],
      hooks: {
        setCursor: [
          (u) => {
            const idx = u.cursor.idx;
            if (idx == null || u.data[0][idx] == null) {
              tooltip.style.visibility = "hidden";
              return;
            }
            const ts = u.data[0][idx] as number;
            const time = new Date(ts * 1000).toLocaleTimeString();
            const parts = props.series
              .map((spec, i) => {
                const val = u.data[i + 1][idx];
                return val == null
                  ? null
                  : `<span class="t-value">${spec.label} ${fmtValue(val as number)}</span>`;
              })
              .filter(Boolean);
            if (parts.length === 0) {
              tooltip.style.visibility = "hidden";
              return;
            }
            tooltip.innerHTML = `<span class="t-time">${time}</span> · ${parts.join(" · ")}`;
            const firstVal = u.data[1][idx];
            tooltip.style.left = `${u.valToPos(ts, "x")}px`;
            tooltip.style.top = `${firstVal == null ? 12 : u.valToPos(firstVal as number, "y")}px`;
            tooltip.style.visibility = "visible";
          },
        ],
      },
    };

    plot = new uPlot(
      opts,
      [[], ...props.series.map(() => [])] as uPlot.AlignedData,
      container,
    );

    const resize = new ResizeObserver(() => {
      if (container.clientWidth > 0) {
        plot?.setSize({
          width: container.clientWidth,
          height: currentHeight(),
        });
      }
    });
    resize.observe(container);
    onCleanup(() => {
      resize.disconnect();
      plot?.destroy();
    });
  });

  createEffect(() => {
    version(); // dépendance réactive : redessine à chaque batch
    const buffers = props.series.map((s) => seriesFor(s.seriesKey));
    // Les séries d'un même collecteur partagent leurs timestamps ; on
    // aligne par la fin au cas où l'une démarre plus tard.
    const len = Math.min(...buffers.map((b) => b.ts.length));
    const ts = buffers[0].ts.slice(-len);
    plot?.setData([ts, ...buffers.map((b) => b.v.slice(-len))]);
  });

  return (
    <div
      style={
        props.height
          ? undefined
          : { display: "flex", "flex-direction": "column", height: "100%", "min-height": "0" }
      }
    >
      <Show when={props.series.length > 1}>
        <div class="chart-legend">
          <For each={props.series}>
            {(spec, i) => (
              <span>
                <span
                  class="legend-dot"
                  style={{ background: color(spec, i()) }}
                />
                {spec.label}
              </span>
            )}
          </For>
        </div>
      </Show>
      <div
        ref={container}
        class="chart"
        style={props.height ? { height: `${props.height}px` } : { flex: "1", "min-height": "0" }}
      >
        <div ref={tooltip} class="chart-tooltip" />
      </div>
    </div>
  );
}
