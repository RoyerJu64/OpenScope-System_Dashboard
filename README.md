# OpenScope

Tableau de bord système moderne, open source et multiplateforme (Linux, Windows, macOS), inspiré de btop, htop, nvtop, iotop et Grafana.

> ⚡ Objectif : un outil **très rapide, léger et agréable**, sans Electron.
> Stack retenue : **Rust (backend) + Tauri 2 + SolidJS (frontend)**.

## Vision

Un seul outil pour surveiller en temps réel :

| Domaine   | Métriques |
|-----------|-----------|
| CPU       | usage global & par cœur, fréquence, température, consommation (RAPL) |
| RAM       | utilisée, libre, cache, swap |
| GPU       | charge, VRAM, température, consommation, encode/decode, multi-GPU (NVIDIA/AMD/Intel) |
| Disques   | débit R/W, occupation, IOPS, SMART |
| Réseau    | up/down, connexions actives, historique, interfaces |
| Processus | tri dynamique, recherche, kill, priorité, consommation GPU |
| Docker    | conteneurs, ressources, logs |
| VM        | libvirt (Linux), optionnel |

Plus : historique persistant, exports CSV/JSON, snapshots, alertes, comparaison avant/après, monitoring distant via SSH, multi-machines, système de plugins.

## Documentation de conception

| Document | Contenu |
|----------|---------|
| [docs/01-architecture.md](docs/01-architecture.md) | Architecture globale, modules, flux de données, design patterns |
| [docs/02-choix-techniques.md](docs/02-choix-techniques.md) | Justification de la stack, dépendances, alternatives écartées |
| [docs/03-arborescence.md](docs/03-arborescence.md) | Structure du monorepo (workspace Cargo + frontend) |
| [docs/04-interfaces.md](docs/04-interfaces.md) | Traits Rust, API IPC, protocole agent distant, API plugins |
| [docs/05-mvp.md](docs/05-mvp.md) | Périmètre exact du MVP |
| [docs/06-roadmap.md](docs/06-roadmap.md) | Roadmap en 6 phases |
| [docs/07-github-tasks.md](docs/07-github-tasks.md) | Milestones, labels et découpage en issues |

## Principes directeurs

1. **Modularité stricte** — chaque domaine (collecte, historique, alertes, plugins, remote, UI) est un crate indépendant avec une interface publique minimale.
2. **La collecte ne connaît pas l'UI** — les collecteurs publient des échantillons sur un bus ; l'UI, l'historique et les alertes sont des consommateurs interchangeables.
3. **Local = distant** — le même code de collecte tourne en local et dans l'agent SSH headless (`openscoped`). Une machine distante est juste une source de plus sur le bus.
4. **Dégradation gracieuse** — si une capacité manque (pas de GPU, pas de SMART, pas de Docker), le widget correspondant est masqué, jamais d'erreur bloquante.
5. **Budget performance** — < 1 % CPU au repos à 1 Hz de rafraîchissement, < 150 Mo RAM, démarrage < 1 s.

## Développement

Prérequis : Rust stable, Node 20+, et sous Linux les dépendances Tauri
(`libwebkit2gtk-4.1-dev`, `libgtk-3-dev`).

```bash
# installer les dépendances frontend (une fois)
npm --prefix frontend install

# lancer l'app en mode dev (hot-reload frontend + backend)
cargo tauri dev          # si tauri-cli est installé (cargo install tauri-cli)
# ou sans tauri-cli :
npm --prefix frontend run dev &   # terminal 1 : vite sur :1420
cargo run -p openscope-app       # terminal 2

# tests et lints
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
npm --prefix frontend run check

# régénérer les types TypeScript après modification d'un DTO Rust
cargo xtask gen-types

# politique de dépendances (licences, advisories)
cargo deny check

# bench budget CPU
cargo test -p openscope-collect --test perf -- --ignored --nocapture
```

## Licence

MIT ou Apache-2.0 (double licence, standard Rust).
