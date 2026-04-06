mod constants;

use std::collections::HashMap;
use std::fs;
use std::io;
use std::path::PathBuf;

use dirs::{config_dir};
use ratatui::style::{Color};
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use serde_json::{from_str, to_string_pretty};
use strum::{Display, EnumIter, IntoEnumIterator};

use constants::TAILWIND_GRID;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, EnumIter, Display)]
pub enum ThemeField {
    #[strum(serialize = "window background")]
    WindowBg,
    #[strum(serialize = "untyped text")]
    Untyped,
    #[strum(serialize = "typed correct")]
    TypedCorrect,
    #[strum(serialize = "typed incorrect")]
    TypedIncorrect,
    #[strum(serialize = "cursor background")]
    CursorBg,
    #[strum(serialize = "cursor foreground")]
    CursorFg,
    #[strum(serialize = "cursor foreground (error)")]
    CursorFgError,

    // multiplayer
    #[strum(serialize = "opponent cursor background")]
    OppCursorBg,
    #[strum(serialize = "opponent cursor foreground")]
    OppCursorFg,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Theme {
    #[serde(flatten)]
    #[serde(with = "color_map_serde")]
    pub fields: HashMap<ThemeField, Color>,
}

impl Default for Theme {
    fn default() -> Self {
        let mut fields = HashMap::new();
        fields.insert(ThemeField::WindowBg, Color::Rgb(16, 16, 16));
        fields.insert(ThemeField::Untyped, Color::DarkGray);
        fields.insert(ThemeField::TypedCorrect, Color::Green);
        fields.insert(ThemeField::TypedIncorrect, Color::Red);
        fields.insert(ThemeField::CursorBg, Color::DarkGray);
        fields.insert(ThemeField::CursorFg, Color::White);
        fields.insert(ThemeField::CursorFgError, Color::Red);

        // multiplayer
        fields.insert(ThemeField::OppCursorBg, Color::LightMagenta);
        fields.insert(ThemeField::OppCursorFg, Color::White);

        Self { fields }
    }
}

impl Theme {
    pub fn get(&self, field: ThemeField) -> Color {
        *self.fields.get(&field).unwrap_or(&Color::Reset)
    }

    pub fn set(&mut self, field: ThemeField, c: Color) {
        self.fields.insert(field, c);
    }

    pub fn cycle_palette(&mut self, field: ThemeField, delta: isize) {
        let (r, c) = self.get_grid_pos(field).unwrap_or((0, 5));
        let next_r = (r as isize + delta).rem_euclid(22) as usize;

        self.set(field, TAILWIND_GRID[next_r][5]);
    }

    pub fn cycle_shade(&mut self, field: ThemeField, delta: isize) {
        let (r, c) = self.get_grid_pos(field).unwrap_or((0, 5));
        let next_c = (c as isize + delta).rem_euclid(11) as usize;

        self.set(field, TAILWIND_GRID[r][next_c]);
    }

    fn get_grid_pos(&self, field: ThemeField) -> Option<(usize, usize)> {
        let curr = self.get(field);
        for (r_idx, row) in TAILWIND_GRID.iter().enumerate() {
            if let Some(c_idx) = row.iter().position(|&c| c == curr) {
                return Some((r_idx, c_idx));
            }
        }
        None
    }
}

pub fn theme_config_path() -> Option<PathBuf> {
    config_dir().map(|mut p| {
        p.push("tuiper/theme_config.json");
        p
    })
}

pub fn load() -> Theme {
    theme_config_path()
        .and_then(|path| fs::read_to_string(path).ok())
        .and_then(|data_str| from_str(&data_str).ok())
        .unwrap_or_else(Theme::default)
}

pub fn save(theme: &Theme) -> io::Result<()> {
    let Some(path) = theme_config_path() else { return Ok(()); };
    if let Some(dir) = path.parent() {
        fs::create_dir_all(dir)?;
    }
    let json = to_string_pretty(theme)
        .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
    fs::write(path, json)
}

mod color_map_serde {
    use super::*;

    pub fn serialize<S>(map: &HashMap<ThemeField, Color>, serializer: S) -> Result<S::Ok, S::Error>
    where S: Serializer {
        use serde::ser::SerializeMap;
        let mut state = serializer.serialize_map(Some(map.len()))?;
        for (k, v) in map {
            state.serialize_entry(&format!("{:?}", k), &color_to_hex(*v))?;
        }
        state.end()
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<HashMap<ThemeField, Color>, D::Error>
    where D: Deserializer<'de> {
        let map_raw = HashMap::<String, String>::deserialize(deserializer)?;
        let mut map = HashMap::new();
        for field in ThemeField::iter() {
            if let Some(hex) = map_raw.get(&format!("{:?}", field)) {
                map.insert(field, hex_to_color(hex));
            }
        }
        Ok(map)
    }

    fn color_to_hex(c: Color) -> String {
        if let Color::Rgb(r, g, b) = c {
            format!("#{:02x}{:02x}{:02x}", r, g, b)
        } else {
            "#ffffff".to_string()
        }
    }

    fn hex_to_color(s: &str) -> Color {
        if s.starts_with('#') && s.len() == 7 {
            let r = u8::from_str_radix(&s[1..3], 16).unwrap_or(255);
            let g = u8::from_str_radix(&s[3..5], 16).unwrap_or(255);
            let b = u8::from_str_radix(&s[5..7], 16).unwrap_or(255);
            Color::Rgb(r, g, b)
        } else {
            Color::Rgb(255, 255, 255)
        }
    }
}
