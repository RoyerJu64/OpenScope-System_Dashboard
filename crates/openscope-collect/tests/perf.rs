//! Budget performance (issue #10) : la chaîne scheduler → bus → abonné ne
//! doit presque rien consommer au repos. Le budget produit (< 1 % CPU pour
//! l'app complète à 1 Hz) sera mesuré sur l'app réelle à partir de la
//! Phase 1 ; ce test borne déjà le coût du cœur de collecte en CI.
//!
//! Test d'intégration séparé : il tourne dans son propre processus, la
//! mesure n'est donc pas polluée par les autres tests.

#![cfg(target_os = "linux")]

use std::time::Duration;

use openscope_collect::{mock::MockCollector, scheduler::LocalScheduler};
use openscope_core::{MetricBus, SourceId};

/// Temps CPU cumulé (user + system) du processus, en secondes.
fn process_cpu_seconds() -> f64 {
    let stat = std::fs::read_to_string("/proc/self/stat").expect("lecture /proc/self/stat");
    // Les champs suivent le nom du processus entre parenthèses ; on repart
    // de la parenthèse fermante pour ne pas dépendre du nom.
    let rest = stat.rsplit_once(')').expect("format /proc/self/stat").1;
    let fields: Vec<&str> = rest.split_whitespace().collect();
    // utime = champ 14, stime = champ 15 ; après ')' le premier champ est
    // l'état (champ 3), d'où les index 11 et 12.
    let utime: f64 = fields[11].parse().expect("utime");
    let stime: f64 = fields[12].parse().expect("stime");
    // Les temps de /proc sont exprimés en USER_HZ, fixé à 100 par l'ABI Linux.
    const USER_HZ: f64 = 100.0;
    (utime + stime) / USER_HZ
}

#[tokio::test]
#[ignore = "bench perf — lancé explicitement (cargo test -p openscope-collect --test perf -- --ignored)"]
async fn collect_chain_stays_under_cpu_budget() {
    const MEASURE: Duration = Duration::from_secs(5);
    // 5 % d'un cœur pour tout le processus de test : large marge CI, tout en
    // attrapant une régression grossière (boucle chaude, tick emballé).
    const BUDGET_CPU_SECONDS: f64 = 0.25;

    let bus = MetricBus::default();

    // Un abonné qui draine, pour mesurer le coût complet publication incluse.
    let mut rx = bus.subscribe();
    tokio::spawn(async move { while rx.recv().await.is_ok() {} });

    let _scheduler = LocalScheduler::start(
        SourceId::local(),
        bus.clone(),
        vec![Box::new(MockCollector::new())],
    )
    .await;

    let before = process_cpu_seconds();
    tokio::time::sleep(MEASURE).await;
    let used = process_cpu_seconds() - before;

    println!(
        "temps CPU sur {}s de collecte à 2 Hz : {used:.3}s (budget {BUDGET_CPU_SECONDS}s)",
        MEASURE.as_secs()
    );
    assert!(
        used < BUDGET_CPU_SECONDS,
        "budget CPU dépassé : {used:.3}s > {BUDGET_CPU_SECONDS}s"
    );
}
