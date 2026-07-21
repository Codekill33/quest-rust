use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Session {
    pub player_id: String,
    pub score: u64,
    pub duration_secs: u64,
    pub completed_at: chrono::DateTime<chrono::Utc>,
}