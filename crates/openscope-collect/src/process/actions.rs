//! Actions sur les processus (issue #18) : kill et renice, sans
//! privilèges — l'utilisateur n'agit que sur ses propres processus.
//! L'élévation ciblée (polkit/UAC) arrive en Phase 2 (issue #45).

use openscope_core::ActionOutcome;

/// Envoie `signal` (SIGTERM=15, SIGKILL=9…) au processus `pid`.
pub fn kill(pid: i32, signal: i32) -> ActionOutcome {
    if pid <= 0 {
        // Interdit les cibles de groupe (0, -1, -pgid) : une seule cible.
        return ActionOutcome::failure("pid invalide");
    }
    if unsafe { libc::kill(pid, signal) } == 0 {
        ActionOutcome::success()
    } else {
        ActionOutcome::failure(os_error())
    }
}

/// Change la priorité (`nice` entre -20 et 19). Sans privilèges, seule
/// une augmentation (baisse de priorité) est permise par le noyau.
pub fn set_priority(pid: i32, nice: i32) -> ActionOutcome {
    if pid <= 0 {
        return ActionOutcome::failure("pid invalide");
    }
    let nice = nice.clamp(-20, 19);
    // setpriority retourne -1 aussi bien en erreur qu'en valeur légitime :
    // on remet errno à zéro et on ne juge que lui.
    unsafe { *libc::__errno_location() = 0 };
    let ret = unsafe { libc::setpriority(libc::PRIO_PROCESS, pid as libc::id_t, nice) };
    let errno = unsafe { *libc::__errno_location() };
    if ret == -1 && errno != 0 {
        ActionOutcome::failure(os_error())
    } else {
        ActionOutcome::success()
    }
}

fn os_error() -> String {
    let err = std::io::Error::last_os_error();
    match err.raw_os_error() {
        Some(libc::EPERM) => "permission refusée (processus d'un autre utilisateur ?)".to_owned(),
        Some(libc::ESRCH) => "processus introuvable".to_owned(),
        Some(libc::EACCES) => "accès refusé".to_owned(),
        _ => err.to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::process::Command;

    #[test]
    fn kill_terminates_own_child_and_rejects_bad_targets() {
        let mut child = Command::new("sleep").arg("30").spawn().unwrap();
        let pid = child.id() as i32;

        assert!(kill(pid, libc::SIGTERM).ok);
        assert!(child.wait().unwrap().signal() == Some(libc::SIGTERM));

        let gone = kill(999_999_999, 0);
        assert!(!gone.ok);
        assert!(gone.message.unwrap().contains("introuvable"));

        assert!(!kill(0, libc::SIGTERM).ok, "cibles de groupe interdites");
        assert!(!kill(-1, libc::SIGTERM).ok);
    }

    #[test]
    fn renice_own_child_up_is_allowed() {
        let mut child = Command::new("sleep").arg("30").spawn().unwrap();
        let pid = child.id() as i32;

        let outcome = set_priority(pid, 10);
        assert!(outcome.ok, "{:?}", outcome.message);

        let _ = kill(pid, libc::SIGKILL);
        let _ = child.wait();
    }

    use std::os::unix::process::ExitStatusExt;
}
