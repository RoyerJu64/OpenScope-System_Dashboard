import { For } from "solid-js";
import { currentPage, navigate, type PageId } from "../stores/navigation";

interface NavEntry {
  icon: string;
  label: string;
  /** Page routée ; absent = pas encore livrée. */
  page?: PageId;
  /** Issue GitHub qui livrera l'entrée (tooltip des entrées grisées). */
  issue?: number;
}

const NAV: NavEntry[] = [
  { icon: "▦", label: "Dashboard", page: "dashboard" },
  { icon: "≡", label: "Processus", page: "processes" },
  { icon: "▣", label: "CPU", issue: 25 },
  { icon: "◔", label: "GPU", issue: 35 },
  { icon: "◫", label: "Disques", issue: 37 },
  { icon: "⇅", label: "Réseau", issue: 42 },
  { icon: "◧", label: "Docker", issue: 40 },
  { icon: "◷", label: "Historique", issue: 49 },
  { icon: "◎", label: "Alertes", issue: 55 },
  { icon: "⚙", label: "Paramètres", page: "settings" },
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
            classList={{ active: entry.page === currentPage() }}
            disabled={!entry.page}
            title={entry.issue ? `Arrive avec l'issue #${entry.issue}` : undefined}
            onClick={() => entry.page && navigate(entry.page)}
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
