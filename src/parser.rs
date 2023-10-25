use std::collections::VecDeque;

use crate::{
    md::Markdown,
    tag::{Tag, TagType, TagValue},
    tag_builder::TagBuilder,
};

const LANGUAGE_TEXT: &str = "text";

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
        while let Some(builder) = self.builders.pop_back() {
            self.handle_builder(builder);
        }

        self.root.values.retain(|x| match x {
            TagValue::Text(s) => !s.is_empty(),
            _ => true,
        });

        // todo: combine blockquotes

        std::mem::take(&mut self.root)
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
                            new_tag.append_values(p2.completed);
                            new_tag.add_text(&p2.in_progress);
                            self.bubble_tag(new_tag);
                        } else {
                            self.push_md(Markdown::DoubleStar);
                        }
                    } else {
                        let mut new_tag = Tag::from(TagType::Italics);
                        new_tag.append_values(prev.completed);
                        new_tag.add_text(&prev.in_progress);
                        self.bubble_tag(new_tag);
                    }
                }
                _ => self.push_char(c),
            },
            Markdown::DoubleStar if c == '*' => {
                // triple star - means nothing.
                if prev_empty {
                    self.builders.pop_back();
                    self.push_char('*');
                    self.push_md(Markdown::DoubleStar);
                } else {
                    self.push_md(Markdown::Star);
                }
            }
            Markdown::Underscore => match c {
                '_' => {
                    let prev = self.builders.pop_back().unwrap();
                    if prev_empty {
                        // this is the 2nd prev (at least it was before the pop_back())
                        if self.prev_matches(Markdown::DoubleUnderscore) {
                            let p2 = self.builders.pop_back().unwrap();
                            let mut new_tag = Tag::from(TagType::Bold);
                            new_tag.append_values(p2.completed);
                            new_tag.add_text(&p2.in_progress);
                            self.bubble_tag(new_tag);
                        } else {
                            self.push_md(Markdown::DoubleUnderscore);
                        }
                    } else {
                        let mut new_tag = Tag::from(TagType::Italics);
                        new_tag.append_values(prev.completed);
                        new_tag.add_text(&prev.in_progress);
                        self.bubble_tag(new_tag);
                    }
                }
                _ => self.push_char(c),
            },
            Markdown::DoubleUnderscore if c == '_' => {
                // triple underscore - means nothing.
                if prev_empty {
                    self.builders.pop_back();
                    self.push_char('_');
                    self.push_md(Markdown::DoubleUnderscore);
                } else {
                    self.push_md(Markdown::Underscore);
                }
            }
            Markdown::TripleBacktick => match c {
                '`' => {
                    // 4 backticks in a row
                    if prev_empty {
                        self.builders.pop_back();
                        self.push_char('`');
                        self.push_md(Markdown::TripleBacktick);
                    } else if self.is_prev_backslash() {
                        self.push_char(c);
                    } else {
                        self.push_md(Markdown::Backtick);
                    }
                }
                _ => self.push_char(c),
            },
            Markdown::DoubleBacktick => match c {
                '`' => {
                    if prev_empty {
                        if let Some(prev) = self.builders.back_mut() {
                            prev.md = Markdown::TripleBacktick;
                        } else {
                            unreachable!();
                        }
                        let prev = self.builders.pop_back().unwrap();
                        if self.prev_matches(Markdown::TripleBacktick) {
                            let text = self.get_text_from_code_block();
                            let (language, text) = get_language(&text);
                            let mut new_tag = Tag::from(TagType::Code(language));
                            new_tag.add_text(&text);
                            self.bubble_tag(new_tag);
                        } else {
                            self.builders.push_back(prev);
                        }
                    } else if self.is_prev_backslash() {
                        self.push_char(c);
                    } else {
                        // double backticks, some text, then another backtick
                        let prev = self.builders.pop_back().unwrap();
                        self.push_char('`');
                        let mut new_tag = Tag::from(TagType::Code(LANGUAGE_TEXT.into()));
                        debug_assert!(prev.completed.is_empty());
                        new_tag.add_text(&prev.in_progress);
                        self.bubble_tag(new_tag);
                    }
                }
                _ => self.push_char(c),
            },
            Markdown::Backtick if c != '\n' => match c {
                '`' => {
                    if prev_empty {
                        if let Some(prev) = self.builders.back_mut() {
                            prev.md = Markdown::DoubleBacktick;
                        } else {
                            unreachable!();
                        }
                    } else if self.is_prev_backslash() {
                        self.push_char(c);
                    } else {
                        let prev = self.builders.pop_back().unwrap();
                        let mut new_tag = Tag::from(TagType::Code(LANGUAGE_TEXT.into()));
                        // prev.completed should be empty
                        debug_assert!(prev.completed.is_empty());
                        new_tag.add_text(&prev.in_progress);
                        self.bubble_tag(new_tag);
                    }
                }
                _ => self.push_char(c),
            },
            Markdown::Tilde => match c {
                '~' => {
                    if prev_empty {
                        if let Some(prev) = self.builders.back_mut() {
                            prev.md = Markdown::DoubleTilde;
                        } else {
                            unreachable!();
                        }
                        // todo: check if p2 is double tilde and if so, make strikethrough
                    } else {
                        // todo: turn prev tilde into a regular character and push a new markdown
                    }
                }
                _ => self.push_char(c),
            },
            _ => match c {
                '\n' => {
                    while let Some(builder) = self.builders.pop_back() {
                        self.handle_builder(builder);
                    }
                    let new_tag = Tag::from(TagType::NewLine);
                    self.root.add_tag(new_tag);
                }
                '*' => self.push_md(Markdown::Star),
                '_' => self.push_md(Markdown::Underscore),
                '`' => {
                    if !self.is_prev_backslash() {
                        self.push_md(Markdown::Backtick);
                    } else if let Some(prev) = self.builders.back_mut() {
                        prev.in_progress.pop();
                        prev.in_progress.push(c);
                    } else {
                        unreachable!();
                    }
                }
                '~' => self.push_md(Markdown::Tilde),
                ' ' if self.is_start_of_blockquote() => {
                    if let Some(prev) = self.builders.back_mut() {
                        prev.in_progress.pop();
                    } else {
                        unreachable!();
                    }
                    self.push_md(Markdown::BlockQuote);
                }
                ' ' => {
                    if let Some(md) = self.try_get_header_md() {
                        self.builders.pop_back();
                        self.push_md(md);
                    } else {
                        self.push_char(c);
                    }
                }
                _ => self.push_char(c),
            },
        }
    }

    fn bubble_tag(&mut self, tag: Tag) {
        if let Some(builder) = self.builders.back_mut() {
            builder.append_value(TagValue::Tag(tag));
        } else {
            self.root.add_tag(tag);
        }
    }

    fn handle_builder(&mut self, mut builder: TagBuilder) {
        match builder.md {
            Markdown::BlockQuote => {
                let mut tag = Tag::from(TagType::BlockQuote);
                tag.append_values(builder.completed);
                tag.add_text(&builder.in_progress);
                self.bubble_tag(tag);
            }
            Markdown::H1 | Markdown::H2 | Markdown::H3 | Markdown::H4 | Markdown::H5 => {
                let tag_type = match builder.md {
                    Markdown::H1 => TagType::H1,
                    Markdown::H2 => TagType::H2,
                    Markdown::H3 => TagType::H3,
                    Markdown::H4 => TagType::H4,
                    Markdown::H5 => TagType::H5,
                    _ => unreachable!(),
                };
                let mut tag = Tag::from(tag_type);
                tag.append_values(builder.completed);
                tag.add_text(&builder.in_progress);
                self.bubble_tag(tag);
            }
            _ => {
                let mut values = builder.to_values();
                if let Some(prev) = self.builders.back_mut() {
                    prev.append_values(values);
                } else {
                    self.root.append_values(values);
                }
            }
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
        if let Some(builder) = self.builders.back_mut() {
            builder.save_progress();
        }
        self.builders.push_back(md.into());
    }

    fn prev_matches(&self, md: Markdown) -> bool {
        self.builders.back().map(|x| x.md == md).unwrap_or_default()
    }

    fn is_prev_backslash(&self) -> bool {
        self.builders
            .back()
            .as_ref()
            .and_then(|x| x.in_progress.chars().last())
            .map(|c| c == '\\')
            .unwrap_or_default()
    }

    fn is_start_of_blockquote(&self) -> bool {
        match self.builders.back() {
            None => false,
            Some(t) => match t.md {
                Markdown::NewLine | Markdown::Line => {
                    t.completed.is_empty() && t.in_progress == ">"
                }
                _ => false,
            },
        }
    }

    fn get_text_from_code_block(&mut self) -> String {
        let mut p2 = self.builders.pop_back().unwrap();
        p2.completed
            .pop_front()
            .map(|x| match x {
                TagValue::Text(y) => y,
                _ => {
                    debug_assert!(false);
                    String::default()
                }
            })
            .unwrap_or_default()
    }

    fn try_get_header_md(&mut self) -> Option<Markdown> {
        self.builders.back().and_then(|builder| {
            if !builder.completed.is_empty() {
                None
            } else {
                match builder.in_progress.as_str() {
                    "#" => Some(Markdown::H1),
                    "##" => Some(Markdown::H2),
                    "###" => Some(Markdown::H3),
                    "####" => Some(Markdown::H4),
                    "#####" => Some(Markdown::H5),
                    _ => None,
                }
            }
        })
    }
}

