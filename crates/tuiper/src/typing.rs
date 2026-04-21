use std::time::Instant;

use crate::app::RaceMode;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CharState {
    Untyped,
    Correct,
    Incorrect,
    Current,
}

#[derive(Debug, Clone, Default)]
pub struct TypingStats {
    pub wpm: f64,
    pub raw_wpm: f64,
    pub accuracy: f64,
    pub consistency: f64,
    pub chars_typed: u32,
    pub correct_chars: u32,
    pub duration_secs: f64,
    pub wpm_history: Vec<(f64, f64)>,
}

pub struct TypingState {
    text: String,
    input: String,
    start_time: Option<Instant>,
    value: u32,
    total_keypresses: u32,
    correct_keypresses: u32,
    raw_wpm_samples: Vec<f64>,
    wpm_samples: Vec<(f64, f64)>,
    last_sample_time: Option<Instant>,
}

impl TypingState {
    pub fn new(text: String, value: u32) -> Self {
        Self {
            text,
            input: String::new(),
            start_time: None,
            value,
            total_keypresses: 0,
            correct_keypresses: 0,
            raw_wpm_samples: Vec::new(),
            wpm_samples: Vec::new(),
            last_sample_time: None,
        }
    }

    pub fn start(&mut self) {
        if self.start_time.is_none() {
            self.start_time = Some(Instant::now());
            self.last_sample_time = self.start_time;
        }
    }

    pub fn text(&self) -> &str {
        &self.text
    }

    pub fn input(&self) -> &str {
        &self.input
    }

    pub fn cursor(&self) -> usize {
        self.input.chars().count()
    }

    pub fn start_time(&self) -> Option<Instant> {
        self.start_time
    }

    pub fn value(&self) -> u32 {
        self.value
    }

    pub fn words_typed(&self) -> u32 {
        self.input.split_whitespace().count() as u32
    }

    pub fn append_text(&mut self, more: &str) {
        if more.is_empty() {
            return;
        }
        if !self.text.is_empty() && !self.text.ends_with(' ') {
            self.text.push(' '); // Add a space character at the end if needed
        }
        self.text.push_str(more);
    }

    pub fn is_finished(&self, mode: RaceMode) -> bool {
        let Some(start) = self.start_time else {
            return false;
        };
        let elapsed = start.elapsed().as_secs_f64();
        let cursor = self.cursor();
        let text_len = self.text.chars().count();

        match mode {
            RaceMode::Time => elapsed >= self.value as f64,
            RaceMode::Words => !self.has_unfixed_error() && cursor >= text_len,
        }
    }

    pub fn progress_ratio(&self, mode: RaceMode) -> f64 {
        match mode {
            RaceMode::Time => {
                let v = self.value as f64;
                if v <= 0.0 {
                    return 1.0;
                }
                (self.elapsed_secs() / v).min(1.0)
            }
            RaceMode::Words => {
                let goal = self.value;
                if goal == 0 {
                    return 1.0;
                }
                (self.words_typed().min(goal) as f64 / goal as f64).min(1.0)
            }
        }
    }

    pub fn correct_chars(&self) -> usize {
        let expected: Vec<char> = self.text.chars().collect();
        let actual: Vec<char> = self.input.chars().collect();
        let len = expected.len().min(actual.len());
        let mut count = 0;
        for i in 0..len {
            if expected[i] == actual[i] {
                count += 1;
            } else {
                break;
            }
        }
        count
    }

    pub fn elapsed_secs(&self) -> f64 {
        self.start_time
            .map(|t| t.elapsed().as_secs_f64())
            .unwrap_or(0.0)
    }

    pub fn wpm(&self) -> f64 {
        let mins = self.elapsed_secs() / 60.0;
        if mins <= 0.0 {
            return 0.0;
        }
        let correct = self.correct_chars() as f64;
        (correct / 5.0) / mins
    }

    pub fn raw_wpm(&self) -> f64 {
        let mins = self.elapsed_secs() / 60.0;
        if mins <= 0.0 {
            return 0.0;
        }
        let typed = self.cursor() as f64;
        (typed / 5.0) / mins
    }

    pub fn accuracy(&self) -> f64 {
        if self.total_keypresses == 0 {
            return 100.0;
        }
        100.0 * self.correct_keypresses as f64 / self.total_keypresses as f64
    }

