import { onMount } from "solid-js";
import { Sidebar } from "./layout/Sidebar";
import { Dashboard } from "./pages/Dashboard";
import { startMetricsListener } from "./stores/metrics";

export function App() {
  onMount(() => {
    void startMetricsListener();
  });

  return (
    <div class="shell">
      <Sidebar />
      <Dashboard />
    </div>
  );
}