pub fn text_to_html2(text: &str) -> Tag {
    let mut parser = Parser::new();
    for c in text.chars() {
        parser.process(c);
    }
    parser.finish()
}

fn get_language(text: &str) -> (String, String) {
    let default = (LANGUAGE_TEXT.to_string(), text.to_string());
    match text.find('\n') {
        Some(x) => {
            let before = text[0..x].to_string();
            let after: String = text.chars().skip(x + 1).collect();
            match before.trim() {
                x if !x.is_empty() => (x.to_string(), after),
                _ => default,
            }
        }
        None => match text.find(' ') {
            Some(x) => {
                let before = text[0..x].to_string();
                let after: String = text.chars().skip(x + 1).collect();
                match before.trim() {
                    x if !x.is_empty() => (x.to_string(), after),
                    _ => default,
                }
            }
            None => default,
        },
    }
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
    fn test_plain_bold2() {
        let test = text_to_html2("abcd__bold__");
        let mut expected = Tag::from(TagType::Paragraph);
        expected.add_text("abcd");
        expected.add_tag_w_text(TagType::Bold, "bold");

        assert_eq!(test, expected);
    }

    #[test]
    fn test_partial_bold1() {
        let test = text_to_html2("abcd**bold");
        let mut expected = Tag::from(TagType::Paragraph);
        expected.add_text("abcd**bold");
        assert_eq!(test, expected);
    }

    #[test]
    fn test_partial_bold2() {
        let test = text_to_html2("abcd__bold");
        let mut expected = Tag::from(TagType::Paragraph);
        expected.add_text("abcd__bold");
        assert_eq!(test, expected);
    }

    #[test]
    fn test_nesting1() {
        let test = text_to_html2("**bold _end bold**");
        let mut expected = Tag::from(TagType::Paragraph);
        expected.add_tag_w_text(TagType::Bold, "bold _end bold");
        assert_eq!(test, expected);
    }

    #[test]
    fn test_nesting2() {
        let test = text_to_html2("**bold _end bold_");
        let mut expected = Tag::from(TagType::Paragraph);
        expected.add_text("**bold ");
        expected.add_tag_w_text(TagType::Italics, "end bold");
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
    fn test_bold2() {
        let test = text_to_html2("__bold__");
        let mut expected = Tag::from(TagType::Paragraph);
        expected.add_tag_w_text(TagType::Bold, "bold");
        assert_eq!(test, expected);
    }

    #[test]
    fn test_triple_star1() {
        let test = text_to_html2("***question**");
        let mut expected = Tag::from(TagType::Paragraph);
        expected.add_text("*");
        expected.add_tag_w_text(TagType::Bold, "question");
        assert_eq!(test, expected);
    }

    #[test]
    fn test_triple_star2() {
        let test = text_to_html2("***question*");
        let mut expected = Tag::from(TagType::Paragraph);
        expected.add_text("***question*");
        assert_eq!(test, expected);
    }

    #[test]
    fn test_triple_star3() {
        let test = text_to_html2("***question***");
        let mut expected = Tag::from(TagType::Paragraph);
        expected.add_text("*");
        expected.add_tag_w_text(TagType::Bold, "question");
        expected.add_text("*");
        assert_eq!(test, expected);
    }

    #[test]
    fn test_triple_underscore1() {
        let test = text_to_html2("___question__");
        let mut expected = Tag::from(TagType::Paragraph);
        expected.add_text("_");
        expected.add_tag_w_text(TagType::Bold, "question");
        assert_eq!(test, expected);
    }

    #[test]
    fn test_triple_underscore2() {
        let test = text_to_html2("___question_");
        let mut expected = Tag::from(TagType::Paragraph);
        expected.add_text("___question_");
        assert_eq!(test, expected);
    }

    #[test]
    fn test_triple_underscore3() {
        let test = text_to_html2("___question___");
        let mut expected = Tag::from(TagType::Paragraph);
        expected.add_text("_");
        expected.add_tag_w_text(TagType::Bold, "question");
        expected.add_text("_");
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

    #[test]
    fn test_italics2() {
        let test = text_to_html2("_italics_");
        let mut expected = Tag::from(TagType::Paragraph);
        expected.add_tag_w_text(TagType::Italics, "italics");
        assert_eq!(test, expected);
    }

    #[test]
    fn test_nested_bold_italics1() {
        let test = text_to_html2("abcd**bold *italics***");
        let mut expected = Tag::from(TagType::Paragraph);
        expected.add_text("abcd");
        let mut bold = Tag::from(TagType::Bold);
        bold.add_text("bold ".into());
        bold.add_tag_w_text(TagType::Italics, "italics");
        expected.add_tag(bold);
        assert_eq!(test, expected);
    }

    #[test]
    fn test_nested_bold_italics2() {
        let test = text_to_html2("abcd__bold *italics*__");
        let mut expected = Tag::from(TagType::Paragraph);
        expected.add_text("abcd");
        let mut bold = Tag::from(TagType::Bold);
        bold.add_text("bold ".into());
        bold.add_tag_w_text(TagType::Italics, "italics");
        expected.add_tag(bold);
        assert_eq!(test, expected);
    }

    #[test]
    fn test_code1() {
        let test = text_to_html2("`hello world`");
        let mut expected = Tag::from(TagType::Paragraph);
        expected.add_tag_w_text(TagType::Code(LANGUAGE_TEXT.into()), "hello world");
        assert_eq!(test, expected);
    }

    #[test]
    fn test_code2() {
        let test = text_to_html2(r"`hello\` world`");
        let mut expected = Tag::from(TagType::Paragraph);
        expected.add_tag_w_text(TagType::Code(LANGUAGE_TEXT.into()), r"hello\` world");
        assert_eq!(test, expected);
    }

    #[test]
    fn test_code3() {
        let test = text_to_html2(r"\``hello world`");
        let mut expected = Tag::from(TagType::Paragraph);
        expected.add_text(r"`");
        expected.add_tag_w_text(TagType::Code(LANGUAGE_TEXT.into()), "hello world");
        assert_eq!(test, expected);
    }

    #[test]
    fn test_code4() {
        let test = text_to_html2(r"```rust hello world```");
        let mut expected = Tag::from(TagType::Paragraph);
        expected.add_tag_w_text(TagType::Code("rust".into()), "hello world");
        assert_eq!(test, expected);
    }

    #[test]
    fn test_code5() {
        let test = text_to_html2(r"```rust \`hello world```");
        let mut expected = Tag::from(TagType::Paragraph);
        expected.add_tag_w_text(TagType::Code("rust".into()), r"\`hello world");
        assert_eq!(test, expected);
    }

    #[test]
    fn test_code6() {
        let test = text_to_html2("```rust\n hello\n world```");
        let mut expected = Tag::from(TagType::Paragraph);
        expected.add_tag_w_text(TagType::Code("rust".into()), " hello\n world");
        assert_eq!(test, expected);
    }

    #[test]
    fn test_blockquote1() {
        let test = text_to_html2("> some blockquote");
        let mut expected = Tag::from(TagType::Paragraph);
        expected.add_tag_w_text(TagType::BlockQuote, "some blockquote".into());
        assert_eq!(test, expected);
    }

    #[test]
    fn test_blockquote2() {
        let test = text_to_html2("> some blockquote __bold__");
        let mut expected = Tag::from(TagType::Paragraph);
        let mut bq = Tag::from(TagType::BlockQuote);
        bq.add_text("some blockquote ");
        bq.add_tag_w_text(TagType::Bold, "bold");
        expected.add_tag(bq);
        assert_eq!(test, expected);
    }

    #[test]
    fn test_blockquote3() {
        let test = text_to_html2("abc\n> some blockquote __bold__\ndef");
        let mut expected = Tag::from(TagType::Paragraph);
        expected.add_text("abc");
        expected.add_tag(TagType::NewLine.into());
        let mut bq = Tag::from(TagType::BlockQuote);
        bq.add_text("some blockquote ");
        bq.add_tag_w_text(TagType::Bold, "bold");
        expected.add_tag(bq);
        expected.add_tag(TagType::NewLine.into());
        expected.add_text("def");
        assert_eq!(test, expected);
    }

    #[test]
    fn test_h1() {
        let text = text_to_html2("# heading");
        let mut expected = Tag::from(TagType::Paragraph);
        expected.add_tag_w_text(TagType::H1, "heading");
        assert_eq!(text, expected);
    }

    #[test]
    fn test_h1_1() {
        let text = text_to_html2("# heading **bold**");
        let mut expected = Tag::from(TagType::Paragraph);
        let mut h1 = Tag::from(TagType::H1);
        h1.add_text("heading ");
        h1.add_tag_w_text(TagType::Bold, "bold");
        expected.add_tag(h1);
        assert_eq!(text, expected);
    }

    #[test]
    fn test_h1_2() {
        let text = text_to_html2("# heading **bold**\n# heading **bold**");
        let mut expected = Tag::from(TagType::Paragraph);
        let mut h1 = Tag::from(TagType::H1);
        h1.add_text("heading ");
        h1.add_tag_w_text(TagType::Bold, "bold");
        expected.add_tag(h1.clone());
        expected.add_tag(TagType::NewLine.into());
        expected.add_tag(h1);
        assert_eq!(text, expected);
    }

    #[test]
    fn test_h1_fail() {
        let text = text_to_html2("#heading");
        let mut expected = Tag::from(TagType::Paragraph);
        expected.add_text("#heading");
        assert_eq!(text, expected);
    }

    #[test]
    fn test_h2() {
        let text = text_to_html2("## heading");
        let mut expected = Tag::from(TagType::Paragraph);
        expected.add_tag_w_text(TagType::H2, "heading");
        assert_eq!(text, expected);
    }

    #[test]
    fn test_h3() {
        let text = text_to_html2("### heading");
        let mut expected = Tag::from(TagType::Paragraph);
        expected.add_tag_w_text(TagType::H3, "heading");
        assert_eq!(text, expected);
    }

    #[test]
    fn test_h4() {
        let text = text_to_html2("#### heading");
        let mut expected = Tag::from(TagType::Paragraph);
        expected.add_tag_w_text(TagType::H4, "heading");
        assert_eq!(text, expected);
    }

    #[test]
    fn test_h5() {
        let text = text_to_html2("##### heading");
        let mut expected = Tag::from(TagType::Paragraph);
        expected.add_tag_w_text(TagType::H5, "heading");
        assert_eq!(text, expected);
    }

    #[test]
    fn test_h6() {
        let text = text_to_html2("###### heading");
        let mut expected = Tag::from(TagType::Paragraph);
        expected.add_text("###### heading");
        assert_eq!(text, expected);
    }
}
