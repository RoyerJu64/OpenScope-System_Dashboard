//! Collecteur mémoire (issue #14), depuis `/proc/meminfo`.
//!
//! Métriques émises (gauges, octets sauf mention) :
//! - `mem.total_bytes`, `mem.used_bytes`, `mem.available_bytes`,
//!   `mem.free_bytes`, `mem.cached_bytes`
//! - `mem.used_pct` (gauge %) — pratique pour les graphes
//! - `swap.total_bytes`, `swap.used_bytes`
//!
//! « used » suit la définition de `free(1)` moderne :
//! `total - available` (ce que les applications ne peuvent pas récupérer).

mod linux;

pub use linux::MemoryCollector;

/// Valeurs de /proc/meminfo utiles, en octets.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub(crate) struct MemInfo {
    pub total: u64,
    pub free: u64,
    pub available: u64,
    pub buffers: u64,
    pub cached: u64,
    pub s_reclaimable: u64,
    pub swap_total: u64,
    pub swap_free: u64,
}

impl MemInfo {
    pub(crate) fn used(&self) -> u64 {
        self.total.saturating_sub(self.available)
    }

    /// Cache au sens large : page cache + buffers + slab récupérable
    /// (la définition de la colonne « buff/cache » de free(1)).
    pub(crate) fn cache(&self) -> u64 {
        self.cached + self.buffers + self.s_reclaimable
    }

    pub(crate) fn swap_used(&self) -> u64 {
        self.swap_total.saturating_sub(self.swap_free)
    }
}

/// Parse `/proc/meminfo` : lignes `Clé:   valeur kB` (les valeurs sont
/// toujours en kibioctets malgré le « kB »).
pub(crate) fn parse_meminfo(content: &str) -> MemInfo {
    let mut info = MemInfo::default();
    for line in content.lines() {
        let Some((key, rest)) = line.split_once(':') else {
            continue;
        };
        let slot = match key {
            "MemTotal" => &mut info.total,
            "MemFree" => &mut info.free,
            "MemAvailable" => &mut info.available,
            "Buffers" => &mut info.buffers,
            "Cached" => &mut info.cached,
            "SReclaimable" => &mut info.s_reclaimable,
            "SwapTotal" => &mut info.swap_total,
            "SwapFree" => &mut info.swap_free,
            _ => continue,
        };
        if let Some(kib) = rest
            .split_whitespace()
            .next()
            .and_then(|v| v.parse::<u64>().ok())
        {
            *slot = kib * 1024;
        }
    }
    info
}

#[cfg(test)]
mod tests {
    use super::*;

    const MEMINFO: &str = "\
MemTotal:       16000000 kB
MemFree:         2000000 kB
MemAvailable:    9000000 kB
Buffers:          500000 kB
Cached:          5000000 kB
SwapCached:            0 kB
SReclaimable:     300000 kB
SwapTotal:       4000000 kB
SwapFree:        3500000 kB
";

    #[test]
    fn parses_and_derives() {
        let info = parse_meminfo(MEMINFO);
        assert_eq!(info.total, 16_000_000 * 1024);
        assert_eq!(info.available, 9_000_000 * 1024);
        assert_eq!(info.used(), 7_000_000 * 1024);
        assert_eq!(info.cache(), (5_000_000 + 500_000 + 300_000) * 1024);
        assert_eq!(info.swap_used(), 500_000 * 1024);
    }

    #[test]
    fn tolerates_missing_keys_and_garbage() {
        let info = parse_meminfo("MemTotal: 1000 kB\nGarbage\nFoo: bar\n");
        assert_eq!(info.total, 1000 * 1024);
        assert_eq!(info.available, 0);
        assert_eq!(info.used(), info.total); // available absent → tout compté utilisé
    }
}
