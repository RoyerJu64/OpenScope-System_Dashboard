# 07 — Découpage GitHub

## Organisation

- **1 milestone par phase** de la roadmap (`M0 Fondations` → `M6 v1.0`).
- **Labels domaine** : `core`, `collect`, `history`, `alerts`, `remote`, `plugins`, `frontend`, `packaging`, `ci`.
- **Labels type** : `feat`, `bug`, `spike`, `docs`, `perf`, `good-first-issue`.
- Issues rédigées avec critères d'acceptation ; les épics sont des issues chapeau cochant leurs sous-issues.

## M0 — Fondations

| # | Issue | Labels |
|---|-------|--------|
| 1 | Initialiser le workspace Cargo + crates squelettes + rust-toolchain | core |
| 2 | `openscope-core` : types Sample/MetricId/Value/Labels + tests | core |
| 3 | `openscope-core` : traits Collector, MetricSource, HistoryStore | core |
| 4 | `openscope-core` : MetricBus (broadcast, batches Arc) + bench | core, perf |
| 5 | Squelette Tauri 2 + frontend Vite/SolidJS + dark tokens CSS | frontend |
| 6 | Génération types TS via ts-rs + `cargo xtask gen-types` | core, ci |
| 7 | CI GitHub Actions : build+test+clippy 3 OS, eslint/tsc | ci |
| 8 | cargo-deny + politique de licences | ci |
| 9 | Prototype bout-en-bout : collecteur factice → bus → IPC batch → graphe uPlot | core, frontend, spike |
| 10 | Bench perf CI : CPU au repos < 1 % à 1 Hz | perf, ci |

## M1 — MVP (v0.1)

| # | Issue | Labels |
|---|-------|--------|
| 11 | Scheduler de collecte (intervalles indépendants par collecteur) | collect |
| 12 | Collecteur CPU Linux : usage global + par cœur (procfs) | collect |
| 13 | Collecteur CPU : fréquences + températures hwmon + RAPL | collect |
| 14 | Collecteur mémoire : used/free/cache/swap | collect |
| 15 | Collecteur disque : débits R/W, IOPS (diskstats), occupation | collect |
| 16 | Collecteur réseau : débits par interface | collect |
| 17 | Collecteur processus : snapshot, tri, deltas CPU | collect |
| 18 | Actions processus : kill + renice (non privilégié) via pattern Command | collect |
| 19 | `probe()` + `get_capabilities` + câblage dégradation gracieuse | collect, core |
| 20 | Fenêtre chaude en mémoire (ring buffer 10 min) | history |
| 21 | Forwarder IPC : agrégation par tick, event `metrics-batch` | core |
| 22 | Frontend : Shell (sidebar + routing pages) | frontend |
| 23 | Frontend : moteur de grille drag & drop + redimensionnement | frontend |
| 24 | Frontend : persistance de la disposition (`save_layout`) | frontend |
| 25 | Widgets : CPU global/cœurs, fréquence/température, mémoire | frontend |
| 26 | Widgets : débit disque, débit réseau, top processus | frontend |
| 27 | Wrapper uPlot thémé (TimeSeries, Sparkline, Gauge) | frontend |
| 28 | Page Processus : table triable, recherche, kill (confirmation), renice | frontend |
| 29 | Page Paramètres : intervalles de rafraîchissement par collecteur | frontend |
| 30 | Config figment (~/.config/openscope/config.toml) + `get/set_config` | core |

## M2 — Matériel & parité OS (v0.2)

| # | Issue | Labels |
|---|-------|--------|
| 31 | Spike 2 j : lecture GPU AMD (sysfs amdgpu) et Intel (i915/xe hwmon) | collect, spike |
| 32 | GPU NVIDIA via NVML : charge, VRAM, temp, power, enc/dec | collect |
| 33 | GPU : multi-GPU + métriques par processus + colonne GPU table processus | collect, frontend |
| 34 | GPU AMD (sysfs) puis Intel | collect |
| 35 | Widget/page GPU (multi-GPU) | frontend |
| 36 | SMART via `smartctl --json` + élévation ciblée | collect |
| 37 | Page Disques enrichie (SMART, latences) | frontend |
| 38 | Docker via bollard : conteneurs + stats par conteneur | collect |
| 39 | Docker : logs streamés + actions start/stop/restart | collect, frontend |
| 40 | Page Docker | frontend |
| 41 | Connexions réseau actives (netlink + fallback procfs) | collect |
| 42 | Page Réseau complète (interfaces, connexions, historique court) | frontend |
| 43 | Collecteurs Windows : CPU/RAM/disque/réseau/processus | collect |
| 44 | Collecteurs macOS : CPU/RAM/disque/réseau/processus | collect |
| 45 | Spike : élévation polkit/UAC/macOS pour actions privilégiées | collect, spike |
| 46 | Light mode + transitions/animations discrètes | frontend |

## M3 — Historique & alertes (v0.3)

| # | Issue | Labels |
|---|-------|--------|
| 47 | `openscope-history` : schéma SQLite + writer par lot (WAL) | history |
| 48 | Downsampling par paliers + tâche de rétention | history |
| 49 | `query_history` (plage + résolution) + page Historique (zoom, sélection) | history, frontend |
| 50 | Export CSV + JSON | history |
| 51 | Snapshots : création, liste, stockage | history |
| 52 | Diff de snapshots + UI comparaison avant/après | history, frontend |
| 53 | `openscope-alerts` : modèle de règles + machine à états + tests anti-flapping | alerts |
| 54 | Notifications desktop + event UI | alerts |
| 55 | Page Alertes : CRUD règles, historique, silence/ack | frontend |

## M4 — Remote (v0.4)

| # | Issue | Labels |
|---|-------|--------|
| 56 | `openscope-agent` : binaire headless JSON-Lines + build statique musl (xtask) | remote |
| 57 | Protocole versionné : hello/configure/batch/action + tests | remote |
| 58 | `openscope-remote` : connexion russh (clé, agent, mot de passe) | remote |
| 59 | Déploiement auto de l'agent via SFTP + cache + vérification version | remote |
| 60 | RemoteSource : reconnexion backoff, statuts, actions distantes | remote |
| 61 | UI machines : ajout/édition, statut dans la sidebar, dashboard par machine | frontend |
| 62 | Vue flotte : comparaison multi-machines | frontend |
| 63 | Historique + alertes multi-sources (tests) | history, alerts |

## M5 — Plugins (v0.5)

| # | Issue | Labels |
|---|-------|--------|
| 64 | `openscope-plugins` : runtime Extism + host functions + timeouts | plugins |
| 65 | Manifeste plugin.toml + permissions + validation | plugins |
| 66 | Circuit breaker + isolation des échecs | plugins |
| 67 | Widgets déclaratifs de plugins | plugins, frontend |
| 68 | Page gestion des plugins (installer, activer, permissions) | frontend |
| 69 | Plugins d'exemple + guide d'écriture | plugins, docs |
| 70 | VM via libvirt (feature `vm`) + widget | collect |

## M6 — v1.0

| # | Issue | Labels |
|---|-------|--------|
| 71 | Packaging Linux : AppImage, deb, rpm, Flathub | packaging |
| 72 | Packaging Windows (MSI, winget) + macOS (dmg, Homebrew) + signatures | packaging |
| 73 | Auto-update Tauri | packaging |
| 74 | Audit perf final + budget mémoire | perf |
| 75 | Accessibilité : clavier, contrastes, reader | frontend |
| 76 | i18n fr/en | frontend |
| 77 | Documentation utilisateur + site | docs |
