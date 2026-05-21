pub mod auth;
pub mod game;
pub mod leaderboard;
pub mod races;

pub use auth::{
    AuthRequest,
    AuthAction,
    AccountPublic,
    AuthResponse,
    ApiError,
};

pub use game::{
    ClientMessage,
    ServerMessage,
    RaceOpponent,
    RaceResults,
    PlayerResult,
    Winner,
};

pub use leaderboard::{
    DailyWpmLeaderboardEntry,
    EloLeaderboardEntry,
    LeaderboardResponse,
};

pub use races::{
    RaceHistoryEntry,
    RaceHistoryResponse,
    RaceHistorySummary,
};
