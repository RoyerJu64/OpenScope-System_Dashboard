# 04 — Interfaces entre modules

Quatre frontières, chacune avec un contrat explicite :

1. Traits Rust dans `openscope-core` (entre crates backend)
2. API IPC Tauri (backend ↔ frontend)
3. Protocole agent (app ↔ `openscoped` distant)
4. API plugins (host ↔ WASM)

---

## 1. Traits `openscope-core`

### Le modèle de données

```rust
/// Identité stable d'une métrique : "cpu.usage", "gpu.0.vram.used", "disk.nvme0n1.read_bytes"
pub struct MetricId(pub Cow<'static, str>);

/// D'où vient la donnée : machine locale ou distante.
pub struct SourceId(pub String); // "local", "ssh:serveur-prod", ...

pub enum Value {
    Gauge(f64),            // valeur instantanée (%, °C, octets)
    Counter(u64),          // cumulatif, l'UI/historique calcule le taux
    Text(String),          // ex. état SMART "PASSED"
}

pub struct Sample {
    pub source: SourceId,
    pub metric: MetricId,
    pub ts: SystemTime,
    pub value: Value,
    pub labels: Labels,    // ex. {"core": "3"}, {"iface": "eth0"}, {"pid": "1234"}
}
```

### `Collector` — implémenté par chaque domaine de `openscope-collect` et par les plugins

```rust
#[async_trait]
pub trait Collector: Send + Sync {
    fn id(&self) -> &'static str;                    // "cpu", "gpu", "docker", ...
    /// Détection au démarrage : matériel/service présent et accessible ?
    async fn probe(&mut self) -> ProbeResult;        // Available(Capabilities) | Unavailable(raison)
    /// Un tick de collecte. Doit être rapide ; le lourd part en spawn_blocking.
    async fn collect(&mut self) -> anyhow::Result<Vec<Sample>>;
    /// Intervalle par défaut (l'utilisateur peut l'écraser dans la config).
    fn default_interval(&self) -> Duration;
}
```

### `MetricBus` — le seul canal entre producteurs et consommateurs

```rust
impl MetricBus {
    pub fn publish(&self, batch: Arc<Vec<Sample>>);          // 1 batch par tick de collecteur
    pub fn subscribe(&self) -> broadcast::Receiver<Arc<Vec<Sample>>>;
}
```

### `HistoryStore` — implémenté par `openscope-history::sqlite`

```rust
#[async_trait]
pub trait HistoryStore: Send + Sync {
    async fn append(&self, batch: &[Sample]) -> Result<()>;
    async fn query(&self, q: RangeQuery) -> Result<Vec<Series>>;   // metric+source+plage+résolution
    async fn snapshot(&self, name: &str) -> Result<SnapshotId>;
    async fn diff(&self, a: SnapshotId, b: SnapshotId) -> Result<SnapshotDiff>;
    async fn export(&self, q: RangeQuery, fmt: ExportFormat, path: &Path) -> Result<()>;
}
```

### `MetricSource` — abstraction locale/distante (implémentée par le scheduler local et `openscope-remote`)

```rust
#[async_trait]
pub trait MetricSource: Send + Sync {
    fn id(&self) -> SourceId;
    async fn start(&mut self, bus: MetricBus) -> Result<()>;
    async fn stop(&mut self) -> Result<()>;
    fn status(&self) -> SourceStatus;                // Connected, Connecting, Error(String)
    async fn execute(&self, action: Action) -> Result<ActionOutcome>;  // kill, renice… routé local ou SSH
}
```

---

## 2. API IPC (Tauri) — backend ↔ frontend

### Events (push backend → front)

| Event | Payload | Fréquence |
|-------|---------|-----------|
| `metrics-batch` | `{ source, ts, samples: [{metric, value, labels}] }` | 1 par tick et par source |
| `alert-transition` | `{ rule_id, state, metric, value, ts }` | sur changement d'état |
| `source-status` | `{ source, status }` | sur changement |
| `docker-event` | `{ container_id, kind }` | sur événement Docker |

