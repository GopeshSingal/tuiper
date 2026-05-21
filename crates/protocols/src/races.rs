use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RaceHistoryEntry {
    pub race_id: String,
    pub played_at: String,
    pub wpm: f64,
    pub accuracy: Option<f64>,
    pub opponent_username: Option<String>,
    pub opponent_wpm: Option<f64>,
    pub won: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RaceHistorySummary {
    pub total_races: u32,
    pub avg_wpm: f64,
    pub avg_accuracy: Option<f64>,
    pub best_wpm: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RaceHistoryResponse {
    pub races: Vec<RaceHistoryEntry>,
    pub summary: RaceHistorySummary,
}
