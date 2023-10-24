use std::collections::VecDeque;

use crate::{md::Markdown, tag::TagValue};

pub struct TagBuilder {
    pub md: Markdown,
    pub in_progress: String,
    pub completed: VecDeque<TagValue>,
}

impl TagBuilder {
    pub fn to_values(&mut self) -> VecDeque<TagValue> {
        let mut values = VecDeque::new();
        values.push_back(TagValue::Text(self.md.to_string()));
        values.append(&mut self.completed);
        if !self.in_progress.is_empty() {
            values.push_back(TagValue::Text(std::mem::take(&mut self.in_progress)));
        }
        values
    }
}

impl Default for TagBuilder {
    fn default() -> Self {
        Self {
            md: Markdown::Line,
            in_progress: String::new(),
            completed: Default::default(),
        }
    }
}

impl From<Markdown> for TagBuilder {
    fn from(value: Markdown) -> Self {
        Self {
            md: value,
            in_progress: String::new(),
            completed: Default::default(),
        }
    }
}
