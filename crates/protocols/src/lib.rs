use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ClientMessage {
    JoinQueue {
        value: u32,
    },
    LeaveQueue,
    RaceProgress {
        wpm: f64,
        accuracy: f64,
        chars_typed: u32,
    },
    RaceFinished {
        wpm: f64,
        accuracy: f64,
        consistency: f64,
        chars_typed: u32,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ServerMessage {
    Queue,
    RaceStart {
        race_id: String,
        value: u32,
        seed: u64,
        start_at_unix_ms: u64,
    },
    OpponentProgress {
        wpm: f64,
        chars_typed: u32,
    },
    RaceEnd {
        results: RaceResults,
    },
    Error {
        message: String
    },
}


#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RaceResults {
    pub me: PlayerResult,
    pub opponent: PlayerResult,
    pub winner: Option<Winner>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlayerResult {
    pub wpm: f64,
    pub accuracy: f64,
    pub consistency: f64,
    pub chars_typed: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Winner {
    Me,
    Opponent,
}
