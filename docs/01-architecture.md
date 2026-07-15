# 01 — Architecture

## Vue d'ensemble

Le projet est un **monorepo** : un workspace Cargo (backend) + un frontend TypeScript embarqué par Tauri.

```
┌────────────────────────────── Frontend (webview Tauri) ──────────────────────────────┐
│  SolidJS + TypeScript                                                                │
│  Sidebar · Grille de widgets drag & drop · Graphes uPlot · Thèmes · Paramètres       │
└────────────▲─────────────────────────────────────────────┬──────────────────────────┘
             │ events (push IPC : batches de métriques)     │ commands (invoke IPC)
┌────────────┴─────────────────────────────────────────────▼──────────────────────────┐
│  openscope-app  (binaire Tauri — orchestration uniquement, pas de logique métier)        │
│                                                                                      │
│   ┌──────────────────────────  MetricBus (tokio::sync::broadcast)  ───────────────┐  │
│   │      producteurs ──► Sample{source, metric, ts, value, labels} ──► abonnés    │  │
│   └───▲───────▲───────▲──────────────────────────┬───────────┬─────────┬──────────┘  │
│       │       │       │                          │           │         │             │
│  ┌────┴───┐ ┌─┴─────┐ ┌┴─────────┐        ┌──────▼───┐ ┌─────▼────┐ ┌──▼─────────┐   │
│  │ openscope- │ │ openscope-│ │ openscope-   │        │ openscope-   │ │ openscope-   │ │ frontend   │   │
│  │ collect│ │ remote│ │ plugins  │        │ history  │ │ alerts   │ │ forwarder  │   │
│  │ (local)│ │ (SSH) │ │ (WASM)   │        │ (SQLite) │ │ (règles) │ │ (IPC push) │   │
│  └────────┘ └───────┘ └──────────┘        └──────────┘ └──────────┘ └────────────┘   │
└──────────────────────────────────────────────────────────────────────────────────────┘

          openscope-agent (« openscoped ») : binaire headless réutilisant openscope-collect,
          lancé sur les machines distantes via SSH, parle JSON-Lines sur stdio.
```

## Les modules (crates)

### `openscope-core` — le contrat commun
Aucune dépendance système. Contient :
- les **types de données** : `Sample`, `MetricId`, `SourceId`, `Value`, `Labels`, `Capabilities` ;
- les **traits** : `Collector`, `HistoryStore`, `AlertSink`, `MetricSource` ;
- le **MetricBus** (wrapper typé autour de `tokio::sync::broadcast`) ;
- les erreurs communes (`thiserror`).

Tout le reste dépend de `openscope-core` ; aucun crate ne dépend d'un autre crate métier directement. C'est ce qui garantit la modularité.

### `openscope-collect` — collecte système locale
Un sous-module par domaine, chacun implémentant le trait `Collector` :

| Collecteur | Linux | Windows | macOS |
|------------|-------|---------|-------|
| `cpu`      | procfs + hwmon + RAPL | sysinfo + WMI/LibreHW | sysinfo + SMC |
| `memory`   | procfs | sysinfo | sysinfo |
| `gpu`      | NVML / sysfs amdgpu / i915 hwmon | NVML / DXGI | Metal (limité) |
| `disk`     | procfs diskstats + smartctl | PDH + smartctl | IOKit + smartctl |
| `network`  | procfs + netlink (sockets) | GetIfTable2 | sysctl |
| `process`  | procfs | NtQuerySystemInformation (via sysinfo) | libproc |
| `docker`   | bollard (socket unix) | bollard (named pipe) | bollard |
| `vm`       | libvirt | — (post-v1) | — |

Chaque collecteur expose `probe()` : au démarrage, le runtime détecte ce qui est disponible et n'active que les collecteurs fonctionnels (→ dégradation gracieuse).

Le **scheduler de collecte** (dans ce crate) tick chaque collecteur à son intervalle configuré (les intervalles sont indépendants : processus à 2 s, CPU à 1 s, SMART à 5 min).

### `openscope-history` — historique et exports
- Buffer circulaire en mémoire (fenêtre chaude, ~10 min à résolution pleine) pour les graphes temps réel.
- Persistance **SQLite** (`rusqlite`) avec **downsampling par paliers** (style RRD) :
  - brut 1 s → conservé 15 min
  - agrégé 10 s (min/max/avg) → 24 h
  - agrégé 1 min → 30 j
  - agrégé 15 min → 1 an
- Exports **CSV** et **JSON** sur une plage temporelle et une sélection de métriques.
- **Snapshots** : capture complète de l'état à l'instant T (toutes métriques + liste de processus), stockée et nommable → base de la fonction *comparaison avant/après* (diff de deux snapshots).

### `openscope-alerts` — moteur d'alertes
- Règles déclaratives : `métrique + condition (>, <, absent) + durée de maintien + sévérité`.
- Abonné au MetricBus ; machine à états par règle (`Ok → Pending → Firing → Resolved`).
- Sorties : notification desktop (plugin Tauri), événement UI, hook de commande shell (post-MVP), webhook (post-MVP).

