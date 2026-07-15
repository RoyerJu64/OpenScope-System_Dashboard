import {
  createEffect,
  createMemo,
  createSignal,
  For,
  onCleanup,
  onMount,
} from "solid-js";
import { Dynamic } from "solid-js/web";
import { getLayout, saveLayout } from "../../ipc/client";
import { isAvailable } from "../../stores/capabilities";
import type { WidgetDef } from "../../widgets/registry";

export const GRID_COLS = 6;
const MIN_H = 1;
const MAX_H = 6;

/** Schéma persisté (issue #24), versionné pour évoluer sans casse. */
interface SavedLayout {
  version: 1;
  order: string[];
  sizes: Record<string, { w: number; h: number }>;
}

interface Props {
  page: string;
  widgets: WidgetDef[];
}

/**
 * Grille de widgets (issue #23) : drag & drop par réordonnancement
 * (CSS grid `dense` compacte automatiquement) et redimensionnement par
 * poignée, en cellules de grille. Disposition sauvegardée par page,
 * avec un debounce.
 */
export function Grid(props: Props) {
  const [order, setOrder] = createSignal<string[]>(
    props.widgets.map((w) => w.id),
  );
  const [sizes, setSizes] = createSignal<Record<string, { w: number; h: number }>>(
    Object.fromEntries(props.widgets.map((w) => [w.id, w.defaultSize])),
  );
  const [dragged, setDragged] = createSignal<string | null>(null);
  let container!: HTMLDivElement;
  let loaded = false;
  let saveTimer: number | undefined;

  const visible = createMemo(() => {
    const byId = new Map(props.widgets.map((w) => [w.id, w]));
    return order()
      .map((id) => byId.get(id))
      .filter((w): w is WidgetDef => !!w && isAvailable(w.collector));
  });

  onMount(async () => {
    try {
      const saved = (await getLayout(props.page)) as SavedLayout | null;
      if (saved?.version === 1) {
        // Widgets apparus depuis la sauvegarde : ajoutés à la fin.
        const known = new Set(saved.order);
        setOrder([
          ...saved.order.filter((id) => props.widgets.some((w) => w.id === id)),
          ...props.widgets.filter((w) => !known.has(w.id)).map((w) => w.id),
        ]);
        setSizes((current) => ({ ...current, ...saved.sizes }));
      }
    } catch {
      // disposition illisible : défauts
    }
    loaded = true;
  });

  const persist = () => {
    if (!loaded) return;
    window.clearTimeout(saveTimer);
    saveTimer = window.setTimeout(() => {
      void saveLayout(props.page, {
        version: 1,
        order: order(),
        sizes: sizes(),
      } satisfies SavedLayout).catch(() => {});
    }, 500);
  };
  onCleanup(() => window.clearTimeout(saveTimer));

  // ── Drag : réordonner en survolant les autres widgets ──────────────
  const onDragStart = (id: string, e: PointerEvent) => {
    if (e.button !== 0) return;
    e.preventDefault();
    setDragged(id);

    const move = (ev: PointerEvent) => {
      const target = document
        .elementsFromPoint(ev.clientX, ev.clientY)
        .find(
          (el): el is HTMLElement =>
            el instanceof HTMLElement &&
            el.dataset.wid !== undefined &&
            el.dataset.wid !== id,
        );
      if (!target) return;
      const overId = target.dataset.wid!;
      const rect = target.getBoundingClientRect();
      // Avant ou après le widget survolé, selon la moitié franchie.
      const after =
        ev.clientX - rect.left > rect.width / 2 ||
        ev.clientY - rect.top > rect.height / 2;
      setOrder((current) => {
        const without = current.filter((x) => x !== id);
        const at = without.indexOf(overId) + (after ? 1 : 0);
        if (current[current.indexOf(overId) + (after ? 1 : -1)] === id) {
          return current; // déjà à la bonne place : évite l'oscillation
        }
        return [...without.slice(0, at), id, ...without.slice(at)];
      });
    };
    const up = () => {
      window.removeEventListener("pointermove", move);
      window.removeEventListener("pointerup", up);
      setDragged(null);
      persist();
    };
    window.addEventListener("pointermove", move);
    window.addEventListener("pointerup", up);
  };

  // ── Resize : poignée en bas à droite, en cellules ───────────────────
  const onResizeStart = (id: string, e: PointerEvent) => {
    if (e.button !== 0) return;
    e.preventDefault();
    e.stopPropagation();
    const start = sizes()[id] ?? { w: 2, h: 2 };
    const cellW = container.clientWidth / GRID_COLS;
    const rowH = 96 + 14; // grid-auto-rows + gap (voir CSS)
    const from = { x: e.clientX, y: e.clientY };

    const move = (ev: PointerEvent) => {
      const w = Math.min(
        GRID_COLS,
        Math.max(1, start.w + Math.round((ev.clientX - from.x) / cellW)),
      );
      const h = Math.min(
        MAX_H,
        Math.max(MIN_H, start.h + Math.round((ev.clientY - from.y) / rowH)),
      );
      setSizes((current) =>
        current[id]?.w === w && current[id]?.h === h
          ? current
          : { ...current, [id]: { w, h } },
      );
    };
    const up = () => {
      window.removeEventListener("pointermove", move);
      window.removeEventListener("pointerup", up);
      persist();
    };
    window.addEventListener("pointermove", move);
    window.addEventListener("pointerup", up);
  };

  createEffect(() => {
    order();
    sizes();
    persist();
  });

  return (
    <div ref={container} class="grid">
      <For each={visible()}>
        {(widget) => {
          const size = () => sizes()[widget.id] ?? widget.defaultSize;
          return (
            <section
              class="card widget"
              classList={{ dragging: dragged() === widget.id }}
              data-wid={widget.id}
              style={{
                "grid-column": `span ${Math.min(size().w, GRID_COLS)}`,
                "grid-row": `span ${size().h}`,
              }}
            >
              <h2
                class="card-title widget-handle"
                onPointerDown={(e) => onDragStart(widget.id, e)}
              >
                {widget.title}
              </h2>
              <div class="widget-body">
                <Dynamic component={widget.component} />
              </div>
              <div
                class="widget-resize"
                title="Redimensionner"
                onPointerDown={(e) => onResizeStart(widget.id, e)}
              />
            </section>
          );
        }}
      </For>
    </div>
  );
}