### Commands (invoke front → backend) — extraits principaux

```ts
// métriques & capacités
get_capabilities(source: SourceId): Capabilities        // pilote l'affichage des widgets
set_collector_interval(source, collector, ms): void

// historique
query_history(q: RangeQuery): Series[]
export_history(q: RangeQuery, format: 'csv'|'json', path: string): void
create_snapshot(name: string): SnapshotId
list_snapshots(): SnapshotMeta[]
diff_snapshots(a: SnapshotId, b: SnapshotId): SnapshotDiff

// processus
list_processes(source, sort, filter): ProcessRow[]      // pull à 2s quand la page est visible
kill_process(source, pid, signal): ActionOutcome
set_priority(source, pid, nice): ActionOutcome

// docker
list_containers(source): ContainerRow[]
container_logs_open(source, id): ChannelId              // stream via event dédié
container_action(source, id, 'start'|'stop'|'restart'): ActionOutcome

// alertes
list_rules() / upsert_rule(rule) / delete_rule(id) / silence_rule(id, until)

// machines distantes
list_machines() / add_machine(cfg: SshConfig) / connect(source) / disconnect(source)

// layout & config
get_layout(page): WidgetLayout[] / save_layout(page, layout): void
get_config() / set_config(patch): void

// plugins
list_plugins(): PluginMeta[] / set_plugin_enabled(id, on): void
```

Tous les DTOs sont générés vers TypeScript via `ts-rs` — le front ne redéclare jamais un type à la main.

---

## 3. Protocole agent (`openscope-app` ↔ `openscoped` via SSH stdio)

JSON-Lines, une trame par ligne, versionné.

```jsonc
// app → agent
{"v":1,"cmd":"hello"}
{"v":1,"cmd":"configure","intervals":{"cpu":1000,"process":2000},"enable":["cpu","memory","disk","network","process"]}
{"v":1,"cmd":"action","action":{"kind":"kill","pid":4242,"signal":15},"req_id":"abc"}
{"v":1,"cmd":"bye"}

// agent → app
{"v":1,"ev":"hello","agent_version":"0.3.0","capabilities":{...}}
{"v":1,"ev":"batch","ts":1760000000123,"samples":[["cpu.usage",42.5,{}],["cpu.usage",91.0,{"core":"0"}]]}
{"v":1,"ev":"action_result","req_id":"abc","ok":true}
{"v":1,"ev":"error","scope":"gpu","msg":"NVML introuvable"}   // non fatal : dégradation
```

Règles : l'agent n'ouvre **aucun** port ; incompatibilité de version majeure → l'app redéploie le binaire ; perte de connexion → reconnexion avec backoff exponentiel, la source passe en `Error` dans l'UI.

---

## 4. API plugins (host ↔ WASM Extism)

### Manifeste `plugin.toml`

```toml
[plugin]
id = "ping-monitor"
name = "Ping Monitor"
version = "0.1.0"
kind = "collector"            # "collector" | "widget" | "both"

[permissions]                  # affichées à l'utilisateur à l'installation
network = ["8.8.8.8", "1.1.1.1"]
exec = []
fs_read = []

[metrics]
"ping.latency_ms" = { unit = "ms", kind = "gauge" }

[widget]                       # optionnel : widget déclaratif
type = "timeseries"
title = "Latence ping"
metrics = ["ping.latency_ms"]
```

### Fonctions exportées par le plugin (guest)

```
probe() -> ProbeResult          // JSON
collect() -> Vec<PluginSample>  // JSON ; le host stampe source/ts et republie sur le bus
configure(config_json)          // config utilisateur du plugin
```

### Host functions offertes au plugin

```
host_log(level, msg)
host_config_get(key) -> value
host_http_get(url) -> bytes     // uniquement si permission network accordée
```

Le host applique timeout (défaut 2 s par `collect()`), limite mémoire, et coupe le plugin après N échecs consécutifs (circuit breaker) — un plugin défaillant ne dégrade jamais l'app.
