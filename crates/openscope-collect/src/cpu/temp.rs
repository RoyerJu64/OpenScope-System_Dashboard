//! Températures CPU via hwmon. On choisit le capteur le plus fiable
//! présent (Intel : coretemp ; AMD : k10temp/zenpower ; ARM :
//! cpu_thermal ; à défaut acpitz), et on lit ses `tempN_input`
//! (millidegrés Celsius).

use std::fs;
use std::path::{Path, PathBuf};

/// Par ordre de priorité décroissante.
const CPU_SENSORS: &[&str] = &["coretemp", "k10temp", "zenpower", "cpu_thermal", "acpitz"];

/// Labels désignant la température « globale » du CPU, promue en
/// `cpu.temp_c` sans label.
const PACKAGE_LABELS: &[&str] = &["Package id 0", "Tctl", "Tdie"];

pub(super) struct TempReader {
    sensor: String,
    inputs: Vec<TempInput>,
}

struct TempInput {
    path: PathBuf,
    label: String,
}

impl TempReader {
    pub fn detect() -> Option<Self> {
        Self::detect_in(Path::new("/sys/class/hwmon"))
    }

    pub fn detect_in(hwmon_root: &Path) -> Option<Self> {
        let mut best: Option<(usize, PathBuf, String)> = None;
        for entry in fs::read_dir(hwmon_root).ok()?.flatten() {
            let Ok(name) = fs::read_to_string(entry.path().join("name")) else {
                continue;
            };
            let name = name.trim().to_owned();
            let Some(priority) = CPU_SENSORS.iter().position(|s| *s == name) else {
                continue;
            };
            if best.as_ref().is_none_or(|(p, _, _)| priority < *p) {
                best = Some((priority, entry.path(), name));
            }
        }
        let (_, dir, sensor) = best?;

        let mut inputs = Vec::new();
        for entry in fs::read_dir(&dir).ok()?.flatten() {
            let file = entry.file_name();
            let Some(stem) = file
                .to_str()
                .and_then(|f| f.strip_suffix("_input"))
                .filter(|s| s.starts_with("temp"))
            else {
                continue;
            };
            let label = fs::read_to_string(dir.join(format!("{stem}_label")))
                .map(|l| l.trim().to_owned())
                .unwrap_or_else(|_| stem.to_owned());
            inputs.push(TempInput {
                path: entry.path(),
                label,
            });
        }
        if inputs.is_empty() {
            return None;
        }
        inputs.sort_by(|a, b| a.path.cmp(&b.path));
        Some(Self { sensor, inputs })
    }

    pub fn sensor(&self) -> &str {
        &self.sensor
    }

    /// `(label, °C)` pour chaque sonde lisible.
    pub fn read(&self) -> Vec<(String, f64)> {
        self.inputs
            .iter()
            .filter_map(|input| {
                let raw = fs::read_to_string(&input.path).ok()?;
                let millideg: f64 = raw.trim().parse().ok()?;
                Some((input.label.clone(), millideg / 1000.0))
            })
            .collect()
    }

    /// Température « globale » : label package connu, sinon première sonde.
    pub fn package_of(readings: &[(String, f64)]) -> Option<f64> {
        readings
            .iter()
            .find(|(label, _)| PACKAGE_LABELS.contains(&label.as_str()))
            .or_else(|| readings.first())
            .map(|(_, celsius)| *celsius)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fake_hwmon(root: &Path, dir: &str, name: &str, temps: &[(&str, &str, &str)]) {
        let d = root.join(dir);
        fs::create_dir_all(&d).unwrap();
        fs::write(d.join("name"), name).unwrap();
        for (stem, label, millideg) in temps {
            if !label.is_empty() {
                fs::write(d.join(format!("{stem}_label")), label).unwrap();
            }
            fs::write(d.join(format!("{stem}_input")), millideg).unwrap();
        }
    }

    #[test]
    fn picks_highest_priority_sensor_and_reads_labels() {
        let root = std::env::temp_dir().join(format!("openscope-temp-{}", std::process::id()));
        let _ = fs::remove_dir_all(&root);
        fake_hwmon(&root, "hwmon0", "acpitz", &[("temp1", "", "51000")]);
        fake_hwmon(
            &root,
            "hwmon1",
            "coretemp",
            &[
                ("temp1", "Package id 0", "45000"),
                ("temp2", "Core 0", "43500"),
            ],
        );

        let reader = TempReader::detect_in(&root).expect("capteur attendu");
        assert_eq!(reader.sensor(), "coretemp");
        let readings = reader.read();
        assert_eq!(readings.len(), 2);
        assert!(readings.contains(&("Package id 0".to_owned(), 45.0)));
        assert!(readings.contains(&("Core 0".to_owned(), 43.5)));
        assert_eq!(TempReader::package_of(&readings), Some(45.0));

        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn no_cpu_sensor_means_none() {
        let root = std::env::temp_dir().join(format!("openscope-notemp-{}", std::process::id()));
        let _ = fs::remove_dir_all(&root);
        fake_hwmon(&root, "hwmon0", "nvme", &[("temp1", "", "35000")]);
        assert!(TempReader::detect_in(&root).is_none());
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn package_falls_back_to_first_reading() {
        let readings = vec![("Core 0".to_owned(), 40.0), ("Core 1".to_owned(), 42.0)];
        assert_eq!(TempReader::package_of(&readings), Some(40.0));
    }
}
