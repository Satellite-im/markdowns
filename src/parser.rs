use std::collections::VecDeque;

use crate::{
    md::Markdown,
    tag::{Tag, TagType, TagValue},
    tag_builder::TagBuilder,
};

pub struct Parser {
    // everything gets appended to root.values
    root: Tag,
    builders: VecDeque<TagBuilder>,
}

impl Parser {
    pub fn new() -> Self {
        Self {
            root: TagType::Paragraph.into(),
            builders: Default::default(),
        }
    }

    pub fn finish(&mut self) -> Tag {
        while let Some(mut builder) = self.builders.pop_back() {
            let mut values = builder.to_values();
            if let Some(prev) = self.builders.back_mut() {
                prev.completed.append(&mut values);
            } else {
                self.root.values.append(&mut values);
            }
        }

        self.root.values.retain(|x| match x {
            TagValue::Text(s) => !s.is_empty(),
            _ => true,
        });

        let mut r = std::mem::take(&mut self.root);
        r.values = r.values.drain(..).rev().collect();
        r
    }

    pub fn process(&mut self, c: char) {
        if self.builders.is_empty() {
            self.builders.push_back(Default::default());
        }

        let (prev_md, prev_empty) = self
            .builders
            .back()
            .map(|x| {
                (
                    x.md.clone(),
                    x.in_progress.is_empty() && x.completed.is_empty(),
                )
            })
            .unwrap(); // this never fails per the previous statement.

        match prev_md {
            Markdown::Star => match c {
                '*' => {
                    let prev = self.builders.pop_back().unwrap();
                    if prev_empty {
                        // this is the 2nd prev (at least it was before the pop_back())
                        if self.prev_matches(Markdown::DoubleStar) {
                            let p2 = self.builders.pop_back().unwrap();
                            let mut new_tag = Tag::from(TagType::Bold);
                            new_tag.add_tag_values(p2.completed);
                            new_tag.add_text(&p2.in_progress);
                            self.bubble_tag(new_tag);
                        } else {
                            self.push_md(Markdown::DoubleStar);
                        }
                    } else {
                        let mut new_tag = Tag::from(TagType::Italics);
                        new_tag.add_tag_values(prev.completed);
                        new_tag.add_text(&prev.in_progress);
                        self.bubble_tag(new_tag);
                    }
                }
                _ => self.push_char(c),
            },
            _ => match c {
                '*' => self.push_md(Markdown::Star),
                _ => self.push_char(c),
            },
        }
    }

    fn bubble_tag(&mut self, tag: Tag) {
        if let Some(builder) = self.builders.back_mut() {
            builder.completed.push_back(TagValue::Tag(tag));
        } else {
            self.root.add_tag(tag);
        }
    }

    fn push_char(&mut self, c: char) {
        if let Some(builder) = self.builders.back_mut() {
            builder.in_progress.push(c);
        } else {
            unreachable!()
        }
    }

    fn push_md(&mut self, md: Markdown) {
        self.builders.push_back(md.into());
    }

    fn prev_matches(&self, md: Markdown) -> bool {
        self.builders.back().map(|x| x.md == md).unwrap_or_default()
    }
}

pub fn text_to_html2(text: &str) -> Tag {
    let mut parser = Parser::new();
    for c in text.chars() {
        parser.process(c);
    }
    parser.finish()
}

#[cfg(test)]
mod test {
    use crate::tag::TagType;

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
