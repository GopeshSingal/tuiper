use crate::typing::{TypingState, TypingStats};
use crate::words::{generate_next_chunk};

use common::now_unix_ms;
use protocols::{ClientMessage, RaceResults, ServerMessage};
use protocols::ServerMessage::*;

use std::sync::mpsc;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Screen {
    Lobby,
    Queue,
    Race,
    Results,
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
    pub opponent_wpm: f64,
    pub opponent_chars: u32,
    pub last_progress_sent: f64,
    pub race_results: Option<RaceResults>,
    pub multiplayer_start_at_unix_ms: Option<u64>,
}

impl App {
    pub fn new() -> Self {
        Self {
            screen: Screen::Lobby,
            typing: None,
            result: None,
            quit: false,

            // streaming
            seed: None,
            words_so_far: 0,

            // multiplayer
            ws_tx: None,
            multiplayer_race: false,
            opponent_wpm: 0.0,
            opponent_chars: 0,
            last_progress_sent: 0.0,
            race_results: None,
            multiplayer_start_at_unix_ms: None,
        }
    }

    pub fn handle_server_message(&mut self, msg: ServerMessage) {
        match msg {
            Queue => {
                self.screen = Screen::Queue;
            }
            RaceStart { race_id: _, value, seed, start_at_unix_ms } => {
                self.start_multiplayer(seed, value, start_at_unix_ms);
            }
            OpponentProgress { wpm, chars_typed } => {
                if self.multiplayer_race {
                    self.opponent_wpm = wpm;
                    self.opponent_chars = chars_typed;
                }
            }
            RaceEnd { results } => {
                self.multiplayer_race = false;
                self.race_results = Some(results);
                self.typing = None;
                self.result = None;
                self.multiplayer_start_at_unix_ms = None;
                self.screen = Screen::Results;
            }
            Error { message: _ } => {
                self.multiplayer_race = false;
                self.disconnect_websocket();
                self.screen = Screen::Lobby;
            }
        }
    }

    pub fn start_race(&mut self, value: u32) {
        self.multiplayer_race = false;
        self.result = None;
        self.race_results = None;
        self.screen = Screen::Race;
        self.multiplayer_start_at_unix_ms = None;

        let seed = rand::random();
        let first_chunk = generate_next_chunk(seed, value, 0)
            .unwrap_or_else(|| "the quick brown fox".to_string());
        let word_count = first_chunk.split_whitespace().count() as u32;
        self.typing = Some(TypingState::new(first_chunk, value));
        self.seed = Some(seed);
        self.words_so_far = word_count;
    }

    pub fn start_multiplayer(&mut self, seed: u64, value: u32, start_at_unix_ms: u64) {
        self.multiplayer_race = true;
        self.result = None;
        self.race_results = None;
        self.opponent_wpm = 0.0;
        self.opponent_chars = 0;
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
                }
            }
            t.sample_raw_wpm();

            if let Some(seed) = self.seed {
                let text_words = t.text().split_whitespace().count();
                let cursor_words = t.input().split_whitespace().count();
                let words_ahead = text_words.saturating_sub(cursor_words);
                if words_ahead <= REFILL_THRESHOLD {
                    if let Some(chunk) = generate_next_chunk(seed, t.value(), self.words_so_far) {
                        if !chunk.is_empty() {
                            t.append_text(&chunk);
                            self.words_so_far += chunk.split_whitespace().count() as u32;
                        }
                    } else {
                        self.seed = None;
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

            if t.is_finished() {
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
            && self.typing.as_ref().is_some_and(|t| t.start_time().is_none())
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
