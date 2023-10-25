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
        if self.completed.is_empty() {
            let mut text = self.md.to_string();
            text.push_str(&self.in_progress);
            values.push_back(TagValue::Text(text));
        } else {
            values.push_back(TagValue::Text(self.md.to_string()));
            values.append(&mut self.completed);
            if !self.in_progress.is_empty() {
                values.push_back(TagValue::Text(std::mem::take(&mut self.in_progress)));
            }
        }
        values
    }

    pub fn append_values(&mut self, mut values: VecDeque<TagValue>) {
        while let Some(v) = values.pop_front() {
            self.append_value(v);
        }
    }

    pub fn append_value(&mut self, v: TagValue) {
        match v {
            TagValue::Text(v2) => {
                if let Some(TagValue::Text(s)) = self.completed.back_mut() {
                    s.push_str(&v2);
                } else {
                    self.completed.push_back(TagValue::Text(v2));
                }
            }
            _ => self.completed.push_back(v),
        }
    }

    pub fn save_progress(&mut self) {
        if !self.in_progress.is_empty() {
            let p = std::mem::take(&mut self.in_progress);
            self.completed.push_back(TagValue::Text(p));
        }
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
