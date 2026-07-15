import { createSignal } from "solid-js";
import { getCapabilities } from "../ipc/client";
import type { Capabilities } from "../ipc/types";

/**
 * Capacités de la machine (issue #19) : quels collecteurs sont
 * disponibles, avec quels détails. Pilote l'affichage des widgets —
 * dégradation gracieuse : pas de capteur → pas de widget, pas d'erreur.
 */
const [capabilities, setCapabilities] = createSignal<Capabilities | null>(null);
export { capabilities };

/**
 * `false` uniquement si le backend a explicitement déclaré le collecteur
 * indisponible ; `true` sinon (y compris avant le chargement, pour ne pas
 * faire clignoter les widgets au démarrage).
 */
export function isAvailable(collector: string): boolean {
  const caps = capabilities();
  if (!caps) return true;
  return caps.collectors[collector]?.available ?? false;
}

/** Détail remonté par le probe (ex. "cores" → "16"), sinon null. */
export function capabilityDetail(
  collector: string,
  key: string,
): string | null {
  return capabilities()?.collectors[collector]?.details?.[key] ?? null;
}

/**
 * Charge les capacités. Le scheduler démarre en asynchrone côté backend :
 * on réessaie tant que la liste est vide (démarrage), puis on s'arrête.
 */
export async function loadCapabilities(): Promise<void> {
  for (let attempt = 0; attempt < 20; attempt += 1) {
    try {
      const caps = await getCapabilities();
      // "process" est déclaré statiquement par le backend : on attend
      // qu'au moins un collecteur du scheduler soit probé.
      if (Object.keys(caps.collectors).some((k) => k !== "process")) {
        setCapabilities(caps);
        return;
      }
    } catch {
      // backend pas prêt : on retente
    }
    await new Promise((resolve) => setTimeout(resolve, 250));
  }
}
