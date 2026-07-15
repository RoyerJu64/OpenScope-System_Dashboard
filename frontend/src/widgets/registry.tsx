import type { Component } from "solid-js";
import { CpuCores } from "./CpuCores";
import { CpuInstant } from "./CpuInstant";
import { CpuUsage } from "./CpuUsage";
import { DiskIo } from "./DiskIo";
import { MemoryInstant } from "./MemoryInstant";
import { MemoryUsage } from "./MemoryUsage";
import { NetworkIo } from "./NetworkIo";
import { TopProcesses } from "./TopProcesses";

export interface WidgetDef {
  id: string;
  title: string;
  /** Capacité requise : le widget n'apparaît que si elle est disponible. */
  collector: string;
  /** Taille par défaut en cellules de grille (colonnes × rangées). */
  defaultSize: { w: number; h: number };
  component: Component;
}

/**
 * Le catalogue des widgets du dashboard. Ajouter un widget = une entrée
 * ici + son composant ; la grille, la persistance et la dégradation
 * gracieuse sont automatiques.
 */
export const WIDGETS: WidgetDef[] = [
  {
    id: "cpu-usage",
    title: "CPU — utilisation",
    collector: "cpu",
    defaultSize: { w: 4, h: 3 },
    component: CpuUsage,
  },
  {
    id: "cpu-instant",
    title: "CPU — instantané",
    collector: "cpu",
    defaultSize: { w: 2, h: 3 },
    component: CpuInstant,
  },
  {
    id: "cpu-cores",
    title: "CPU — cœurs",
    collector: "cpu",
    defaultSize: { w: 6, h: 2 },
    component: CpuCores,
  },
  {
    id: "mem-usage",
    title: "Mémoire — utilisation",
    collector: "memory",
    defaultSize: { w: 4, h: 2 },
    component: MemoryUsage,
  },
  {
    id: "mem-instant",
    title: "Mémoire — instantané",
    collector: "memory",
    defaultSize: { w: 2, h: 2 },
    component: MemoryInstant,
  },
  {
    id: "disk-io",
    title: "Disque — débits",
    collector: "disk",
    defaultSize: { w: 3, h: 2 },
    component: DiskIo,
  },
  {
    id: "net-io",
    title: "Réseau — débits",
    collector: "network",
    defaultSize: { w: 3, h: 2 },
    component: NetworkIo,
  },
  {
    id: "top-processes",
    title: "Top processus",
    collector: "process",
    defaultSize: { w: 3, h: 3 },
    component: TopProcesses,
  },
];
