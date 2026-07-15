# 03 — Arborescence du projet

Monorepo : workspace Cargo à la racine + frontend Vite dans `frontend/`.

```
SystemDashboard/
├── Cargo.toml                      # workspace : members = crates/*, resolver 2
├── rust-toolchain.toml
├── deny.toml                       # cargo-deny (licences, advisories)
├── .github/
│   ├── workflows/
│   │   ├── ci.yml                  # build + test + clippy + tsc, matrice 3 OS
│   │   └── release.yml             # tauri-action, binaires signés
│   └── ISSUE_TEMPLATE/
├── docs/                           # cette documentation + ADRs futurs
│
├── crates/
│   ├── openscope-core/                 # types, traits, bus — ZÉRO dépendance système
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── sample.rs           # Sample, MetricId, Value, Labels
│   │       ├── source.rs           # SourceId, MetricSource, Capabilities
│   │       ├── collector.rs        # trait Collector
│   │       ├── bus.rs              # MetricBus
│   │       ├── history.rs          # trait HistoryStore, TimeRange, Resolution
│   │       └── error.rs
│   │
│   ├── openscope-collect/
│   │   └── src/
│   │       ├── lib.rs              # registry + probe_all()
│   │       ├── scheduler.rs        # ticks par collecteur, intervalles indépendants
│   │       ├── cpu/                # mod.rs + linux.rs / windows.rs / macos.rs
│   │       ├── memory/
│   │       ├── gpu/                # mod.rs, nvidia.rs (NVML), amd.rs (sysfs), intel.rs
│   │       ├── disk/               # io.rs, usage.rs, smart.rs (smartctl -j)
│   │       ├── network/            # throughput.rs, connections.rs
│   │       ├── process/            # snapshot.rs, actions.rs (kill/renice)
│   │       ├── docker.rs           # bollard
│   │       └── vm.rs               # libvirt, feature "vm"
│   │
│   ├── openscope-history/
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── ring.rs             # fenêtre chaude en mémoire
│   │       ├── sqlite.rs           # impl HistoryStore
│   │       ├── downsample.rs       # paliers 1s/10s/1min/15min
│   │       ├── export.rs           # CSV / JSON
│   │       └── snapshot.rs         # capture + diff (comparaison avant/après)
│   │
│   ├── openscope-alerts/
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── rule.rs             # définition déclarative des règles
│   │       ├── engine.rs           # machine à états Ok/Pending/Firing/Resolved
│   │       └── sinks.rs            # notification desktop, event UI, webhook
│   │
│   ├── openscope-remote/
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── ssh.rs              # russh : connexion, auth (agent, clé, mdp)
│   │       ├── deploy.rs           # push binaire agent via SFTP + cache
│   │       └── source.rs           # RemoteSource : impl MetricSource
│   │
│   ├── openscope-agent/                # binaire "openscoped" (headless, statique musl)
│   │   └── src/main.rs             # openscope-collect → JSON-Lines sur stdout
│   │
│   ├── openscope-plugins/
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── manifest.rs         # plugin.toml : permissions, métriques, widget
│   │       ├── host.rs             # runtime Extism + host functions
│   │       └── widget_spec.rs      # description déclarative des widgets plugin
│   │
│   └── openscope-app/                  # binaire Tauri
│       ├── tauri.conf.json
│       ├── capabilities/           # permissions Tauri 2
│       ├── icons/
│       └── src/
│           ├── main.rs             # bootstrap : config, runtime, câblage des modules
│           ├── state.rs            # AppState (handles vers chaque module)
│           ├── config.rs           # figment : ~/.config/openscope/config.toml
│           ├── forwarder.rs        # bus → events IPC (batch par tick)
│           └── commands/           # 1 fichier par domaine IPC
│               ├── metrics.rs      #   subscribe, get_capabilities
│               ├── history.rs      #   query, export, snapshots
│               ├── process.rs      #   list, kill, renice
│               ├── docker.rs       #   containers, logs
│               ├── alerts.rs       #   CRUD règles
│               ├── remote.rs       #   machines CRUD, connect/disconnect
│               ├── plugins.rs      #   list, enable, disable
│               └── layout.rs       #   sauvegarde disposition widgets
│
├── frontend/
│   ├── package.json
│   ├── vite.config.ts
│   ├── index.html
│   └── src/
│       ├── main.tsx
│       ├── ipc/                    # wrappers typés invoke/listen + types DTO
│       │   ├── client.ts
│       │   └── types.ts            # miroir TS des DTOs Rust (généré, cf. note)
│       ├── stores/                 # 1 store Solid par domaine + ring buffers client
│       │   ├── metrics.ts
│       │   ├── processes.ts
│       │   ├── machines.ts
│       │   └── settings.ts
│       ├── layout/
│       │   ├── Shell.tsx           # sidebar + zone de contenu
│       │   ├── Sidebar.tsx
│       │   └── grid/               # moteur de grille drag & drop maison
│       │       ├── Grid.tsx
│       │       └── useDrag.ts
│       ├── widgets/                # 1 widget = 1 composant autonome
│       │   ├── registry.ts         # catalogue { type → composant, taille défaut }
│       │   ├── CpuUsage.tsx        # global + par cœur
│       │   ├── CpuFreqTemp.tsx
│       │   ├── MemoryGauge.tsx
│       │   ├── GpuPanel.tsx
│       │   ├── DiskIo.tsx
│       │   ├── DiskUsage.tsx
│       │   ├── NetworkThroughput.tsx
│       │   ├── ProcessTable.tsx
│       │   └── DockerContainers.tsx
│       ├── pages/
│       │   ├── Dashboard.tsx       # la grille de widgets
│       │   ├── Cpu.tsx  Gpu.tsx  Disks.tsx  Network.tsx
│       │   ├── Processes.tsx  Docker.tsx
│       │   ├── History.tsx         # requêtes de plages, exports, snapshots
│       │   ├── Alerts.tsx
│       │   └── Settings.tsx
│       ├── charts/
│       │   ├── TimeSeries.tsx      # wrapper uPlot thémé
│       │   ├── Gauge.tsx
│       │   └── Sparkline.tsx
│       └── theme/
│           ├── tokens.css          # custom properties (couleurs, espacements)
│           ├── dark.css
│           └── light.css
│
├── plugins-examples/               # plugins de démonstration (Rust → WASM)
│   └── ping-monitor/
└── xtask/                          # cargo xtask : build agent musl, gen types TS
```

**Note — types partagés Rust ↔ TS** : les DTOs de `openscope-app/src/commands/` sont annotés avec `ts-rs` pour générer `frontend/src/ipc/types.ts` à la compilation (`cargo xtask gen-types`). Une seule source de vérité, pas de dérive.

**Règle de dépendance entre crates** (vérifiée en CI par un test) :

```
openscope-core  ←  openscope-collect, openscope-history, openscope-alerts, openscope-remote, openscope-plugins
openscope-collect  ←  openscope-agent, openscope-app
tous  ←  openscope-app (seul point d'assemblage)
```

Aucune autre arête n'est autorisée. `openscope-collect` ne connaît ni l'historique ni l'UI ; `openscope-history` ne sait pas d'où viennent les samples.
