//! `openscoped` — agent headless de collecte distante (Phase 4, issue #56).
//!
//! Réutilisera `openscope-collect` tel quel : boucle collecte → JSON-Lines
//! sur stdout, commandes reçues sur stdin. Compilé en statique (musl) pour
//! être déployé via SFTP sans dépendances.

fn main() {
    eprintln!(
        "openscoped {} — l'agent distant sera implémenté en Phase 4 (issue #56)",
        env!("CARGO_PKG_VERSION")
    );
    std::process::exit(1);
}
