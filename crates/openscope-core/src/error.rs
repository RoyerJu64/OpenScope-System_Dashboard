use thiserror::Error;

/// Erreurs communes aux frontières entre modules.
#[derive(Debug, Error)]
pub enum CoreError {
    #[error("source indisponible : {0}")]
    SourceUnavailable(String),

    #[error("action refusée : {0}")]
    ActionDenied(String),

    #[error("stockage historique : {0}")]
    Storage(String),

    #[error("entrée invalide : {0}")]
    InvalidInput(String),

    #[error(transparent)]
    Io(#[from] std::io::Error),

    #[error(transparent)]
    Other(#[from] anyhow::Error),
}
