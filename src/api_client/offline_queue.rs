use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::Mutex;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum QueuedAction {
    SubmitSession(String),   // serialized session JSON
    MintAchievement(String), // achievement_id
}

#[derive(Default, Clone)]
pub struct OfflineQueue {
    inner: Arc<Mutex<Vec<QueuedAction>>>,
}

impl OfflineQueue {
    pub fn new() -> Self {
        Self::default()
    }

    pub async fn push(&self, action: QueuedAction) {
        self.inner.lock().await.push(action);
    }

    pub async fn drain(&self) -> Vec<QueuedAction> {
        let mut guard = self.inner.lock().await;
        std::mem::take(&mut *guard)
    }

    pub async fn len(&self) -> usize {
        self.inner.lock().await.len()
    }
}

/// Detects whether a reqwest error indicates the client is offline
/// (as opposed to a server-side error like 500).
pub fn is_offline_error(err: &reqwest::Error) -> bool {
    err.is_connect() || err.is_timeout()
}