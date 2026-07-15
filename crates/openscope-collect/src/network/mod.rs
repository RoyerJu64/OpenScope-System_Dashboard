//! Collecteur réseau (issue #16) : débits par interface.
//!
//! Métriques émises (gauges, deltas de /proc/net/dev) :
//! - `net.rx_bps` / `net.tx_bps` — global sans label (somme hors
//!   loopback), par interface avec `{"iface": "wlan0"}`

mod linux;

pub use linux::NetworkCollector;

/// Compteurs cumulés d'une interface, extraits de /proc/net/dev.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct IfaceStats {
    pub name: String,
    pub rx_bytes: u64,
    pub tx_bytes: u64,
}

/// Parse /proc/net/dev (les deux premières lignes sont des en-têtes) :
/// `iface: rx_bytes rx_packets … (8 champs) tx_bytes tx_packets …`
pub(crate) fn parse_net_dev(content: &str) -> Vec<IfaceStats> {
    let mut out = Vec::new();
    for line in content.lines().skip(2) {
        let Some((name, rest)) = line.split_once(':') else {
            continue;
        };
        let fields: Vec<u64> = rest
            .split_whitespace()
            .map(|f| f.parse().unwrap_or(0))
            .collect();
        if fields.len() < 16 {
            continue;
        }
        out.push(IfaceStats {
            name: name.trim().to_owned(),
            rx_bytes: fields[0],
            tx_bytes: fields[8],
        });
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    const NET_DEV: &str = "\
Inter-|   Receive                                                |  Transmit
 face |bytes    packets errs drop fifo frame compressed multicast|bytes    packets errs drop fifo colls carrier compressed
    lo:  111111     100    0    0    0     0          0         0   111111     100    0    0    0     0       0          0
 wlan0: 5000000    4000    0    0    0     0          0         0  1000000    2000    0    0    0     0       0          0
docker0:       0       0    0    0    0     0          0         0      648       8    0    0    0     0       0          0
";

    #[test]
    fn parses_interfaces_with_rx_tx() {
        let stats = parse_net_dev(NET_DEV);
        assert_eq!(stats.len(), 3);
        let wlan = stats.iter().find(|s| s.name == "wlan0").unwrap();
        assert_eq!(wlan.rx_bytes, 5_000_000);
        assert_eq!(wlan.tx_bytes, 1_000_000);
        let lo = stats.iter().find(|s| s.name == "lo").unwrap();
        assert_eq!(lo.rx_bytes, 111_111);
    }
}
