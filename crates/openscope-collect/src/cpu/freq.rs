//! Fréquences par cœur via cpufreq (`scaling_cur_freq`, en kHz).

use std::fs;
use std::path::PathBuf;

pub(super) struct FreqReader {
    cpus_dir: PathBuf,
}

impl FreqReader {
    pub fn new() -> Self {
        Self::with_root(PathBuf::from("/sys/devices/system/cpu"))
    }

    pub fn with_root(cpus_dir: PathBuf) -> Self {
        Self { cpus_dir }
    }

    /// Au moins un cœur expose cpufreq (absent dans la plupart des VM).
    pub fn probe(&self) -> bool {
        !self.read_all().is_empty()
    }

    /// `(cœur, MHz)`, trié par cœur.
    pub fn read_all(&self) -> Vec<(u32, f64)> {
        let mut out = Vec::new();
        let Ok(entries) = fs::read_dir(&self.cpus_dir) else {
            return out;
        };
        for entry in entries.flatten() {
            let name = entry.file_name();
            let Some(core) = name
                .to_str()
                .and_then(|n| n.strip_prefix("cpu"))
                .and_then(|n| n.parse::<u32>().ok())
            else {
                continue;
            };
            let path = entry.path().join("cpufreq/scaling_cur_freq");
            if let Ok(content) = fs::read_to_string(&path) {
                if let Ok(khz) = content.trim().parse::<f64>() {
                    out.push((core, khz / 1000.0));
                }
            }
        }
        out.sort_unstable_by_key(|(core, _)| *core);
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn reads_frequencies_and_ignores_non_core_entries() {
        let root = std::env::temp_dir().join(format!("openscope-freq-{}", std::process::id()));
        for (dir, khz) in [("cpu0", "1200000"), ("cpu10", "3400000")] {
            let d = root.join(dir).join("cpufreq");
            fs::create_dir_all(&d).unwrap();
            fs::write(d.join("scaling_cur_freq"), khz).unwrap();
        }
        // cpu1 sans cpufreq, et une entrée non-cœur : ignorés sans erreur.
        fs::create_dir_all(root.join("cpu1")).unwrap();
        fs::create_dir_all(root.join("cpufreq")).unwrap();

        let freqs = FreqReader::with_root(root.clone()).read_all();
        assert_eq!(freqs, vec![(0, 1200.0), (10, 3400.0)]);

        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn probe_false_without_cpufreq() {
        let root = std::env::temp_dir().join(format!("openscope-nofreq-{}", std::process::id()));
        fs::create_dir_all(root.join("cpu0")).unwrap();
        assert!(!FreqReader::with_root(root.clone()).probe());
        let _ = fs::remove_dir_all(root);
    }
}
