use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LeaderboardEntry {
    pub player_id: String,
    pub username: String,
    pub score: u64,
    pub rank: u32,
}