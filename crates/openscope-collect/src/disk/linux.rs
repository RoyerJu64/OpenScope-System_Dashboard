use std::collections::BTreeMap;
use std::ffi::CString;
use std::time::{Duration, Instant};

use async_trait::async_trait;
use openscope_core::{source::CollectorCapability, Collector, ProbeResult, Sample, SourceId};

use super::{parse_diskstats, parse_mounts, DiskStats, SECTOR_BYTES};

const PROC_DISKSTATS: &str = "/proc/diskstats";
const PROC_MOUNTS: &str = "/proc/mounts";
const SYS_BLOCK: &str = "/sys/block";

pub struct DiskCollector {
    prev: Option<(Vec<DiskStats>, Instant)>,
}

impl DiskCollector {
    pub fn new() -> Self {
        Self { prev: None }
    }
}

impl Default for DiskCollector {
    fn default() -> Self {
        Self::new()
    }
}

/// Disques entiers (pas les partitions), hors périphériques virtuels
/// sans intérêt (loop, ram, zram).
fn whole_disks() -> Vec<String> {
    let Ok(entries) = std::fs::read_dir(SYS_BLOCK) else {
        return Vec::new();
    };
    entries
        .flatten()
        .filter_map(|e| e.file_name().into_string().ok())
        .filter(|n| !n.starts_with("loop") && !n.starts_with("ram") && !n.starts_with("zram"))
        .collect()
}

/// (total, utilisé) en octets du système de fichiers portant `mount`,
/// vu de l'utilisateur (l'espace réservé root compte comme utilisé).
fn fs_usage(mount: &str) -> Option<(u64, u64)> {
    let path = CString::new(mount).ok()?;
    let mut stat: libc::statvfs = unsafe { std::mem::zeroed() };
    if unsafe { libc::statvfs(path.as_ptr(), &mut stat) } != 0 {
        return None;
    }
    let frsize = if stat.f_frsize > 0 {
        stat.f_frsize
    } else {
        stat.f_bsize
    } as u64;
    let total = stat.f_blocks as u64 * frsize;
    let avail = stat.f_bavail as u64 * frsize;
    if total == 0 {
        return None;
    }
    Some((total, total.saturating_sub(avail)))
}

#[async_trait]
impl Collector for DiskCollector {
    fn id(&self) -> &'static str {
        "disk"
    }

    async fn probe(&mut self) -> ProbeResult {
        if let Err(e) = std::fs::read_to_string(PROC_DISKSTATS) {
            return ProbeResult::Unavailable(format!("{PROC_DISKSTATS} illisible : {e}"));
        }
        let disks = whole_disks();
        let mut details = BTreeMap::new();
        details.insert("disks".to_owned(), disks.join(","));
        ProbeResult::Available(CollectorCapability {
            available: true,
            reason: None,
            details,
        })
    }

    async fn collect(&mut self) -> anyhow::Result<Vec<Sample>> {
        let source = SourceId::local(); // écrasé par le scheduler
        let mut samples = Vec::new();

        // Débits et IOPS par deltas de /proc/diskstats.
        let disks = whole_disks();
        let content = std::fs::read_to_string(PROC_DISKSTATS)?;
        let current = parse_diskstats(&content, |n| disks.iter().any(|d| d == n));
        let now = Instant::now();
        let prev = self.prev.replace((current.clone(), now));

        if let Some((prev, prev_at)) = prev {
            let dt = now.duration_since(prev_at).as_secs_f64();
            if dt > 0.0 {
                let (mut sum_read_bps, mut sum_write_bps) = (0.0, 0.0);
                let (mut sum_read_iops, mut sum_write_iops) = (0.0, 0.0);
                for cur in &current {
                    let Some(p) = prev.iter().find(|p| p.name == cur.name) else {
                        continue;
                    };
                    // Compteur qui recule (hotplug, reset) → disque sauté.
                    let (Some(dsr), Some(dsw), Some(dr), Some(dw)) = (
                        cur.sectors_read.checked_sub(p.sectors_read),
                        cur.sectors_written.checked_sub(p.sectors_written),
                        cur.reads.checked_sub(p.reads),
                        cur.writes.checked_sub(p.writes),
                    ) else {
                        continue;
                    };
                    let read_bps = dsr as f64 * SECTOR_BYTES as f64 / dt;
                    let write_bps = dsw as f64 * SECTOR_BYTES as f64 / dt;
                    let read_iops = dr as f64 / dt;
                    let write_iops = dw as f64 / dt;
                    sum_read_bps += read_bps;
                    sum_write_bps += write_bps;
                    sum_read_iops += read_iops;
                    sum_write_iops += write_iops;
                    for (metric, v) in [
                        ("disk.read_bps", read_bps),
                        ("disk.write_bps", write_bps),
                        ("disk.read_iops", read_iops),
                        ("disk.write_iops", write_iops),
                    ] {
                        samples.push(
                            Sample::gauge(&source, metric, v).with_label("disk", cur.name.clone()),
                        );
                    }
                }
                for (metric, v) in [
                    ("disk.read_bps", sum_read_bps),
                    ("disk.write_bps", sum_write_bps),
                    ("disk.read_iops", sum_read_iops),
                    ("disk.write_iops", sum_write_iops),
                ] {
                    samples.push(Sample::gauge(&source, metric, v));
                }
            }
        }

        // Occupation par point de montage.
        let mounts = parse_mounts(&std::fs::read_to_string(PROC_MOUNTS)?);
        for mount in mounts {
            let Some((total, used)) = fs_usage(&mount) else {
                continue;
            };
            samples.push(
                Sample::gauge(&source, "fs.total_bytes", total as f64)
                    .with_label("mount", mount.clone()),
            );
            samples.push(
                Sample::gauge(&source, "fs.used_bytes", used as f64)
                    .with_label("mount", mount.clone()),
            );
            samples.push(
                Sample::gauge(&source, "fs.used_pct", used as f64 / total as f64 * 100.0)
                    .with_label("mount", mount),
            );
        }

        Ok(samples)
    }

    fn default_interval(&self) -> Duration {
        Duration::from_secs(1)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn collects_rates_and_fs_usage() {
        let mut collector = DiskCollector::new();
        assert!(collector.probe().await.is_available());

        let first = collector.collect().await.unwrap();
        // Premier tick : pas de débits (pas de delta), mais l'occupation
        // des montages est déjà là.
        assert!(first.iter().all(|s| s.metric.as_str() != "disk.read_bps"));
        assert!(
            first.iter().any(|s| s.metric.as_str() == "fs.used_pct"),
            "au moins un montage attendu"
        );

        tokio::time::sleep(Duration::from_millis(80)).await;
        let samples = collector.collect().await.unwrap();
        let global_read = samples
            .iter()
            .find(|s| s.metric.as_str() == "disk.read_bps" && s.labels.is_empty())
            .expect("débit global attendu");
        assert!(global_read.value.as_f64().unwrap() >= 0.0);

        for s in &samples {
            if s.metric.as_str() == "fs.used_pct" {
                let v = s.value.as_f64().unwrap();
                assert!((0.0..=100.0).contains(&v), "occupation hors bornes : {v}");
            }
        }
    }
}
