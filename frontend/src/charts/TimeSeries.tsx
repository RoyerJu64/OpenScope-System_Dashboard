import { createEffect, onCleanup, onMount } from "solid-js";
import uPlot from "uplot";
import "uplot/dist/uPlot.min.css";
import { seriesFor, version } from "../stores/metrics";

interface Props {
  /** Clé de série du store (métrique, éventuellement + labels). */
  seriesKey: string;
  label: string;
  /** Token CSS de couleur, ex. "--series-1". */
  colorToken?: string;
  unit?: string;
  height?: number;
  /** Bornes fixes de l'axe Y (ex. 0–100 pour des %). */
  range?: [number, number];
}

function cssVar(name: string): string {
  return getComputedStyle(document.documentElement)
    .getPropertyValue(name)
    .trim();
}

/**
 * Graphe temporel uPlot thémé : ligne 2px, remplissage léger, grille
 * discrète, crosshair + tooltip. Une série par instance (les widgets
 * multi-séries arrivent avec les vrais collecteurs).
 */
export function TimeSeries(props: Props) {
  let container!: HTMLDivElement;
  let tooltip!: HTMLDivElement;
  let plot: uPlot | undefined;
  const height = props.height ?? 200;

  onMount(() => {
    const stroke = cssVar(props.colorToken ?? "--series-1");
    const grid = cssVar("--gridline");
    const muted = cssVar("--ink-muted");
    const axisFont = `11px ${cssVar("--font") || "system-ui"}`;

    const fmtValue = (v: number) =>
      `${v.toFixed(1)}${props.unit ? ` ${props.unit}` : ""}`;

    const opts: uPlot.Options = {
      width: container.clientWidth || 300,
      height,
      legend: { show: false },
      cursor: {
        y: false,
        points: { size: 8, fill: stroke },
      },
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
          size: 46,
          grid: { stroke: grid, width: 1 },
          ticks: { show: false },
        },
      ],
      series: [
        {},
        {
          label: props.label,
          stroke,
          width: 2,
          fill: `${stroke}1f`,
          points: { show: false },
        },
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
            const val = u.data[1][idx];
            if (val == null) {
              tooltip.style.visibility = "hidden";
              return;
            }
            const time = new Date(ts * 1000).toLocaleTimeString();
            tooltip.innerHTML = `<span class="t-time">${time}</span> · <span class="t-value">${fmtValue(val as number)}</span>`;
            tooltip.style.left = `${u.valToPos(ts, "x")}px`;
            tooltip.style.top = `${u.valToPos(val as number, "y")}px`;
            tooltip.style.visibility = "visible";
          },
        ],
      },
    };

    plot = new uPlot(opts, [[], []], container);

    const resize = new ResizeObserver(() => {
      if (container.clientWidth > 0) {
        plot?.setSize({ width: container.clientWidth, height });
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
    const buf = seriesFor(props.seriesKey);
    plot?.setData([buf.ts, buf.v]);
  });

  return (
    <div ref={container} class="chart" style={{ height: `${height}px` }}>
      <div ref={tooltip} class="chart-tooltip" />
    </div>
  );
}
