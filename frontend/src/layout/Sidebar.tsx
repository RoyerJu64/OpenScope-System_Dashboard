import { For } from "solid-js";

interface NavEntry {
  icon: string;
  label: string;
  /** Phase de la roadmap qui livrera la page (désactivée d'ici là). */
  phase?: number;
}

const NAV: NavEntry[] = [
  { icon: "▦", label: "Dashboard" },
  { icon: "▣", label: "CPU", phase: 1 },
  { icon: "◔", label: "GPU", phase: 2 },
  { icon: "◫", label: "Disques", phase: 1 },
  { icon: "⇅", label: "Réseau", phase: 1 },
  { icon: "≡", label: "Processus", phase: 1 },
  { icon: "◧", label: "Docker", phase: 2 },
  { icon: "◷", label: "Historique", phase: 3 },
  { icon: "◎", label: "Alertes", phase: 3 },
  { icon: "⚙", label: "Paramètres", phase: 1 },
];

export function Sidebar() {
  return (
    <nav class="sidebar">
      <div class="brand">
        <span class="brand-dot" />
        OpenScope
      </div>
      <For each={NAV}>
        {(entry) => (
          <button
            class="nav-item"
            classList={{ active: !entry.phase }}
            disabled={!!entry.phase}
            title={entry.phase ? `Arrive en Phase ${entry.phase}` : undefined}
          >
            <span class="nav-icon">{entry.icon}</span>
            {entry.label}
          </button>
        )}
      </For>
      <div class="sidebar-footer">v0.1.0-dev · Phase 1</div>
    </nav>
  );
}
