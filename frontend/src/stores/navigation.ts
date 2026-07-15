import { createSignal } from "solid-js";

/**
 * Navigation (issue #22) : un simple signal — pas d'URL, pas de lib de
 * routing, une app desktop n'en a pas besoin.
 */
export type PageId = "dashboard" | "processes" | "settings";

const [currentPage, setCurrentPage] = createSignal<PageId>("dashboard");

export { currentPage };

export function navigate(page: PageId) {
  setCurrentPage(page);
}
