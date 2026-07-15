import { Grid } from "../layout/grid/Grid";
import { WIDGETS } from "../widgets/registry";

/**
 * Dashboard : la grille de widgets (issues #23/#24). Les widgets
 * eux-mêmes vivent dans `widgets/` et leur catalogue dans
 * `widgets/registry.tsx` ; la dégradation gracieuse (capacités) est
 * gérée par la grille.
 */
export function Dashboard() {
  return (
    <main class="main">
      <h1 class="page-title">Vue d'ensemble</h1>
      <Grid page="dashboard" widgets={WIDGETS} />
    </main>
  );
}
