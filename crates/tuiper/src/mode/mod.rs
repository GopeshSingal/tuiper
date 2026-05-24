use std::fs;
use std::io;
use std::path::PathBuf;

use dirs::config_dir;
use serde::{Deserialize, Serialize};
use serde_json::{from_str, to_string_pretty};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum RaceMode {
    Time,
    Words,
}

pub const WORDS_PRESETS: [u32; 4] = [10, 25, 50, 100];
pub const TIME_PRESETS: [u32; 3] = [15, 30, 60];

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct ModeConfig {
    pub mode: RaceMode,
    pub words_preset_idx: u8,
    pub time_preset_idx: u8,
}

impl Default for ModeConfig {
    fn default() -> Self {
        Self {
            mode: RaceMode::Time,
            words_preset_idx: 1,
            time_preset_idx: 1,
        }
    }
}

impl ModeConfig {
    fn clamp_indices(&mut self) {
        self.words_preset_idx = self
            .words_preset_idx
            .min((WORDS_PRESETS.len() - 1) as u8);
        self.time_preset_idx = self
            .time_preset_idx
            .min((TIME_PRESETS.len() - 1) as u8);
    }
}

pub fn mode_config_path() -> Option<PathBuf> {
    config_dir().map(|mut p| {
        p.push("tuiper/mode_config.json");
        p
    })
}

pub fn load() -> ModeConfig {
    let mut config: ModeConfig = mode_config_path()
        .and_then(|path| fs::read_to_string(path).ok())
        .and_then(|data_str| from_str(&data_str).ok())
        .unwrap_or_default();
    config.clamp_indices();
    config
}

pub fn save(config: &ModeConfig) -> io::Result<()> {
    let Some(path) = mode_config_path() else {
        return Ok(());
    };
    if let Some(dir) = path.parent() {
        fs::create_dir_all(dir)?;
    }
    let json =
        to_string_pretty(config).map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
    fs::write(path, json)
}
