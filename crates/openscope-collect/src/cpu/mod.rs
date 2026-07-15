//! Collecteur CPU (issue #12) : usage global + par cœur.
//!
//! Métriques émises :
//! - `cpu.usage` (gauge %) — global sans label, par cœur avec `{"core": "N"}`
//!
//! Fréquences, températures et RAPL arrivent avec l'issue #13.

// Module entier réservé à Linux (gate dans lib.rs) : le format de
// /proc/stat n'existe pas ailleurs. Windows/macOS : issues #43/#44.
mod linux;

pub use linux::CpuCollector;

/// Temps cumulés d'un CPU (en ticks USER_HZ), réduits à busy/idle.
/// L'usage se calcule par delta entre deux lectures :
/// `busy_delta / (busy_delta + idle_delta)`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct CpuTimes {
    /// `None` = ligne agrégée (`cpu `), `Some(n)` = cœur `cpuN`.
    pub core: Option<u32>,
    pub busy: u64,
    pub idle: u64,
}

impl CpuTimes {
    /// Usage en % entre deux lectures ; `None` si aucun tick ne s'est
    /// écoulé (lectures trop rapprochées) ou si le compteur a reculé
    /// (reset kernel, hotplug de cœur).
    pub(crate) fn usage_since(&self, prev: &CpuTimes) -> Option<f64> {
        let busy = self.busy.checked_sub(prev.busy)?;
        let idle = self.idle.checked_sub(prev.idle)?;
        let total = busy + idle;
        if total == 0 {
            return None;
        }
        Some(busy as f64 / total as f64 * 100.0)
    }
}

/// Parse le contenu de `/proc/stat` : la ligne agrégée `cpu ` puis une
/// ligne `cpuN` par cœur. Les champs sont, dans l'ordre : user, nice,
/// system, idle, iowait, irq, softirq, steal (guest/guest_nice sont déjà
/// comptés dans user/nice, on les ignore).
pub(crate) fn parse_proc_stat(content: &str) -> Vec<CpuTimes> {
    let mut out = Vec::new();
    for line in content.lines() {
        let Some(rest) = line.strip_prefix("cpu") else {
            continue;
        };
        let mut fields = rest.split_whitespace();
        // `cpuN ...` : le premier token est l'identifiant du cœur ;
        // `cpu  ...` (agrégat) : les tokens sont directement les ticks.
        let core: Option<u32> = match rest.chars().next() {
            Some(c) if c.is_ascii_digit() => fields.next().and_then(|id| id.parse().ok()),
            _ => None,
        };

        let mut ticks = [0u64; 8];
        for slot in ticks.iter_mut() {
            *slot = fields.next().and_then(|f| f.parse().ok()).unwrap_or(0);
        }
        let [user, nice, system, idle, iowait, irq, softirq, steal] = ticks;

        out.push(CpuTimes {
            core,
            busy: user + nice + system + irq + softirq + steal,
            idle: idle + iowait,
        });
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    const STAT_T0: &str = "\
cpu  100 0 50 800 50 0 0 0 0 0
cpu0 60 0 30 400 10 0 0 0 0 0
cpu1 40 0 20 400 40 0 0 0 0 0
intr 12345
ctxt 6789
";

    const STAT_T1: &str = "\
cpu  150 0 70 860 70 0 0 0 0 0
cpu0 100 0 45 415 20 0 0 0 0 0
cpu1 50 0 25 445 50 0 0 0 0 0
";

    #[test]
    fn parse_extracts_aggregate_and_cores() {
        let times = parse_proc_stat(STAT_T0);
        assert_eq!(times.len(), 3);
        assert_eq!(times[0].core, None);
        assert_eq!(times[1].core, Some(0));
        assert_eq!(times[2].core, Some(1));
        // agrégat : busy = 100+0+50 = 150, idle = 800+50 = 850
        assert_eq!(times[0].busy, 150);
        assert_eq!(times[0].idle, 850);
    }

    #[test]
    fn usage_is_computed_from_deltas() {
        let t0 = parse_proc_stat(STAT_T0);
        let t1 = parse_proc_stat(STAT_T1);
        // agrégat : busy Δ = 220-150 = 70, idle Δ = 930-850 = 80 → 70/150
        let global = t1[0].usage_since(&t0[0]).unwrap();
        assert!((global - 70.0 / 150.0 * 100.0).abs() < 1e-9);
        // cpu0 : busy Δ = 145-90 = 55, idle Δ = 435-410 = 25 → 55/80
        let core0 = t1[1].usage_since(&t0[1]).unwrap();
        assert!((core0 - 55.0 / 80.0 * 100.0).abs() < 1e-9);
    }

    #[test]
    fn usage_handles_no_elapsed_ticks_and_counter_reset() {
        let t = parse_proc_stat(STAT_T0);
        assert_eq!(t[0].usage_since(&t[0]), None); // aucun tick écoulé
        let t0 = parse_proc_stat(STAT_T0);
        let t1 = parse_proc_stat(STAT_T1);
        assert_eq!(t0[0].usage_since(&t1[0]), None); // compteur qui recule
    }

    #[test]
    fn parse_tolerates_missing_fields() {
        // Vieux kernels : moins de colonnes. Les champs absents valent 0.
        let times = parse_proc_stat("cpu  10 0 5 100\ncpu0 10 0 5 100\n");
        assert_eq!(times[0].busy, 15);
        assert_eq!(times[0].idle, 100);
    }
}
