import { onMount } from "solid-js";
import { Sidebar } from "./layout/Sidebar";
import { Dashboard } from "./pages/Dashboard";
import { loadCapabilities } from "./stores/capabilities";
import { startMetricsListener } from "./stores/metrics";

export function App() {
  onMount(() => {
    void startMetricsListener();
    void loadCapabilities();
  });

  return (
    <div class="shell">
      <Sidebar />
      <Dashboard />
    </div>
  );
}
