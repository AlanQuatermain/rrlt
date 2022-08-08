use super::{append_entry, LogFragment};
use crate::prelude::*;

pub struct Logger {
    current_color: RGB,
    fragments: Vec<LogFragment>,
}

impl Logger {
    pub fn new() -> Self {
        Logger {
            current_color: RGB::named(WHITE),
            fragments: Vec::new(),
        }
    }

    pub fn color<COLOR>(mut self, color: COLOR) -> Self
    where
        COLOR: Into<RGB>,
    {
        self.current_color = color.into();
        self
    }

    pub fn append<T: ToString>(mut self, text: T) -> Self {
        self.fragments.push(LogFragment {
            color: self.current_color,
            text: text.to_string(),
        });
        self
    }

    pub fn npc_name<T: ToString>(mut self, text: T) -> Self {
        self.fragments.push(LogFragment {
            color: RGB::named(YELLOW),
            text: text.to_string(),
        });
        self
    }

    pub fn item_name<T: ToString>(mut self, text: T) -> Self {
        self.fragments.push(LogFragment {
            color: RGB::named(CYAN),
            text: text.to_string(),
        });
        self
    }

    pub fn damage(mut self, damage: i32) -> Self {
        self.fragments.push(LogFragment {
            color: RGB::named(YELLOW),
            text: format!("{}", damage).to_string(),
        });
        self
    }

    pub fn log(self) {
        append_entry(self.fragments)
    }
}
