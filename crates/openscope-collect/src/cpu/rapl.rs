//! Consommation du CPU via RAPL (powercap) : les compteurs d'énergie
//! `energy_uj` (µJ) des packages `intel-rapl:N` — présents aussi sur AMD
//! moderne. Souvent illisibles sans privilèges : dans ce cas `detect`
//! retourne `None` et la métrique est simplement absente.

use std::fs;
use std::path::{Path, PathBuf};
use std::time::Instant;

pub(super) struct RaplReader {
    packages: Vec<PathBuf>,
    prev: Option<(u64, Instant)>,
}

impl RaplReader {
    pub fn detect() -> Option<Self> {
        Self::detect_in(Path::new("/sys/class/powercap"))
    }

    pub fn detect_in(root: &Path) -> Option<Self> {
        let mut packages = Vec::new();
        for entry in fs::read_dir(root).ok()?.flatten() {
            let name = entry.file_name();
            // Packages uniquement (`intel-rapl:0`), pas les sous-zones
            // (`intel-rapl:0:0`) qui y sont déjà comptées.
            let is_package = name
                .to_str()
                .and_then(|n| n.strip_prefix("intel-rapl:"))
                .is_some_and(|suffix| !suffix.is_empty() && suffix.bytes().all(|b| b.is_ascii_digit()));
            if !is_package {
                continue;
            }
            let energy = entry.path().join("energy_uj");
            // Vérifie la lisibilité tout de suite (permissions).
            if fs::read_to_string(&energy)
                .ok()
                .is_some_and(|s| s.trim().parse::<u64>().is_ok())
            {
                packages.push(energy);
            }
        }
        if packages.is_empty() {
            return None;
        }
        packages.sort();
        Some(Self {
            packages,
            prev: None,
        })
    }

    /// Puissance moyenne (W) depuis le dernier appel. `None` au premier
    /// appel, si le compteur a bouclé, ou si une lecture échoue.
    pub fn read_watts(&mut self) -> Option<f64> {
        let mut total: u64 = 0;
        for path in &self.packages {
            let raw = fs::read_to_string(path).ok()?;
            total = total.checked_add(raw.trim().parse().ok()?)?;
        }
        let now = Instant::now();
        let prev = self.prev.replace((total, now))?;
        let delta_uj = total.checked_sub(prev.0)?; // wrap → on saute un tick
        let dt = now.duration_since(prev.1).as_secs_f64();
        if dt <= 0.0 {
            return None;
        }
        Some(delta_uj as f64 / 1e6 / dt)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    #[test]
    fn computes_watts_from_energy_delta() {
        let root = std::env::temp_dir().join(format!("openscope-rapl-{}", std::process::id()));
        let _ = fs::remove_dir_all(&root);
        let pkg = root.join("intel-rapl:0");
        fs::create_dir_all(&pkg).unwrap();
        fs::write(pkg.join("energy_uj"), "1000000").unwrap();
        // Sous-zone et entrée étrangère : ignorées.
        fs::create_dir_all(root.join("intel-rapl:0:0")).unwrap();
        fs::create_dir_all(root.join("dtpm")).unwrap();

        let mut reader = RaplReader::detect_in(&root).expect("package lisible");
        assert_eq!(reader.read_watts(), None, "premier appel sans référence");

        std::thread::sleep(Duration::from_millis(50));
        fs::write(pkg.join("energy_uj"), "2000000").unwrap(); // +1 J
        let watts = reader.read_watts().expect("delta attendu");
        // 1 J sur ~50 ms → ~20 W ; bornes larges pour absorber le jitter.
        assert!((1.0..=100.0).contains(&watts), "watts hors bornes : {watts}");
    }

    #[test]
    fn unreadable_or_absent_means_none() {
        let root = std::env::temp_dir().join(format!("openscope-norapl-{}", std::process::id()));
        let _ = fs::remove_dir_all(&root);
        fs::create_dir_all(root.join("intel-rapl:0")).unwrap(); // pas d'energy_uj
        assert!(RaplReader::detect_in(&root).is_none());
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn counter_wrap_skips_one_tick() {
        let root = std::env::temp_dir().join(format!("openscope-wrap-{}", std::process::id()));
        let _ = fs::remove_dir_all(&root);
        let pkg = root.join("intel-rapl:0");
        fs::create_dir_all(&pkg).unwrap();
        fs::write(pkg.join("energy_uj"), "5000000").unwrap();
        let mut reader = RaplReader::detect_in(&root).unwrap();
        let _ = reader.read_watts();
        fs::write(pkg.join("energy_uj"), "100").unwrap(); // wrap
        assert_eq!(reader.read_watts(), None);
        let _ = fs::remove_dir_all(root);
    }
}
