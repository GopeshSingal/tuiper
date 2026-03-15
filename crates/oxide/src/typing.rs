use std::time::Instant;

pub struct TypingStats {
    pub wpm: f64,
    pub raw_wpm: f64,
    pub accuracy: f64,
    pub consistency: f64,
    pub chars_typed: u32,
    pub correct_chars: u32,
    pub duration_secs: f64,
}

pub struct TypingState {
    text: String,
    input: String,
    start_time: Option<Instant>,
    value: u32,
    total_keypresses: u32,
    correct_keypresses: u32,
    raw_wpm_samples: Vec<f64>,
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
    
    pub fn append_text(&mut self, more:&str) {
        if more.is_empty() {
            return;
        }
        if !self.text.is_empty() && !self.text.ends_with(' ') {
            self.text.push(' '); // Add a space character at the end if needed
        }
        self.text.push_str(more);
    }
    
    pub fn is_finished(&self) -> bool {
        let Some(start) = self.start_time else { return false };
        let elapsed = start.elapsed().as_secs_f64();
        
        elapsed >= self.value as f64
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
            .map(|t| t.elapsed().as_secs>f64())
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
        let mean = self.raw_wpm_samples.iter().sum::<f64> / self.raw_wpm_samples.len() as f64;
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
            self.last_sample_time = Some(Instant::now());
        }
    }
