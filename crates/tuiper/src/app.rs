use crate::auth;
use crate::mode::{self, ModeConfig, RaceMode, TIME_PRESETS, WORDS_PRESETS};
use crate::theme::{self, Theme, ThemeEditColumn};
use crate::typing::{TypingState, TypingStats};
use crate::words::{generate_next_chunk, generate_words_text};

use common::now_unix_ms;
use protocols::ServerMessage::*;
use protocols::{
    AccountPublic, ClientMessage, LeaderboardResponse, RaceHistoryResponse, RaceOpponent,
    RaceResults, ServerMessage,
};

use std::sync::mpsc;
use std::time::Instant;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Screen {
    Login,
    Lobby,
    Queue,
    Race,
    Results,
    Config,
    Leaderboard,
    Statistics,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LoginField {
    Username,
    Password,
}

impl Screen {
    pub fn uses_shell(self) -> bool {
        matches!(self, Screen::Lobby | Screen::Leaderboard | Screen::Config | Screen::Statistics)
    }
}

const REFILL_THRESHOLD: usize = 10;

pub struct App {
    pub screen: Screen,
    pub typing: Option<TypingState>,
    pub result: Option<TypingStats>,

    pub quit: bool,

    // Streaming words
    pub seed: Option<u64>,
    pub words_so_far: u32,

    // multiplayer
    pub ws_tx: Option<mpsc::Sender<ClientMessage>>,
    multiplayer_race: bool,
    multiplayer_session_active: bool,
    multiplayer_race_t0: Option<Instant>,
    pub opponent_wpm: f64,
    pub opponent_chars: u32,
    pub multiplayer_opponent: Option<RaceOpponent>,
    pub opponent_wpm_history: Vec<(f64, f64)>,
    pub last_progress_sent: f64,
    pub race_results: Option<RaceResults>,
    pub multiplayer_start_at_unix_ms: Option<u64>,

    // theme config
    pub theme: Theme,
    pub theme_edit_row: usize,
    pub theme_edit_col: ThemeEditColumn,

    // race mode
    pub mode: RaceMode,
    pub words_preset_idx: u8,
    pub time_preset_idx: u8,

    pub account: Option<AccountPublic>,
    pub auth_token: Option<String>,

    pub login_username: String,
    pub login_password: String,
    pub login_focus: LoginField,
    pub login_error: Option<String>,

    pub leaderboard: Option<LeaderboardResponse>,
    pub leaderboard_error: Option<String>,

    pub race_history: Option<RaceHistoryResponse>,
    pub race_history_error: Option<String>,
    pub stats_scroll_offset: usize,
}

impl App {
    pub fn new() -> Self {
        let mode_config = mode::load();
        Self {
            screen: Screen::Login,
            typing: None,
            result: None,
            quit: false,

            // streaming
            seed: None,
            words_so_far: 0,

            // multiplayer
            ws_tx: None,
            multiplayer_race: false,
            multiplayer_session_active: false,
            multiplayer_race_t0: None,
            opponent_wpm: 0.0,
            opponent_chars: 0,
            multiplayer_opponent: None,
            opponent_wpm_history: Vec::new(),
            last_progress_sent: 0.0,
            race_results: None,
            multiplayer_start_at_unix_ms: None,

            // theme config
            theme: theme::load(),
            theme_edit_row: 0,
            theme_edit_col: ThemeEditColumn::default(),

            // race mode
            mode: mode_config.mode,
            words_preset_idx: mode_config.words_preset_idx,
            time_preset_idx: mode_config.time_preset_idx,

            account: None,
            auth_token: None,

            login_username: String::new(),
            login_password: String::new(),
            login_focus: LoginField::Username,
            login_error: None,

            leaderboard: None,
            leaderboard_error: None,

            race_history: None,
            race_history_error: None,
            stats_scroll_offset: 0,
        }
    }

    pub fn cycle_login_focus(&mut self, _delta: i32) {
        self.login_focus = match self.login_focus {
            LoginField::Username => LoginField::Password,
            LoginField::Password => LoginField::Username,
        };
    }

    pub fn login_field_mut(&mut self) -> &mut String {
        match self.login_focus {
            LoginField::Username => &mut self.login_username,
            LoginField::Password => &mut self.login_password,
        }
    }

    pub fn complete_login(&mut self, token: String, account: AccountPublic) {
        self.auth_token = Some(token);
        self.account = Some(account);
        self.login_username.clear();
        self.login_password.clear();
        self.login_error = None;
        self.login_focus = LoginField::Username;
        self.screen = Screen::Lobby;
    }

    pub fn enter_as_guest(&mut self) {
        self.auth_token = None;
        self.account = None;
        self.login_username.clear();
        self.login_password.clear();
        self.login_error = None;
        self.login_focus = LoginField::Username;
        self.screen = Screen::Lobby;
    }

    pub fn lobby_value(&self) -> u32 {
        match self.mode {
            RaceMode::Words => WORDS_PRESETS[self.words_preset_idx as usize],
            RaceMode::Time => TIME_PRESETS[self.time_preset_idx as usize],
        }
    }

    fn mode_config(&self) -> ModeConfig {
        ModeConfig {
            mode: self.mode,
            words_preset_idx: self.words_preset_idx,
            time_preset_idx: self.time_preset_idx,
        }
    }

    pub fn persist_mode(&self) {
        let _ = mode::save(&self.mode_config());
    }

    pub fn cycle_length(&mut self, delta: i32) {
        match self.mode {
            RaceMode::Words => {
                let len = WORDS_PRESETS.len() as i32;
                self.words_preset_idx =
                    (self.words_preset_idx as i32 + delta).rem_euclid(len) as u8;
            }
            RaceMode::Time => {
                let len = TIME_PRESETS.len() as i32;
                self.time_preset_idx =
                    (self.time_preset_idx as i32 + delta).rem_euclid(len) as u8;
            }
        }
        self.persist_mode();
    }

    pub fn cycle_mode(&mut self, delta: i32) {
        if delta > 0 {
            self.mode = match self.mode {
                RaceMode::Time => RaceMode::Words,
                RaceMode::Words => RaceMode::Time,
            }
        } else {
            self.mode = match self.mode {
                RaceMode::Time => RaceMode::Words,
                RaceMode::Words => RaceMode::Time,
            }
        }
        self.persist_mode();
    }

    pub fn handle_server_message(&mut self, msg: ServerMessage) {
        match msg {
            Queue => {
                self.screen = Screen::Queue;
            }
            RaceStart {
                race_id: _,
                value,
                seed,
                start_at_unix_ms,
                opponent,
            } => {
                self.start_multiplayer(seed, value, start_at_unix_ms, opponent);
            }
            OpponentProgress { wpm, chars_typed } => {
                if self.multiplayer_session_active {
                    self.opponent_wpm = wpm;
                    self.opponent_chars = chars_typed;
                    if let Some(t0) = self.multiplayer_race_t0 {
                        let secs = t0.elapsed().as_secs_f64();
                        if self.opponent_wpm_history.last().map_or(true, |last| {
                            (last.0 - secs).abs() > 0.05 || (last.1 - wpm).abs() > 0.01
                        }) {
                            self.opponent_wpm_history.push((secs, wpm));
                        }
                    }
                }
            }
            RaceEnd { results } => {
                self.multiplayer_race = false;
                if self.multiplayer_session_active {
                    if let Some(t0) = self.multiplayer_race_t0 {
                        let secs = t0.elapsed().as_secs_f64();
                        let ow = results.opponent.wpm;
                        if self.opponent_wpm_history.last().map_or(true, |last| {
                            (last.1 - ow).abs() > 0.1 || (last.0 - secs).abs() > 0.01
                        }) {
                            self.opponent_wpm_history.push((secs, ow));
                        }
                    }
                }
                self.multiplayer_session_active = false;
                self.multiplayer_race_t0 = None;
                self.multiplayer_opponent = None;
                self.race_results = Some(results);
                if self.result.is_none() {
                    self.result = self.typing.as_ref().map(TypingState::final_stats);
                }
                self.typing = None;
                self.multiplayer_start_at_unix_ms = None;
                self.screen = Screen::Results;
            }
            Error { message: _ } => {
                self.multiplayer_race = false;
                self.multiplayer_session_active = false;
                self.multiplayer_race_t0 = None;
                self.opponent_wpm_history.clear();
                self.disconnect_websocket();
                self.screen = Screen::Lobby;
            }
        }
    }

    pub fn start_race(&mut self, value: u32) {
        self.multiplayer_race = false;
        self.multiplayer_session_active = false;
        self.multiplayer_race_t0 = None;
        self.multiplayer_opponent = None;
        self.opponent_wpm_history.clear();
        self.result = None;
        self.race_results = None;
        self.screen = Screen::Race;
        self.multiplayer_start_at_unix_ms = None;

        let seed = rand::random();
        let first_chunk = match self.mode {
            RaceMode::Time => generate_next_chunk(seed, value, 0)
                .unwrap_or_else(|| "jumped over the lazy dog".to_string()),
            RaceMode::Words => generate_words_text(seed, 0, value)
                .unwrap_or_else(|| "jumped over the lazy dog".to_string()),
        };
        let word_count = first_chunk.split_whitespace().count() as u32;
        self.typing = Some(TypingState::new(first_chunk, value));
        self.seed = Some(seed);
        self.words_so_far = word_count;
    }

    pub fn start_multiplayer(
        &mut self,
        seed: u64,
        value: u32,
        start_at_unix_ms: u64,
        opponent: RaceOpponent,
    ) {
        self.multiplayer_race = true;
        self.multiplayer_session_active = true;
        self.multiplayer_opponent = Some(opponent);
        self.multiplayer_race_t0 = None;
        self.result = None;
        self.race_results = None;
        self.opponent_wpm = 0.0;
        self.opponent_chars = 0;
        self.opponent_wpm_history.clear();
        self.last_progress_sent = 0.0;
        self.multiplayer_start_at_unix_ms = Some(start_at_unix_ms);
        self.screen = Screen::Race;

        let first_chunk = crate::words::generate_next_chunk(seed, value, 0)
            .unwrap_or_else(|| "the quick brown fox".to_string());
        let word_count = first_chunk.split_whitespace().count() as u32;
        self.typing = Some(TypingState::new(first_chunk, value));
        self.seed = Some(seed);
        self.words_so_far = word_count;
    }

    pub fn tick(&mut self) {
        if let Some(ref mut t) = self.typing {
            if let Some(start_at_unix_ms) = self.multiplayer_start_at_unix_ms {
                if t.start_time().is_none() && now_unix_ms() >= start_at_unix_ms {
                    t.start();
                    if self.multiplayer_race_t0.is_none() {
                        self.multiplayer_race_t0 = Some(Instant::now());
                    }
                }
            }
            t.sample_raw_wpm();

            if self.mode == RaceMode::Time || self.multiplayer_race {
                if let Some(seed) = self.seed {
                    let text_words = t.text().split_whitespace().count();
                    let cursor_words = t.input().split_whitespace().count();
                    let words_ahead = text_words.saturating_sub(cursor_words);
                    if words_ahead <= REFILL_THRESHOLD {
                        if let Some(chunk) = generate_next_chunk(seed, t.value(), self.words_so_far)
                        {
                            if !chunk.is_empty() {
                                t.append_text(&chunk);
                                self.words_so_far += chunk.split_whitespace().count() as u32;
                            }
                        } else {
                            self.seed = None;
                        }
                    }
                }
            }

            let elapsed = t.elapsed_secs();
            if self.multiplayer_race {
                if let Some(ref tx) = self.ws_tx {
                    if elapsed - self.last_progress_sent >= 0.3 {
                        let _ = tx.send(ClientMessage::RaceProgress {
                            wpm: t.wpm(),
                            accuracy: t.accuracy(),
                            chars_typed: t.cursor() as u32,
                        });
                        self.last_progress_sent = elapsed;
                    }
                }
            }

            let finish_mode = if self.multiplayer_race {
                RaceMode::Time
            } else {
                self.mode
            };
            if t.is_finished(finish_mode) {
                if self.multiplayer_race {
                    if let Some(ref tx) = self.ws_tx {
                        let stats = t.final_stats();
                        let _ = tx.send(ClientMessage::RaceFinished {
                            wpm: stats.wpm,
                            accuracy: stats.accuracy,
                            consistency: stats.consistency,
                            chars_typed: stats.chars_typed,
                        });
                    }
                    self.multiplayer_race = false;
                }
                self.result = Some(t.final_stats());
                self.typing = None;
                self.seed = None;
                self.words_so_far = 0;
                self.multiplayer_start_at_unix_ms = None;
                self.screen = Screen::Results;
            }
        }
    }

    pub fn disconnect_websocket(&mut self) {
        self.ws_tx = None;
        self.multiplayer_opponent = None;
    }

    pub fn refresh_account_elo(&mut self, ws_url: &str) -> Result<(), String> {
        let Some(account) = self.account.as_ref() else {
            return Ok(());
        };

        let elo_url = auth::account_elo_url_for_ws_url(ws_url, &account.username)?;
        let (_, elo) = auth::fetch_account_elo(&elo_url)?;
        if let Some(account_mut) = self.account.as_mut() {
            account_mut.elo = elo;
        }
        Ok(())
    }

    pub fn refresh_leaderboard(&mut self, ws_url: &str) -> Result<(), String> {
        let url = auth::leaderboard_url_for_ws_url(ws_url)?;
        match auth::fetch_leaderboard(&url) {
            Ok(data) => {
                self.leaderboard = Some(data);
                self.leaderboard_error = None;
                Ok(())
            }
            Err(e) => {
                self.leaderboard_error = Some(e.clone());
                self.leaderboard = None;
                Err(e)
            }
        }
    }

    pub fn refresh_race_history(&mut self, ws_url: &str) -> Result<(), String> {
        let Some(account) = self.account.as_ref() else {
            self.race_history = None;
            self.race_history_error =
                Some("Sign in to view your online race statistics".to_string());
            self.stats_scroll_offset = 0;
            return Ok(());
        };

        let url = auth::race_history_url_for_ws_url(ws_url, &account.username)?;
        match auth::fetch_race_history(&url) {
            Ok(data) => {
                self.race_history = Some(data);
                self.race_history_error = None;
                self.stats_scroll_offset = 0;
                Ok(())
            }
            Err(e) => {
                self.race_history_error = Some(e.clone());
                self.race_history = None;
                self.stats_scroll_offset = 0;
                Err(e)
            }
        }
    }

    pub fn typing(&self) -> Option<&TypingState> {
        self.typing.as_ref()
    }

    pub fn result(&self) -> Option<&TypingStats> {
        self.result.as_ref()
    }

    pub fn is_multi(&self) -> bool {
        self.multiplayer_race && self.screen == Screen::Race
    }

    pub fn is_waiting_for_multiplayer_start(&self) -> bool {
        self.is_multi()
            && self
                .typing
                .as_ref()
                .is_some_and(|t| t.start_time().is_none())
            && self.multiplayer_start_at_unix_ms.is_some()
    }

    pub fn multiplayer_countdown_secs(&self) -> Option<u64> {
        if !self.is_waiting_for_multiplayer_start() {
            return None;
        }
        let now = now_unix_ms();
        self.multiplayer_start_at_unix_ms
            .map(|start_at| start_at.saturating_sub(now).div_ceil(1000))
    }
}
