use std::collections::VecDeque;

use crate::md::Markdown;

struct Parser {
    // everything gets appended to root.values
    root: Tag,
    builders: VecDeque<TagBuilder>,
}

struct TagBuilder {
    md: Markdown,
    in_progress: String,
    completed: VecDeque<Tag>,
}

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
    r#type: TagType,
    values: VecDeque<TagValue>,
}

impl Tag {
    pub fn add_text(&mut self, text: &str) {
        self.values.push_back(TagValue::Text(text.into()));
    }

    pub fn add_tag(&mut self, tag: Tag) {
        self.values.push_back(TagValue::Tag(tag));
    }

    // makes unit testing faster
    pub fn add_tag_w_text(&mut self, tag_type: TagType, text: &str) {
        let mut n: Tag = tag_type.into();
        n.add_text(text.into());
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

pub fn text_to_html2(text: &str) -> Tag {
    todo!()
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_plain_test() {
        let mut expected = Tag::from(TagType::Paragraph);
        expected.add_text("abcd");

        let test = text_to_html2("abcd");
        assert_eq!(test, expected);
    }

    #[test]
    fn test_plain_bold1() {
        let test = text_to_html2("abcd**bold**");
        let mut expected = Tag::from(TagType::Paragraph);
        expected.add_text("abcd");
        expected.add_tag_w_text(TagType::Bold, "bold");

        assert_eq!(test, expected);
    }

    #[test]
    fn test_bold1() {
        let test = text_to_html2("**bold**");
        let mut expected = Tag::from(TagType::Paragraph);
        expected.add_tag_w_text(TagType::Bold, "bold");
        assert_eq!(test, expected);
    }

    #[test]
    fn test_plain_italics() {
        let test = text_to_html2("abcd*italics*");
        let mut expected = Tag::from(TagType::Paragraph);
        expected.add_text("abcd");
        expected.add_tag_w_text(TagType::Italics, "italics");
        assert_eq!(test, expected);
    }

    #[test]
    fn test_italics1() {
        let test = text_to_html2("*italics*");
        let mut expected = Tag::from(TagType::Paragraph);
        expected.add_tag_w_text(TagType::Italics, "italics");
        assert_eq!(test, expected);
    }
}