    pub fn consistency(&self) -> f64 {
        if self.raw_wpm_samples.len() < 2 {
            return 100.0;
        }
        let mean = self.raw_wpm_samples.iter().sum::<f64>() / self.raw_wpm_samples.len() as f64;
        let variance = self
            .raw_wpm_samples
            .iter()
            .map(|x| (x - mean).powi(2))
            .sum::<f64>()
            / self.raw_wpm_samples.len() as f64;
        let stddev = variance.sqrt();
        (100.0 - (stddev * 2.0).min(100.0)).max(0.0)
    }

    pub fn sample_raw_wpm(&mut self) {
        let Some(start) = self.start_time else { return };
        let elapsed = start.elapsed();
        if elapsed.as_secs_f64() < 0.5 {
            return;
        }
        let do_sample = self
            .last_sample_time
            .map(|t| t.elapsed().as_secs_f64() >= 0.5)
            .unwrap_or(true);
        if do_sample {
            self.raw_wpm_samples.push(self.raw_wpm());
            self.wpm_samples.push((elapsed.as_secs_f64(), self.wpm()));
            self.last_sample_time = Some(Instant::now());
        }
    }

    pub fn type_char(&mut self, c: char) -> bool {
        self.start();
        let cursor = self.cursor();
        let expected: Vec<char> = self.text.chars().collect();
        if cursor >= expected.len() {
            return false;
        }

        self.total_keypresses += 1;
        if expected[cursor] == c {
            self.correct_keypresses += 1;
        }
        self.input.push(c);
        true
    }

    pub fn backspace(&mut self) -> bool {
        if self.input.is_empty() {
            return false;
        }
        self.start();
        self.input.pop();
        true
    }

    pub fn backspace_word(&mut self) -> bool {
        if self.input.is_empty() {
            return false;
        }
        self.start();

        let mut removed_any = false;

        while let Some(last) = self.input.chars().last() {
            if !last.is_whitespace() {
                break;
            }
            self.input.pop();
            removed_any = true;
        }

        while let Some(last) = self.input.chars().last() {
            if last.is_whitespace() {
                break;
            }
            self.input.pop();
            removed_any = true;
        }

        removed_any
    }

    pub fn has_unfixed_error(&self) -> bool {
        let expected: Vec<char> = self.text.chars().collect();
        let actual: Vec<char> = self.input.chars().collect();
        let cursor = actual.len();
        for i in 0..cursor {
            if actual[i] != expected[i] {
                return true;
            }
        }
        false
    }

    pub fn char_states(&self) -> Vec<CharState> {
        let expected: Vec<char> = self.text.chars().collect();
        let actual: Vec<char> = self.input.chars().collect();
        let cursor = actual.len();
        let mut first_error = None;
        for i in 0..cursor {
            if actual[i] != expected[i] {
                first_error = Some(i);
                break;
            }
        }

        let mut out = Vec::with_capacity(expected.len());
        for (i, &e) in expected.iter().enumerate() {
            let state = if let Some(k) = first_error {
                if i < k {
                    CharState::Correct
                } else if i < cursor {
                    CharState::Incorrect
                } else if i == cursor {
                    CharState::Current
                } else {
                    CharState::Untyped
                }
            } else if i < cursor {
                if e == actual[i] {
                    CharState::Correct
                } else {
                    CharState::Incorrect
                }
            } else if i == cursor {
                CharState::Current
            } else {
                CharState::Untyped
            };
            out.push(state);
        }
        out
    }

    pub fn final_stats(&self) -> TypingStats {
        let duration_secs = self.elapsed_secs();
        let final_wpm = self.wpm();
        let mut wpm_history = self.wpm_samples.clone();
        if wpm_history
            .last()
            .is_none_or(|(secs, wpm)| *secs < duration_secs || (*wpm - final_wpm).abs() > 0.1)
        {
            wpm_history.push((duration_secs, final_wpm));
        }

        TypingStats {
            wpm: final_wpm,
            raw_wpm: self.raw_wpm(),
            accuracy: self.accuracy(),
            consistency: self.consistency(),
            chars_typed: self.cursor() as u32,
            correct_chars: self.correct_chars() as u32,
            duration_secs,
            wpm_history,
        }
    }
}
