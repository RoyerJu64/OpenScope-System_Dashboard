use std::sync::Arc;

use tokio::sync::broadcast;

use crate::sample::Sample;

/// Un batch = la sortie d'un tick d'un collecteur, partagé sans copie
/// entre tous les abonnés.
pub type Batch = Arc<Vec<Sample>>;

/// Bus de métriques : l'unique canal entre producteurs (collecte locale,
/// sources distantes, plugins) et consommateurs (historique, alertes,
/// forwarder UI).
///
/// Sémantique `tokio::sync::broadcast` : un abonné trop lent perd les
/// batches les plus anciens (`RecvError::Lagged`) — on préfère perdre de
/// l'historique temps réel que bloquer la collecte.
#[derive(Clone)]
pub struct MetricBus {
    tx: broadcast::Sender<Batch>,
}

impl MetricBus {
    /// `capacity` : nombre de batches retenus pour les abonnés lents.
    /// 256 couvre plusieurs secondes de collecte toutes sources confondues.
    pub fn new(capacity: usize) -> Self {
        let (tx, _) = broadcast::channel(capacity);
        Self { tx }
    }

    /// Publie un batch. Sans abonné, le batch est simplement perdu
    /// (démarrage, tests) — ce n'est pas une erreur.
    pub fn publish(&self, batch: Batch) {
        let _ = self.tx.send(batch);
    }

    pub fn subscribe(&self) -> broadcast::Receiver<Batch> {
        self.tx.subscribe()
    }

    pub fn subscriber_count(&self) -> usize {
        self.tx.receiver_count()
    }
}

impl Default for MetricBus {
    fn default() -> Self {
        Self::new(256)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::sample::Sample;
    use crate::source::SourceId;

    #[tokio::test]
    async fn publish_reaches_all_subscribers() {
        let bus = MetricBus::new(8);
        let mut rx1 = bus.subscribe();
        let mut rx2 = bus.subscribe();

        let src = SourceId::local();
        let batch: Batch = Arc::new(vec![Sample::gauge(&src, "cpu.usage", 42.0)]);
        bus.publish(batch.clone());

        let got1 = rx1.recv().await.unwrap();
        let got2 = rx2.recv().await.unwrap();
        assert_eq!(got1[0].value, batch[0].value);
        assert_eq!(got2[0].metric, batch[0].metric);
        // Zéro copie : les trois handles pointent la même allocation.
        assert!(Arc::ptr_eq(&got1, &batch) && Arc::ptr_eq(&got2, &batch));
    }

    #[tokio::test]
    async fn publish_without_subscriber_is_not_an_error() {
        let bus = MetricBus::new(8);
        let src = SourceId::local();
        bus.publish(Arc::new(vec![Sample::gauge(&src, "x", 1.0)]));
        assert_eq!(bus.subscriber_count(), 0);
    }

    #[tokio::test]
    async fn slow_subscriber_lags_instead_of_blocking() {
        let bus = MetricBus::new(2);
        let mut rx = bus.subscribe();
        let src = SourceId::local();
        for i in 0..5 {
            bus.publish(Arc::new(vec![Sample::gauge(&src, "x", i as f64)]));
        }
        // Les 3 premiers batches sont perdus, la réception signale le retard.
        assert!(matches!(
            rx.recv().await,
            Err(broadcast::error::RecvError::Lagged(3))
        ));
    }
}
