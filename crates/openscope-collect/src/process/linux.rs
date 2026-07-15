use std::collections::HashMap;
use std::fs;
use std::os::unix::fs::MetadataExt;
use std::time::Instant;

use super::{parse_stat_line, ProcessRow};

/// Snapshot de la table des processus, avec deltas CPU entre deux appels.
/// À conserver entre les appels (le premier snapshot a des cpu_pct à 0).
pub struct ProcessTable {
    /// pid → ticks cumulés au snapshot précédent.
    prev_ticks: HashMap<i32, u64>,
    prev_at: Option<Instant>,
    users: UserCache,
    page_size: u64,
    clk_tck: f64,
}

impl ProcessTable {
    pub fn new() -> Self {
        Self {
            prev_ticks: HashMap::new(),
            prev_at: None,
            users: UserCache::default(),
            page_size: unsafe { libc::sysconf(libc::_SC_PAGESIZE) }.max(4096) as u64,
            clk_tck: unsafe { libc::sysconf(libc::_SC_CLK_TCK) }.max(1) as f64,
        }
    }

    pub fn snapshot(&mut self) -> Vec<ProcessRow> {
        let now = Instant::now();
        let dt = self
            .prev_at
            .replace(now)
            .map(|at| now.duration_since(at).as_secs_f64());

        let mut rows = Vec::new();
        let mut ticks_seen = HashMap::new();

        let Ok(entries) = fs::read_dir("/proc") else {
            return rows;
        };
        for entry in entries.flatten() {
            let Some(pid) = entry
                .file_name()
                .to_str()
                .and_then(|n| n.parse::<i32>().ok())
            else {
                continue;
            };
            // Le processus peut mourir entre le readdir et les lectures :
            // toute erreur ici = processus ignoré, jamais d'échec global.
            let Ok(stat_raw) = fs::read_to_string(entry.path().join("stat")) else {
                continue;
            };
            let Some(stat) = parse_stat_line(&stat_raw) else {
                continue;
            };

            let cpu_pct = match (dt, self.prev_ticks.get(&pid)) {
                (Some(dt), Some(prev)) if dt > 0.0 => stat
                    .ticks
                    .checked_sub(*prev)
                    .map_or(0.0, |delta| delta as f64 / self.clk_tck / dt * 100.0),
                _ => 0.0,
            };
            ticks_seen.insert(pid, stat.ticks);

            let uid = fs::metadata(entry.path()).map(|m| m.uid()).unwrap_or(0);
            let cmdline = fs::read(entry.path().join("cmdline"))
                .map(|raw| {
                    String::from_utf8_lossy(&raw)
                        .trim_end_matches('\0')
                        .replace('\0', " ")
                })
                .unwrap_or_default();

            rows.push(ProcessRow {
                pid,
                ppid: stat.ppid,
                name: stat.name,
                user: self.users.name(uid),
                state: stat.state,
                cpu_pct,
                rss_bytes: stat.rss_pages * self.page_size,
                cmdline,
            });
        }

        // Remplace (et purge les pids morts) plutôt que d'accumuler.
        self.prev_ticks = ticks_seen;
        rows
    }
}

impl Default for ProcessTable {
    fn default() -> Self {
        Self::new()
    }
}

/// uid → nom, depuis /etc/passwd (parsé une fois, uid inconnu = numéro).
#[derive(Default)]
struct UserCache {
    names: Option<HashMap<u32, String>>,
}

impl UserCache {
    fn name(&mut self, uid: u32) -> String {
        let names = self.names.get_or_insert_with(|| {
            let mut map = HashMap::new();
            if let Ok(passwd) = fs::read_to_string("/etc/passwd") {
                for line in passwd.lines() {
                    let mut fields = line.split(':');
                    let (Some(name), _, Some(id)) = (fields.next(), fields.next(), fields.next())
                    else {
                        continue;
                    };
                    if let Ok(id) = id.parse::<u32>() {
                        map.insert(id, name.to_owned());
                    }
                }
            }
            map
        });
        names.get(&uid).cloned().unwrap_or_else(|| uid.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn snapshot_contains_ourselves_with_rss() {
        let mut table = ProcessTable::new();
        let rows = table.snapshot();
        let me = rows
            .iter()
            .find(|r| r.pid == std::process::id() as i32)
            .expect("notre propre processus doit apparaître");
        assert!(me.rss_bytes > 0);
        assert!(!me.user.is_empty());
        assert_eq!(me.cpu_pct, 0.0, "premier snapshot : pas de delta");
    }

    #[test]
    fn second_snapshot_measures_cpu_of_a_busy_loop() {
        let mut table = ProcessTable::new();
        table.snapshot();

        // Brûle du CPU ~150 ms pour être mesurable au second snapshot.
        let until = Instant::now() + std::time::Duration::from_millis(150);
        let mut x: u64 = 0;
        while Instant::now() < until {
            x = x.wrapping_mul(31).wrapping_add(7);
        }
        std::hint::black_box(x);

        let rows = table.snapshot();
        let me = rows
            .iter()
            .find(|r| r.pid == std::process::id() as i32)
            .unwrap();
        assert!(
            me.cpu_pct > 10.0,
            "boucle chaude attendue dans les deltas : {}",
            me.cpu_pct
        );
    }
}
