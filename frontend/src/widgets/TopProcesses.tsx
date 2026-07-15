import { createSignal, For, onCleanup, onMount } from "solid-js";
import { listProcesses } from "../ipc/client";
import type { ProcessRowDto } from "../ipc/types";

const REFRESH_MS = 2000;
const TOP_N = 8;

/** Top des processus par CPU — version mini pour le dashboard ;
 *  la table complète est sur la page Processus (issue #28). */
export function TopProcesses() {
  const [rows, setRows] = createSignal<ProcessRowDto[]>([]);

  onMount(() => {
    const tick = async () => {
      try {
        const all = await listProcesses();
        all.sort((a, b) => b.cpu_pct - a.cpu_pct);
        setRows(all.slice(0, TOP_N));
      } catch {
        // backend indisponible : la liste précédente reste affichée
      }
    };
    void tick();
    const timer = window.setInterval(() => void tick(), REFRESH_MS);
    onCleanup(() => window.clearInterval(timer));
  });

  return (
    <div class="mini-table">
      <For each={rows()}>
        {(p) => (
          <div class="mini-row" title={p.cmdline || p.name}>
            <span class="mini-name">{p.name}</span>
            <span class="mini-val">{p.cpu_pct.toFixed(1)} %</span>
            <span class="mini-val mini-muted">
              {(p.rss_bytes / 2 ** 20).toFixed(0)} Mio
            </span>
          </div>
        )}
      </For>
    </div>
  );
}
