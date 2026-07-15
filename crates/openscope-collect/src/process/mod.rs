//! Processus (issues #17 et #18) : snapshot de la table des processus et
//! actions (kill, renice).
//!
//! Contrairement aux métriques, la table n'est pas poussée sur le bus :
//! le frontend la demande (pull) quand la page Processus est visible.

pub mod actions;
mod linux;

pub use linux::ProcessTable;

use serde::Serialize;

/// Une ligne de la table des processus. Le `cpu_pct` est en % d'un cœur
/// (convention htop : peut dépasser 100 pour un processus multithread).
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct ProcessRow {
    pub pid: i32,
    pub ppid: i32,
    pub name: String,
    pub user: String,
    /// R (running), S (sleeping), D (uninterruptible), Z (zombie), T (stopped)…
    pub state: char,
    pub cpu_pct: f64,
    pub rss_bytes: u64,
    pub cmdline: String,
}

/// Champs utiles de /proc/[pid]/stat.
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct StatLine {
    pub pid: i32,
    pub name: String,
    pub state: char,
    pub ppid: i32,
    /// utime + stime, en ticks USER_HZ.
    pub ticks: u64,
    /// RSS en pages.
    pub rss_pages: u64,
}

/// Parse /proc/[pid]/stat. Le nom est entre parenthèses et peut contenir
/// espaces et parenthèses : on coupe sur la DERNIÈRE parenthèse fermante.
pub(crate) fn parse_stat_line(content: &str) -> Option<StatLine> {
    let open = content.find('(')?;
    let close = content.rfind(')')?;
    let pid: i32 = content[..open].trim().parse().ok()?;
    let name = content[open + 1..close].to_owned();
    let fields: Vec<&str> = content[close + 1..].split_whitespace().collect();
    // Après la parenthèse : state est le champ 3 du fichier, donc l'index
    // 0 ici ; utime/stime sont les champs 14/15 (index 11/12), rss le
    // champ 24 (index 21).
    let state = fields.first()?.chars().next()?;
    let ppid: i32 = fields.get(1)?.parse().ok()?;
    let utime: u64 = fields.get(11)?.parse().ok()?;
    let stime: u64 = fields.get(12)?.parse().ok()?;
    let rss_pages: u64 = fields.get(21)?.parse().unwrap_or(0);
    Some(StatLine {
        pid,
        name,
        state,
        ppid,
        ticks: utime + stime,
        rss_pages,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_regular_stat_line() {
        let line = "1234 (openscope) S 1 1234 1234 0 -1 4194560 100 0 0 0 250 50 0 0 20 0 8 0 12345 100000000 5000 18446744073709551615 0 0 0 0 0 0 0 0 0 0 0 0 17 3 0 0 0 0 0";
        let stat = parse_stat_line(line).unwrap();
        assert_eq!(stat.pid, 1234);
        assert_eq!(stat.name, "openscope");
        assert_eq!(stat.state, 'S');
        assert_eq!(stat.ppid, 1);
        assert_eq!(stat.ticks, 300); // 250 + 50
        assert_eq!(stat.rss_pages, 5000);
    }

    #[test]
    fn name_with_spaces_and_parens_is_kept_whole() {
        let line = "42 (Web Content (2)) R 1 42 42 0 -1 0 0 0 0 0 10 5 0 0 20 0 1 0 0 0 100 0 0 0 0 0 0 0 0 0 0 0 0 0 17 0 0 0 0 0 0";
        let stat = parse_stat_line(line).unwrap();
        assert_eq!(stat.name, "Web Content (2)");
        assert_eq!(stat.state, 'R');
        assert_eq!(stat.ticks, 15);
    }

    #[test]
    fn garbage_returns_none() {
        assert_eq!(parse_stat_line(""), None);
        assert_eq!(parse_stat_line("pas un stat"), None);
    }
}
