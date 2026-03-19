use crate::typing::{TypingState, TypingStats};
use crate::words::{generate_next_chunk};

use protocols::{ClientMessage, RaceResults};

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
    pub opponent_wpm: f64,
    pub opponent_chars: u32,
    pub last_progress_sent: f64,
    pub race_results: Option<RaceResults>,
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
            opponent_wpm: 0.0,
            opponent_chars: 0,
            last_progress_sent: 0.0,
            race_results: None,
        }
    }

    pub fn start_race(&mut self, value: u32) {
        self.result = None;
        self.screen = Screen::Race;

        let seed = rand::random();
        let first_chunk = generate_next_chunk(seed, value, 0)
            .unwrap_or_else(|| "the quick brown fox".to_string());
        let word_count = first_chunk.split_whitespace().count() as u32;
        self.typing = Some(TypingState::new(first_chunk, value));
        self.seed = Some(seed);
        self.words_so_far = word_count;
    }

    pub fn start_multiplayer(&mut self, seed: u64, value: u32) {
        self.result = None;
        self.race_results = None;
        self.opponent_wpm = 0.0;
        self.opponent_chars = 0;
        self.last_progress_sent = 0.0;
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
            
            // Send progress over WebSocket to opponent
            let elapsed = t.elapsed_secs();
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

            if t.is_finished() {
                if let Some(ref tx) = self.ws_tx {
                    let stats = t.final_stats();
                    let _ = tx.send(ClientMessage::RaceFinished {
                        wpm: stats.wpm,
                        accuracy: stats.accuracy,
                        consistency: stats.consistency,
                        chars_typed: stats.chars_typed,
                    });
                }
                self.result = Some(t.final_stats());
                self.typing = None;
                self.seed = None;
                self.words_so_far = 0;
                self.screen = Screen::Results;
            }
        }
    }

    pub fn typing(&self) -> Option<&TypingState> {
        self.typing.as_ref()
    }

    pub fn result(&self) -> Option<&TypingStats> {
        self.result.as_ref()
    }
}
