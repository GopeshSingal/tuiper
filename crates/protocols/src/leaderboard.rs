use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EloLeaderboardEntry {
    pub username: String,
    pub elo: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DailyWpmLeaderboardEntry {
    pub username: String,
    pub wpm: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LeaderboardResponse {
    pub top_elo: Vec<EloLeaderboardEntry>,
    pub top_daily_wpm: Vec<DailyWpmLeaderboardEntry>,
}
