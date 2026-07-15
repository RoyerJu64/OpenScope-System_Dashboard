# 06 — Roadmap

Six phases. Chaque phase se termine par une release taguée et utilisable.

## Phase 0 — Fondations (~1 semaine)
- Workspace Cargo + squelette des crates + frontend Vite/Solid + Tauri 2 branchés.
- `openscope-core` complet : Sample, traits, MetricBus, tests.
- CI 3 OS, lint, cargo-deny, génération des types TS (`ts-rs` + xtask).
- **Livrable** : app vide qui affiche un compteur factice publié sur le bus et rendu dans un graphe uPlot — la chaîne complète collecteur → bus → IPC → widget est validée.

## Phase 1 — MVP local Linux (~5 semaines) → v0.1
Voir [05-mvp.md](05-mvp.md). CPU, RAM, disque, réseau, processus ; grille de widgets persistée ; dark mode ; rafraîchissement configurable.

## Phase 2 — Matériel complet & parité OS (~5 semaines) → v0.2
- GPU : NVIDIA (NVML : charge, VRAM, temp, power, enc/dec, par-processus) puis AMD (sysfs) puis Intel ; multi-GPU ; colonne GPU dans la table des processus.
- SMART (smartctl -j) + page Disques enrichie (IOPS détaillés, latence).
- Docker : liste conteneurs, CPU/RAM/réseau par conteneur, logs streamés, start/stop/restart.
- Connexions réseau actives (netlink) + page Réseau complète.
- Collecteurs Windows & macOS au niveau du MVP Linux (CPU/RAM/disque/réseau/processus) ; élévation ciblée (polkit/UAC) pour kill/renice privilégiés.
- Light mode + polissage UI/animations.

## Phase 3 — Historique & alertes (~4 semaines) → v0.3
- `openscope-history` SQLite : downsampling par paliers, rétention configurable.
- Page Historique : navigation temporelle, zoom, sélection de métriques.
- Exports CSV / JSON.
- Snapshots + **comparaison avant/après** (diff visuel de deux snapshots : métriques et processus).
- `openscope-alerts` : règles (seuil + durée + sévérité), notifications desktop, page Alertes, silence/ack.

## Phase 4 — Remote & multi-machines (~5 semaines) → v0.4
- `openscope-agent` binaire statique musl + protocole JSON-Lines versionné.
- `openscope-remote` : connexion SSH (clé, agent, mot de passe), déploiement auto de l'agent, reconnexion.
- UI : gestion des machines dans la sidebar, dashboard par machine, vue « flotte » comparant les machines.
- Historique et alertes multi-sources (déjà structurel grâce à `SourceId`).

## Phase 5 — Plugins & extension (~4 semaines) → v0.5
- `openscope-plugins` : runtime Extism, manifeste + permissions, circuit breaker.
- Widgets déclaratifs de plugins ; page de gestion des plugins.
- 2 plugins d'exemple (ping-monitor, sensor custom) + guide d'écriture de plugin.
- VM via libvirt (Linux, feature flag).

## Phase 6 — v1.0 (~3 semaines)
- Packaging : AppImage/deb/rpm + Flathub, MSI/winget, dmg/Homebrew ; binaires signés ; auto-update Tauri.
- Audit perf final, accessibilité (navigation clavier, contrastes), i18n fr/en.
- Documentation utilisateur, site vitrine, templates d'issues communautaires.

### Jalons de risque à traiter tôt
1. **Perf du webview Linux (WebKitGTK)** — validée dès la Phase 0 par le prototype de graphe.
2. **GPU AMD/Intel sans lib mature** — spike de 2 jours en début de Phase 2 avant d'engager la conception.
3. **Élévation de privilèges propre sur 3 OS** — spike en Phase 2.
