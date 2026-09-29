use crate::{AttentionSignal, AttentionWindow, PhysicsProjection, SignalExtractor};
use std::sync::{Arc, Mutex};
use std::time::{SystemTime, UNIX_EPOCH};

/// Trait for systems that need to react to changes in the attention physics manifold.
pub trait SignalSubscriber: Send + Sync {
    /// Called when a new physics projection is calculated from the ingestion loop.
    fn on_physics_update(&self, projection: &PhysicsProjection);
    /// Called when raw raw attention telemetry is extracted.
    fn on_raw_signal(&self, signal: &AttentionSignal);
}

/// The core perception ingestion loop.
/// Pulls external signal streams, normalizes them via AttentionWindow,
/// and broadcasts the resulting physics projection to subscribers.
pub struct IngestionLoop {
    window: Mutex<AttentionWindow>,
    subscribers: Vec<Arc<dyn SignalSubscriber>>,
}

impl IngestionLoop {
    /// Create a new ingestion loop with a sliding window of the specified size.
    pub fn new(window_size: usize) -> Self {
        Self {
            window: Mutex::new(AttentionWindow::new(window_size)),
            subscribers: Vec::new(),
        }
    }

    /// Register a subscriber (e.g., krk-obs telemetry, or krk-memory-plane router).
    pub fn subscribe(&mut self, subscriber: Arc<dyn SignalSubscriber>) {
        self.subscribers.push(subscriber);
    }

    /// Ingest a raw text block (e.g., from X/Twitter firehose, HackerNews, or Digg API).
    pub fn ingest_text_stream(&self, raw_text: &str) {
        let ts = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        // 1. Extract raw signal features
        let signal = SignalExtractor::extract(raw_text, ts);

        // 2. Broadcast raw signal to subscribers (e.g. Memory Plane for raw Memarch capture)
        for sub in &self.subscribers {
            sub.on_raw_signal(&signal);
        }

        // 3. Update temporal window
        let mut window = self.window.lock().unwrap();
        window.push(signal);

        // 4. Compute smoothed physics projection
        if let Some(avg_signal) = window.average() {
            let projection = avg_signal.to_physics();

            // 5. Broadcast smoothed physics projection (e.g. to krk-obs and krk-psc)
            for sub in &self.subscribers {
                sub.on_physics_update(&projection);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicUsize, Ordering};

    struct MockSubscriber {
        pub physics_updates: AtomicUsize,
        pub raw_signals: AtomicUsize,
    }

    impl MockSubscriber {
        fn new() -> Self {
            Self {
                physics_updates: AtomicUsize::new(0),
                raw_signals: AtomicUsize::new(0),
            }
        }
    }

    impl SignalSubscriber for MockSubscriber {
        fn on_physics_update(&self, _p: &PhysicsProjection) {
            self.physics_updates.fetch_add(1, Ordering::SeqCst);
        }
        fn on_raw_signal(&self, _s: &AttentionSignal) {
            self.raw_signals.fetch_add(1, Ordering::SeqCst);
        }
    }

    #[test]
    fn test_ingestion_loop_broadcasts_to_subscribers() {
        let mut ingestion = IngestionLoop::new(5);
        let sub = Arc::new(MockSubscriber::new());
        ingestion.subscribe(sub.clone());

        let text1 = "autonomous agents are scaling fast with new safety guardrails.";
        ingestion.ingest_text_stream(text1);

        assert_eq!(sub.raw_signals.load(Ordering::SeqCst), 1);
        assert_eq!(sub.physics_updates.load(Ordering::SeqCst), 1);

        let text2 = "open-source AI consolidation creates monopoly.";
        ingestion.ingest_text_stream(text2);

        assert_eq!(sub.raw_signals.load(Ordering::SeqCst), 2);
        assert_eq!(sub.physics_updates.load(Ordering::SeqCst), 2);
    }
}
