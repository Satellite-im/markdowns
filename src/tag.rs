use std::collections::VecDeque;

#[derive(Clone, Eq, PartialEq, Debug)]
pub enum TagType {
    // the root node
    Paragraph,
    NewLine,
    Bold,
    Italics,
    Strikethrough,
    H1,
    H2,
    H3,
    H4,
    H5,
    H6,
    BlockQuote,
    Code(String),
}

#[derive(Clone, Eq, PartialEq, Debug)]
pub enum TagValue {
    // probably a span
    Text(String),
    Tag(Tag),
}

#[derive(Clone, Eq, PartialEq, Debug)]
pub struct Tag {
    pub r#type: TagType,
    pub values: VecDeque<TagValue>,
}

impl Tag {
    pub fn add_text(&mut self, text: &str) {
        if !text.is_empty() {
            if let Some(TagValue::Text(v)) = self.values.back_mut() {
                v.push_str(text);
            } else {
                self.values.push_back(TagValue::Text(text.into()));
            }
        }
    }

    pub fn add_tag(&mut self, tag: Tag) {
        self.values.push_back(TagValue::Tag(tag));
    }

    pub fn add_tags(&mut self, mut tags: VecDeque<Tag>) {
        for t in tags.drain(..) {
            self.add_tag(t);
        }
    }

    pub fn append_values(&mut self, mut values: VecDeque<TagValue>) {
        while let Some(v) = values.pop_front() {
            match v {
                TagValue::Text(v2) => {
                    if let Some(TagValue::Text(s)) = self.values.back_mut() {
                        s.push_str(&v2);
                    } else {
                        self.values.push_back(TagValue::Text(v2));
                    }
                }
                _ => self.values.push_back(v),
            }
        }
    }

    // makes unit testing faster
    pub fn add_tag_w_text(&mut self, tag_type: TagType, text: &str) {
        let mut n: Tag = tag_type.into();
        n.add_text(text);
        self.add_tag(n);
    }
}

impl From<TagType> for Tag {
    fn from(value: TagType) -> Self {
        Self {
            r#type: value,
            values: Default::default(),
        }
    }
}

impl Default for Tag {
    fn default() -> Self {
        Self {
            r#type: TagType::Paragraph,
            values: Default::default(),
        }
    }
}