### `openscope-remote` — monitoring distant
- Se connecte en SSH (`russh`), **déploie ou exécute** l'agent `openscope-agent` sur la machine distante, et parle un protocole **JSON-Lines sur stdio** (pas de port à ouvrir, pas de service à installer).
- Chaque machine distante devient une `MetricSource` qui publie sur le même MetricBus, avec un `SourceId` distinct → l'historique, les alertes et l'UI fonctionnent sans modification pour N machines.
- Fallback si l'agent ne peut pas être déployé (arch inconnue) : mode dégradé par parsing de commandes standard (`cat /proc/stat`, etc.).

### `openscope-agent` — binaire headless (`openscoped`)
- Réutilise `openscope-collect` tel quel ; boucle : collecte → sérialise en JSON-Lines → stdout.
- Compilé en binaire statique (musl sur Linux) pour être copié via SFTP sans dépendances.

### `openscope-plugins` — système de plugins
- Runtime **WASM via Extism** : sandboxé, multiplateforme, langage libre pour les auteurs de plugins.
- Deux types de plugins :
  1. **Collecteur** : exporte `probe()` / `collect()` → publie des métriques custom sur le bus ;
  2. **Widget** : le manifeste déclare un widget dont le rendu est décrit en JSON déclaratif (type de graphe, métriques consommées) — pas de JS arbitraire injecté dans la webview (sécurité).
- Manifeste `plugin.toml` : nom, version, permissions demandées (fs, réseau, exec), métriques exposées.

### `openscope-app` — le binaire Tauri
Uniquement de l'**orchestration** :
- démarre le runtime, charge la config, instancie les modules, câble le bus ;
- expose les **commands IPC** (voir [04-interfaces.md](04-interfaces.md)) ;
- forwarde les batches de métriques vers la webview (1 event par tick, jamais 1 event par métrique) ;
- exécute les **actions** (kill, renice) avec confirmation côté UI.

### `frontend/` — SolidJS + TypeScript
- **Shell** : sidebar (machines + pages : Vue d'ensemble, CPU, GPU, Disques, Réseau, Processus, Docker, Historique, Alertes, Paramètres).
- **Dashboard** : grille de widgets déplaçables/redimensionnables (moteur de grille maison léger, ~300 lignes, pas de dépendance lourde), disposition sauvegardée par machine.
- **Graphes** : uPlot (canvas, gère 100k points sans effort) habillé aux couleurs du thème.
- **Store** : un store Solid par domaine, alimenté par les events IPC ; ring buffers côté client pour les fenêtres visibles.
- Dark mode par défaut, light disponible ; animations discrètes (transitions CSS uniquement, pas de lib d'animation).

## Flux de données (exemple : usage CPU)

1. Le scheduler tick `CpuCollector::collect()` toutes les 1 s.
2. Le collecteur lit `/proc/stat`, calcule les deltas, émet `Vec<Sample>` (1 global + 1 par cœur + fréquences + températures).
3. Le bus broadcast le batch. Trois abonnés le reçoivent indépendamment :
   - `openscope-history` : append dans le ring buffer + écriture SQLite par lot (transaction toutes les 10 s) ;
   - `openscope-alerts` : évalue les règles concernées ;
   - le forwarder IPC : agrège le tick complet et pousse **un** event `metrics-batch` à la webview.
4. Le store frontend met à jour ses ring buffers ; les widgets abonnés re-rendent (Solid = réactivité fine, seuls les graphes concernés redessinent).

## Design patterns structurants

| Pattern | Où | Pourquoi |
|---------|-----|----------|
| **Trait objects + registry** (`Vec<Box<dyn Collector>>`) | openscope-collect | ajouter un domaine = un fichier, zéro modification ailleurs |
| **Publish/Subscribe** (MetricBus) | openscope-core | découplage total producteurs/consommateurs ; N machines = N producteurs |
| **Capability detection** (`probe()` + `Capabilities`) | tous les collecteurs | dégradation gracieuse multiplateforme |
| **Repository** (`HistoryStore` trait) | openscope-history | SQLite remplaçable (tests en mémoire, futur backend distant) |
| **Command** (actions kill/renice/pause container) | openscope-app | audit, confirmation UI, permissions élévation |
| **State machine** | openscope-alerts | anti-flapping (durée de maintien avant Firing) |
| **DTO serde à la frontière IPC** | openscope-app ↔ frontend | types internes libres d'évoluer sans casser le front |
| **Sandbox + manifeste de permissions** | openscope-plugins | plugins tiers sans compromettre la machine |

## Threading

- Runtime **tokio multi-thread** pour tout le backend.
- Chaque collecteur tourne dans sa propre tâche ; un collecteur lent (SMART ~200 ms, Docker) ne bloque jamais les autres.
- Les lectures bloquantes (smartctl, libvirt) passent par `spawn_blocking`.
- SQLite sur une tâche dédiée (channel mpsc), écritures par lot.
