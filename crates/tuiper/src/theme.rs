use std::collections::HashMap;
use std::fs;
use std::io;
use std::path::PathBuf;

use ratatui::style::{Color};
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use strum::{Display, EnumIter, IntoEnumIterator};

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
}

mod color_map_serde {
    use super::*;

    pub fn serialize<S>(map: &HashMap<ThemeField, Color>, serializer: S) -> Result<S::Ok, S::Error>
    where S: Serializer {
        // TODO: implement this
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<HashMap<ThemeField, Color>, D::Error>
    where D: Deserializer<'de> {
        //TODO: implement this
    }
}
