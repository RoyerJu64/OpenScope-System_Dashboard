//! Collecteur disque (issue #15) : débits, IOPS et occupation.
//!
//! Métriques émises (gauges) :
//! - `disk.read_bps` / `disk.write_bps` — global sans label (somme), par
//!   disque physique avec `{"disk": "nvme0n1"}` (deltas de /proc/diskstats)
//! - `disk.read_iops` / `disk.write_iops` — mêmes déclinaisons
//! - `fs.total_bytes` / `fs.used_bytes` / `fs.used_pct` — par point de
//!   montage avec `{"mount": "/"}` (statvfs sur les montages /dev/*)

mod linux;

pub use linux::DiskCollector;

/// Compteurs cumulés d'un disque, extraits de /proc/diskstats.
/// Les secteurs y sont toujours des unités de 512 octets.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct DiskStats {
    pub name: String,
    pub reads: u64,
    pub sectors_read: u64,
    pub writes: u64,
    pub sectors_written: u64,
}

pub(crate) const SECTOR_BYTES: u64 = 512;

/// Parse /proc/diskstats. `keep` filtre par nom de périphérique (on lui
/// passe la liste des disques entiers de /sys/block, ce qui écarte
/// partitions, loop et ram).
pub(crate) fn parse_diskstats(content: &str, keep: impl Fn(&str) -> bool) -> Vec<DiskStats> {
    let mut out = Vec::new();
    for line in content.lines() {
        let fields: Vec<&str> = line.split_whitespace().collect();
        // major minor name reads _ sectors_read _ writes _ sectors_written …
        if fields.len() < 10 {
            continue;
        }
        let name = fields[2];
        if !keep(name) {
            continue;
        }
        let parse = |i: usize| fields[i].parse::<u64>().unwrap_or(0);
        out.push(DiskStats {
            name: name.to_owned(),
            reads: parse(3),
            sectors_read: parse(5),
            writes: parse(7),
            sectors_written: parse(9),
        });
    }
    out
}

/// Un point de montage à mesurer, extrait de /proc/mounts : uniquement
/// les périphériques /dev/* (écarte tmpfs, squashfs des snaps, etc.),
/// hors /dev/loop*.
pub(crate) fn parse_mounts(content: &str) -> Vec<String> {
    let mut out: Vec<String> = Vec::new();
    for line in content.lines() {
        let mut fields = line.split_whitespace();
        let (Some(device), Some(mount)) = (fields.next(), fields.next()) else {
            continue;
        };
        if !device.starts_with("/dev/") || device.starts_with("/dev/loop") {
            continue;
        }
        // /proc/mounts échappe les espaces en \040.
        let mount = mount.replace("\\040", " ");
        if !out.contains(&mount) {
            out.push(mount);
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    const DISKSTATS: &str = "\
 259       0 nvme0n1 1000 0 200000 500 2000 0 400000 800 0 0 0 0 0 0 0 0 0
 259       1 nvme0n1p1 900 0 190000 450 1900 0 390000 700 0 0 0 0 0 0 0 0 0
   7       0 loop0 10 0 100 1 0 0 0 0 0 0 0 0 0 0 0 0 0
   8       0 sda 500 0 100000 300 100 0 20000 100 0 0 0 0 0 0 0 0 0
";

    #[test]
    fn parses_and_filters_whole_disks() {
        let whole = ["nvme0n1", "sda"];
        let stats = parse_diskstats(DISKSTATS, |n| whole.contains(&n));
        assert_eq!(stats.len(), 2);
        assert_eq!(stats[0].name, "nvme0n1");
        assert_eq!(stats[0].sectors_read, 200_000);
        assert_eq!(stats[0].writes, 2000);
        assert_eq!(stats[1].name, "sda");
    }

    #[test]
    fn mounts_keeps_dev_devices_only() {
        let mounts = "\
sysfs /sys sysfs rw 0 0
/dev/nvme0n1p2 / ext4 rw 0 0
tmpfs /run tmpfs rw 0 0
/dev/nvme0n1p1 /boot/efi vfat rw 0 0
/dev/loop3 /snap/foo/1 squashfs ro 0 0
/dev/sda1 /mnt/disque\\040usb ext4 rw 0 0
/dev/nvme0n1p2 /var/lib/docker/btrfs ext4 rw 0 0
";
        let mounts = parse_mounts(mounts);
        assert_eq!(
            mounts,
            vec!["/", "/boot/efi", "/mnt/disque usb", "/var/lib/docker/btrfs"]
        );
    }
}
