use std::collections::VecDeque;

use pulldown_cmark::Event;

use crate::{
    md::Markdown,
    tag::{Tag, TagType, TagValue},
    tag_builder::TagBuilder,
};

const LANGUAGE_TEXT: &str = "text";

pub fn text_to_html2(text: &str) -> Tag {
    let mut root = Tag::from(TagType::Paragraph);
    let mut is_first = true;
    let mut options = pulldown_cmark::Options::empty();
    options.insert(pulldown_cmark::Options::ENABLE_STRIKETHROUGH);
    for line in text.lines() {
        let mut values: VecDeque<TagValue> = VecDeque::new();
        let mut tag_stack: VecDeque<Tag> = VecDeque::new();
        let mut in_code_block = false;

        let parser = pulldown_cmark::Parser::new_ext(line, options);

        let mut it = parser.into_iter();
        while let Some(event) = it.next() {
            match event {
                Event::Start(pulldown_tag) => match pulldown_tag {
                    // A paragraph of text and other inline elements.
                    pulldown_cmark::Tag::Paragraph => {}

                    // A heading. The first field indicates the level of the heading,
                    // the second the fragment identifier, and the third the classes.
                    pulldown_cmark::Tag::Heading(heading_level, fragment_identifier, classes) => {}

                    pulldown_cmark::Tag::BlockQuote => {}
                    // A code block.
                    pulldown_cmark::Tag::CodeBlock(code_block_kind) => {
                        in_code_block = true;
                    }

                    // A list. If the list is ordered the field indicates the number of the first item.
                    // Contains only list items.
                    pulldown_cmark::Tag::List(start_number) => {} // TODO: add delim and tight for ast (not needed for html)
                    // A list item.
                    pulldown_cmark::Tag::Item => {}
                    // A footnote definition. The value contained is the footnote's label by which it can
                    // be referred to.
                    pulldown_cmark::Tag::FootnoteDefinition(label) => {}

                    // A table. Contains a vector describing the text-alignment for each of its columns.
                    pulldown_cmark::Tag::Table(text_alignment) => {}
                    // A table header. Contains only `TableCell`s. Note that the table body starts immediately
                    // after the closure of the `TableHead` tag. There is no `TableBody` tag.
                    pulldown_cmark::Tag::TableHead => {}
                    // A table row. Is used both for header rows as body rows. Contains only `TableCell`s.
                    pulldown_cmark::Tag::TableRow => {}
                    pulldown_cmark::Tag::TableCell => {}

                    // span-level tags
                    pulldown_cmark::Tag::Emphasis => tag_stack.push_back(TagType::Italics.into()),
                    pulldown_cmark::Tag::Strong => tag_stack.push_back(TagType::Bold.into()),
                    pulldown_cmark::Tag::Strikethrough => {
                        tag_stack.push_back(TagType::Strikethrough.into())
                    }

                    // A link. The first field is the link type, the second the destination URL and the third is a title.
                    pulldown_cmark::Tag::Link(link_type, dest, link_title) => {}

                    // An image. The first field is the link type, the second the destination URL and the third is a title.
                    pulldown_cmark::Tag::Image(link_type, dest, image_title) => {}
                },
                Event::End(tag_type) => {
                    if matches!(tag_type, pulldown_cmark::Tag::CodeBlock(_)) {
                        in_code_block = false;
                    }
                    if let Some(tag) = tag_stack.pop_back() {
                        if let Some(prev) = tag_stack.back_mut() {
                            prev.add_tag(tag);
                        } else {
                            values.push_back(TagValue::Tag(tag));
                        }
                    }
                }
                Event::Text(text) => {
                    if let Some(tag) = tag_stack.back_mut() {
                        tag.add_text(&text);
                    } else {
                        values.push_back(TagValue::Text(text.to_string()));
                    }
                }
                Event::Code(text) => {
                    if let Some(tag) = tag_stack.back_mut() {
                        tag.add_tag_w_text(TagType::Code(LANGUAGE_TEXT.into()), &text);
                    } else {
                        let mut t = Tag::from(TagType::Code(LANGUAGE_TEXT.into()));
                        t.add_text(&text);
                        values.push_back(TagValue::Tag(t));
                    }
                }
                Event::Html(text) => {
                    if let Some(tag) = tag_stack.back_mut() {
                        tag.add_text(&text);
                    } else {
                        values.push_back(TagValue::Text(text.to_string()));
                    }
                }
                Event::FootnoteReference(text) => {}
                Event::HardBreak | Event::SoftBreak => {
                    if in_code_block {
                        if let Some(tag) = tag_stack.back_mut() {
                            tag.add_text("\n");
                        } else {
                            unreachable!();
                        }
                    } else {
                        let t = Tag::from(TagType::NewLine);
                        if let Some(tag) = tag_stack.back_mut() {
                            tag.add_tag(t);
                        } else {
                            values.push_back(TagValue::Tag(t));
                        }
                    }
                }
                Event::Rule => {}
                Event::TaskListMarker(is_checked) => {}
            }
        }

        // before adding the new tag, check if a newline should be added
        if is_first {
            is_first = false;
        } else {
            // add a newline for root, unless there's 2 successive block quotes
            let t = Tag::from(TagType::NewLine);
            root.add_tag(t);
        }

        // combine tag stack
        while let Some(tag) = tag_stack.pop_back() {
            if let Some(prev) = tag_stack.back_mut() {
                prev.add_tag(tag);
            } else {
                values.push_back(TagValue::Tag(tag));
            }
        }

        root.append_values(values);
    }

    root
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
    fn test_edge1() {
        let test = text_to_html2("*abc _def*");
        let mut expected = Tag::from(TagType::Paragraph);
        expected.add_tag_w_text(TagType::Italics, "abc _def");
        assert_eq!(test, expected);
    }

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
    fn test_bold3() {
        let test = text_to_html2("** bold**");
        let mut expected = Tag::from(TagType::Paragraph);
        expected.add_text("** bold**");
        assert_eq!(test, expected);
    }

    #[test]
    fn test_bold4() {
        let test = text_to_html2("**bold bold**");
        let mut expected = Tag::from(TagType::Paragraph);
        expected.add_tag_w_text(TagType::Bold, "bold bold");
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
