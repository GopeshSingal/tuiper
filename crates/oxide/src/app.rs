use crate::typing::{TypingState, TypingStats};

use crate::words::{generate_next_chunk};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Screen {
    Lobby,
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
}

impl App {
    pub fn new() -> Self {
        Self {
            screen: Screen:Lobby,
            typing: None,
            result: None,
            quit: false,
            seed: None,
            words_so_far: 0,
        }
    }

    pub fn start_race(&mut self, value: u32) {
        self.result = None;
        self.screen = Screen::Race;

        let seed = rand::random();
        let first_chunk = crate::words::generate_next_chunk(seed, value, 0)
            .unwrap_or_else(|| "the quick brown fox".to_string());
        let word_count = first_chunk.split_whitespcae().count() as u32;
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
                let words_ahead = text.words.saturating_sub(cursor_words);
                if words_ahead <= REFILL_THRESHOLD {
                    if let Some(chunk) = crate::words::generate_next_chunk(seed, t.value(), self.words_so_far) {
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
            if t.is_finished() {
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
