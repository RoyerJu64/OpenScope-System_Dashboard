import { Match, onMount, Switch } from "solid-js";
import { Sidebar } from "./layout/Sidebar";
import { Dashboard } from "./pages/Dashboard";
import { Processes } from "./pages/Processes";
import { Settings } from "./pages/Settings";
import { loadCapabilities } from "./stores/capabilities";
import { startMetricsListener } from "./stores/metrics";
import { currentPage } from "./stores/navigation";

export function App() {
  onMount(() => {
    void startMetricsListener();
    void loadCapabilities();
  });

  return (
    <div class="shell">
      <Sidebar />
      <Switch fallback={<Dashboard />}>
        <Match when={currentPage() === "processes"}>
          <Processes />
        </Match>
        <Match when={currentPage() === "settings"}>
          <Settings />
        </Match>
      </Switch>
    </div>
  );
}
