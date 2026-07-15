# 02 — Choix techniques

## Décision principale : Tauri 2 + SolidJS, pas egui, pas Electron

| Critère | Tauri 2 + SolidJS ✅ | egui | Electron |
|---------|---------------------|------|----------|
| Poids binaire | ~10 Mo | ~15 Mo | ~150 Mo+ |
| RAM au repos | 60–120 Mo | 30–60 Mo | 300 Mo+ |
| UI « moderne » (sidebar, widgets drag & drop, animations, thèmes) | Excellent — c'est du web | Possible mais coûteux : immediate mode, tout est à construire à la main | Excellent |
| Graphes fluides haute densité | uPlot (canvas), éprouvé | egui_plot, correct | idem web |
| Écosystème backend | Rust natif | Rust natif | Node (il faudrait du N-API) |
| Accessibilité, i18n, copier/coller riche | natif web | faible | natif web |

**Pourquoi pas egui ?** Pour un TUI-like (btop), egui serait parfait. Mais le cahier des charges demande une UI type Grafana : grille de widgets déplaçables, sidebar, thèmes, animations discrètes. En immediate mode, chaque composant (drag & drop de grille, tooltips riches, tableaux triables avec recherche) est un développement significatif. Le webview de Tauri utilise le moteur du système (WebKitGTK/WebView2/WKWebView) — ce n'est **pas** Electron : pas de Chromium embarqué, pas de process Node.

**Pourquoi SolidJS et pas React/Svelte/Vue ?** Réactivité à granularité fine sans virtual DOM : idéal pour des mises à jour de métriques chaque seconde sans re-render en cascade. Bundle minuscule (~7 ko). Svelte 5 serait un second choix acceptable ; React est écarté (VDOM = travail inutile à 1 Hz sur des dizaines de widgets).

**Risque assumé** : WebKitGTK sur Linux est le maillon faible de Tauri (rendu parfois moins performant que Chromium). Mitigation : uPlot est en canvas 2D pur (très bien supporté), et le budget perf est vérifié en CI sur Linux.

## Dépendances backend (Rust)

### Fondation
| Crate | Rôle | Justification |
|-------|------|---------------|
| `tokio` | runtime async | standard de facto ; broadcast channel = notre bus |
| `serde` + `serde_json` | sérialisation | DTOs IPC, protocole agent, exports |
| `thiserror` / `anyhow` | erreurs | thiserror dans les libs, anyhow dans les binaires |
| `tracing` + `tracing-subscriber` | logs structurés | indispensable pour diagnostiquer les collecteurs |
| `figment` | configuration | fusion fichier TOML + env + défauts |

### Collecte
| Crate | Rôle | Notes |
|-------|------|-------|
| `sysinfo` | CPU/RAM/processus/disques multiplateforme | base portable ; complété par des sources natives quand il est trop pauvre |
| `procfs` | Linux : détails fins (pressure stall, diskstats, sockets) | Linux only, derrière `#[cfg]` |
| `nvml-wrapper` | GPU NVIDIA (charge, VRAM, temp, power, enc/dec, par-processus) | la référence, même source que nvtop |
| lecture sysfs directe | GPU AMD (`/sys/class/drm/.../amdgpu`) & Intel | pas de crate mature : module maison `gpu/amd.rs`, `gpu/intel.rs` |
| `smartctl` (binaire externe, `--json`) | SMART | libatasmart est Linux-only et limitée ; smartctl -j est multiplateforme et complet. Détecté via `probe()` |
| `netlink` (`neli`) | connexions actives Linux | équivalent `ss` ; fallback parsing `/proc/net/tcp` |
| `bollard` | Docker API | async, complet (stats, logs streaming, events) |
| `virt` | libvirt | optionnel, feature flag `vm`, Linux only |

### Historique / alertes / remote / plugins
| Crate | Rôle | Notes |
|-------|------|-------|
| `rusqlite` (bundled) | persistance historique | zéro dépendance système ; WAL mode |
| `csv` | export CSV | |
| `russh` + `russh-sftp` | SSH client pur Rust | pas de dépendance à OpenSSL/libssh ; déploiement agent via SFTP |
| `extism` | runtime plugins WASM | sandbox, host functions, multi-langage côté auteurs |
| `notify-rust` / plugin Tauri notification | notifications desktop | alertes |

### Frontend
| Paquet | Rôle |
|--------|------|
| `solid-js` | framework UI |
| `uplot` | graphes canvas ultra-rapides |
| `@tauri-apps/api` | IPC |
| `vite` + `typescript` | build |
| CSS vanilla + custom properties | theming (pas de Tailwind : contrôle fin du poids et du design system) |

### Outillage
- `cargo-deny` (licences + audit), `clippy` pedantic, `rustfmt`
- CI GitHub Actions : build + tests Linux/Windows/macOS, lint front (eslint, tsc)
- `tauri-action` pour les releases signées multiplateforme

## Choix de conception notables

1. **SQLite plutôt qu'un format maison ou InfluxDB embarqué** : requêtable (la page Historique fait de vraies requêtes de plages), fiable, un seul fichier, et le downsampling par paliers maîtrise la taille (~50 Mo/an/machine pour ~200 séries).
2. **Agent SSH sur stdio plutôt qu'un daemon avec port** : rien à installer ni ouvrir côté serveur, modèle de sécurité = celui de SSH. C'est le modèle « mosh-like » ; le binaire statique musl (~3 Mo) est poussé via SFTP au premier usage puis mis en cache dans `~/.cache/openscope/`.
3. **WASM (Extism) plutôt que dylibs pour les plugins** : pas d'ABI Rust instable, sandbox réelle, un plugin peut être écrit en Rust/Go/JS/Python. Les dylibs natives sont écartées (crash du plugin = crash de l'app, aucune isolation).
4. **smartctl en sous-processus plutôt qu'une lib** : SMART nécessite souvent des privilèges ; passer par le binaire permet une élévation ciblée (polkit/UAC) sans élever toute l'application.
5. **Un event IPC par tick, pas par métrique** : la frontière webview est le point de contention n°1 de Tauri ; on n'y fait passer que des batches compacts.
6. **Élévation de privilèges** : jamais l'app entière en root. Actions privilégiées (kill d'un processus d'un autre utilisateur, renice négatif, SMART) via polkit (Linux), UAC (Windows), SMJobBless/askpass (macOS), au cas par cas.
