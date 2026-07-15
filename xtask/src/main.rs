//! Tâches de développement, invoquées via l'alias `cargo xtask <tâche>`
//! (défini dans `.cargo/config.toml`).

use std::path::{Path, PathBuf};
use std::process::{Command, ExitCode};

fn main() -> ExitCode {
    match std::env::args().nth(1).as_deref() {
        Some("gen-types") => gen_types(),
        _ => {
            eprintln!("usage : cargo xtask <tâche>\n\ntâches disponibles :");
            eprintln!(
                "  gen-types   régénère frontend/src/ipc/bindings/ depuis les types Rust (ts-rs)"
            );
            ExitCode::FAILURE
        }
    }
}

fn workspace_root() -> PathBuf {
    // xtask vit dans <racine>/xtask.
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("xtask doit être à la racine du workspace")
        .to_path_buf()
}

/// Une seule source de vérité pour les types de la frontière IPC : les
/// structs Rust. ts-rs les exporte en .ts via des tests générés
/// (`export_bindings_*`), écrits dans TS_RS_EXPORT_DIR.
fn gen_types() -> ExitCode {
    let root = workspace_root();
    let export_dir = root.join("frontend/src/ipc/bindings");

    // Répertoire régénéré de zéro pour purger les types disparus.
    if export_dir.exists() {
        std::fs::remove_dir_all(&export_dir).expect("suppression de bindings/");
    }

    let status = Command::new("cargo")
        .current_dir(&root)
        .args([
            "test",
            "-p",
            "openscope-core",
            "-p",
            "openscope-app",
            "--features",
            "openscope-core/ts,openscope-app/ts",
            "export_bindings",
        ])
        .env("TS_RS_EXPORT_DIR", &export_dir)
        .status()
        .expect("lancement de cargo test");

    if !status.success() {
        eprintln!("gen-types : échec de l'export ts-rs");
        return ExitCode::FAILURE;
    }

    let count = std::fs::read_dir(&export_dir)
        .map(|entries| entries.count())
        .unwrap_or(0);
    println!(
        "gen-types : {count} fichiers générés dans {}",
        export_dir.display()
    );
    ExitCode::SUCCESS
}
