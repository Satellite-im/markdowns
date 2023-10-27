mod tag;

use std::collections::VecDeque;

use pulldown_cmark::{CodeBlockKind, Event};

use crate::tag::{Tag, TagType, TagValue};

const LANGUAGE_TEXT: &str = "text";

pub fn text_to_html(text: &str) -> Tag {
    let mut root = Tag::from(TagType::Paragraph);

    let mut options = pulldown_cmark::Options::empty();
    options.insert(pulldown_cmark::Options::ENABLE_STRIKETHROUGH);

    let mut values: VecDeque<TagValue> = VecDeque::new();
    let mut tag_stack: VecDeque<Tag> = VecDeque::new();
    let mut in_code_block = false;

    let parser = pulldown_cmark::Parser::new_ext(text, options);
    for event in parser {
        match event {
            Event::Start(pulldown_tag) => match pulldown_tag {
                // A paragraph of text and other inline elements.
                pulldown_cmark::Tag::Paragraph => {}

                // A heading. The first field indicates the level of the heading,
                // the second the fragment identifier, and the third the classes.
                pulldown_cmark::Tag::Heading(heading_level, _fragment_identifier, _classes) => {
                    let tag_type = match heading_level {
                        pulldown_cmark::HeadingLevel::H1 => TagType::H1,
                        pulldown_cmark::HeadingLevel::H2 => TagType::H2,
                        pulldown_cmark::HeadingLevel::H3 => TagType::H3,
                        pulldown_cmark::HeadingLevel::H4 => TagType::H4,
                        pulldown_cmark::HeadingLevel::H5 => TagType::H5,
                        pulldown_cmark::HeadingLevel::H6 => TagType::H6,
                    };
                    tag_stack.push_back(Tag::from(tag_type));
                }

                pulldown_cmark::Tag::BlockQuote => {}
                // A code block.
                pulldown_cmark::Tag::CodeBlock(code_block_kind) => {
                    in_code_block = true;
                    let language = match code_block_kind {
                        CodeBlockKind::Indented => LANGUAGE_TEXT.into(),
                        CodeBlockKind::Fenced(lang) => {
                            if lang.is_empty() {
                                LANGUAGE_TEXT.into()
                            } else {
                                lang.to_string()
                            }
                        }
                    };
                    tag_stack.push_back(Tag::from(TagType::Code(language)));
                }

                // A list. If the list is ordered the field indicates the number of the first item.
                // Contains only list items.
                pulldown_cmark::Tag::List(_start_number) => {} // TODO: add delim and tight for ast (not needed for html)
                // A list item.
                pulldown_cmark::Tag::Item => {}
                // A footnote definition. The value contained is the footnote's label by which it can
                // be referred to.
                pulldown_cmark::Tag::FootnoteDefinition(_label) => {}

                // A table. Contains a vector describing the text-alignment for each of its columns.
                pulldown_cmark::Tag::Table(_text_alignment) => {}
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
                pulldown_cmark::Tag::Link(_link_type, _dest, _link_title) => {}

                // An image. The first field is the link type, the second the destination URL and the third is a title.
                pulldown_cmark::Tag::Image(_link_type, _dest, _image_title) => {}
            },
            Event::End(pulldown_cmark::Tag::CodeBlock(_)) => {
                in_code_block = false;
                let back = tag_stack.back_mut().unwrap();
                if let Some(TagValue::Text(val)) = back.values.back_mut() {
                    if val.ends_with("```") {
                        val.pop();
                        val.pop();
                        val.pop();
                    } else if val.ends_with('`') {
                        val.pop();
                    }
                }
            }
            Event::End(_) => {
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
            Event::FootnoteReference(_text) => {}
            Event::HardBreak | Event::SoftBreak => {
                if in_code_block {
                    if let Some(tag) = tag_stack.back_mut() {
                        tag.add_text("\n");
                    } else {
                        unreachable!();
                    }
                } else {
                    let t: Tag = Tag::from(TagType::NewLine);
                    if let Some(tag) = tag_stack.back_mut() {
                        tag.add_tag(t);
                    } else {
                        values.push_back(TagValue::Tag(t));
                    }
                }
            }
            Event::Rule => {}
            Event::TaskListMarker(_is_checked) => {}
        }
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

    root
}

#[cfg(test)]
mod test {
    use crate::tag::TagType;

    use super::*;

    #[test]
    fn test_edge1() {
        let test = text_to_html("*abc _def*");
        let mut expected = Tag::from(TagType::Paragraph);
        expected.add_tag_w_text(TagType::Italics, "abc _def");
        assert_eq!(test, expected);
    }

    #[test]
    fn test_plain_test() {
        let mut expected = Tag::from(TagType::Paragraph);
        expected.add_text("abcd");

        let test = text_to_html("abcd");
        assert_eq!(test, expected);
    }

    #[test]
    fn test_plain_bold1() {
        let test = text_to_html("abcd**bold**");
        let mut expected = Tag::from(TagType::Paragraph);
        expected.add_text("abcd");
        expected.add_tag_w_text(TagType::Bold, "bold");

        assert_eq!(test, expected);
    }

    #[test]
    fn test_plain_bold2() {
        let test = text_to_html("abcd__bold__");
        let mut expected = Tag::from(TagType::Paragraph);
        expected.add_text("abcd__bold__");
        assert_eq!(test, expected);
    }

    #[test]
    fn test_partial_bold1() {
        let test = text_to_html("abcd**bold");
        let mut expected = Tag::from(TagType::Paragraph);
        expected.add_text("abcd**bold");
        assert_eq!(test, expected);
    }

    #[test]
    fn test_partial_bold2() {
        let test = text_to_html("abcd__bold");
        let mut expected = Tag::from(TagType::Paragraph);
        expected.add_text("abcd__bold");
        assert_eq!(test, expected);
    }

    #[test]
    fn test_nesting1() {
        let test = text_to_html("**bold _end bold**");
        let mut expected = Tag::from(TagType::Paragraph);
        expected.add_tag_w_text(TagType::Bold, "bold _end bold");
        assert_eq!(test, expected);
    }

    #[test]
    fn test_nesting2() {
        let test = text_to_html("**bold _end bold_");
        let mut expected = Tag::from(TagType::Paragraph);
        expected.add_text("**bold ");
        expected.add_tag_w_text(TagType::Italics, "end bold");
        assert_eq!(test, expected);
    }

    #[test]
    fn test_bold1() {
        let test = text_to_html("**bold**");
        let mut expected = Tag::from(TagType::Paragraph);
        expected.add_tag_w_text(TagType::Bold, "bold");
        assert_eq!(test, expected);
    }

    #[test]
    fn test_bold2() {
        let test = text_to_html("__bold__");
        let mut expected = Tag::from(TagType::Paragraph);
        expected.add_tag_w_text(TagType::Bold, "bold");
        assert_eq!(test, expected);
    }

    #[test]
    fn test_bold3() {
        let test = text_to_html("** bold**");
        let mut expected = Tag::from(TagType::Paragraph);
        expected.add_text("** bold**");
        assert_eq!(test, expected);
    }

    #[test]
    fn test_bold4() {
        let test = text_to_html("**bold bold**");
        let mut expected = Tag::from(TagType::Paragraph);
        expected.add_tag_w_text(TagType::Bold, "bold bold");
        assert_eq!(test, expected);
    }

    #[test]
    fn test_triple_star1() {
        let test = text_to_html("***question**");
        let mut expected = Tag::from(TagType::Paragraph);
        expected.add_text("*");
        expected.add_tag_w_text(TagType::Bold, "question");
        assert_eq!(test, expected);
    }

    #[test]
    fn test_triple_star2() {
        let test = text_to_html("***question*");
        let mut expected = Tag::from(TagType::Paragraph);
        expected.add_text("**");
        expected.add_tag_w_text(TagType::Italics, "question");
        assert_eq!(test, expected);
    }

    #[test]
    fn test_triple_star3() {
        let test = text_to_html("***question***");
        let mut expected = Tag::from(TagType::Paragraph);
        let mut italics = Tag::from(TagType::Italics);
        italics.add_tag_w_text(TagType::Bold, "question");
        expected.add_tag(italics);
        assert_eq!(test, expected);
    }

    #[test]
    fn test_triple_underscore1() {
        let test = text_to_html("___question__");
        let mut expected = Tag::from(TagType::Paragraph);
        expected.add_text("_");
        expected.add_tag_w_text(TagType::Bold, "question");
        assert_eq!(test, expected);
    }

    #[test]
    fn test_triple_underscore2() {
        let test = text_to_html("___question_");
        let mut expected = Tag::from(TagType::Paragraph);
        expected.add_text("__");
        expected.add_tag_w_text(TagType::Italics, "question");
        assert_eq!(test, expected);
    }

    #[test]
    fn test_triple_underscore3() {
        let test = text_to_html("___question___");
        let mut expected = Tag::from(TagType::Paragraph);
        let mut italics = Tag::from(TagType::Italics);
        italics.add_tag_w_text(TagType::Bold, "question");
        expected.add_tag(italics);
        assert_eq!(test, expected);
    }

    #[test]
    fn test_plain_italics() {
        let test = text_to_html("abcd*italics*");
        let mut expected = Tag::from(TagType::Paragraph);
        expected.add_text("abcd");
        expected.add_tag_w_text(TagType::Italics, "italics");
        assert_eq!(test, expected);
    }

    #[test]
    fn test_italics1() {
        let test = text_to_html("*italics*");
        let mut expected = Tag::from(TagType::Paragraph);
        expected.add_tag_w_text(TagType::Italics, "italics");
        assert_eq!(test, expected);
    }

    #[test]
    fn test_italics2() {
        let test = text_to_html("_italics_");
        let mut expected = Tag::from(TagType::Paragraph);
        expected.add_tag_w_text(TagType::Italics, "italics");
        assert_eq!(test, expected);
    }

    #[test]
    fn test_nested_bold_italics1() {
        let test = text_to_html("abcd**bold *italics***");
        let mut expected = Tag::from(TagType::Paragraph);
        expected.add_text("abcd");
        let mut bold = Tag::from(TagType::Bold);
        bold.add_text("bold ");
        bold.add_tag_w_text(TagType::Italics, "italics");
        expected.add_tag(bold);
        assert_eq!(test, expected);
    }

    #[test]
    fn test_nested_bold_italics2() {
        let test = text_to_html("abcd__bold *italics*__");
        let mut expected = Tag::from(TagType::Paragraph);
        expected.add_text("abcd__bold ");
        expected.add_tag_w_text(TagType::Italics, "italics");
        expected.add_text("__");
        assert_eq!(test, expected);
    }

    #[test]
    fn test_code1() {
        let test = text_to_html("`hello world`");
        let mut expected = Tag::from(TagType::Paragraph);
        expected.add_tag_w_text(TagType::Code(LANGUAGE_TEXT.into()), "hello world");
        assert_eq!(test, expected);
    }

    #[test]
    fn test_code2() {
        let test = text_to_html(r"`hello\` world`");
        let mut expected = Tag::from(TagType::Paragraph);
        expected.add_tag_w_text(TagType::Code(LANGUAGE_TEXT.into()), r"hello\` world");
        assert_eq!(test, expected);
    }

    #[test]
    fn test_code3() {
        let test = text_to_html(r"\``hello world`");
        let mut expected = Tag::from(TagType::Paragraph);
        expected.add_text(r"`");
        expected.add_tag_w_text(TagType::Code(LANGUAGE_TEXT.into()), "hello world");
        assert_eq!(test, expected);
    }

    #[test]
    fn test_code4() {
        let test = text_to_html(r"```rust hello world```");
        let mut expected = Tag::from(TagType::Paragraph);
        expected.add_tag_w_text(TagType::Code("text".into()), "rust hello world");
        assert_eq!(test, expected);
    }

    #[test]
    fn test_code5() {
        let test = text_to_html(r"```rust \`hello world```");
        let mut expected = Tag::from(TagType::Paragraph);
        expected.add_tag_w_text(TagType::Code("text".into()), r"rust \`hello world");
        assert_eq!(test, expected);
    }

    #[test]
    fn test_code6() {
        let test = text_to_html("```rust\n hello\n world```");
        let mut expected = Tag::from(TagType::Paragraph);
        expected.add_tag_w_text(TagType::Code("rust".into()), " hello\n world");
        assert_eq!(test, expected);
    }

    #[test]
    fn test_blockquote1() {
        let test = text_to_html("> some blockquote");
        let mut expected = Tag::from(TagType::Paragraph);
        expected.add_tag_w_text(TagType::BlockQuote, "some blockquote");
        assert_eq!(test, expected);
    }

    #[test]
    fn test_blockquote2() {
        let test = text_to_html("> some blockquote __bold__");
        let mut expected = Tag::from(TagType::Paragraph);
        let mut bq = Tag::from(TagType::BlockQuote);
        bq.add_text("some blockquote ");
        bq.add_tag_w_text(TagType::Bold, "bold");
        expected.add_tag(bq);
        assert_eq!(test, expected);
    }

    #[test]
    fn test_blockquote3() {
        let test = text_to_html("abc\n> some blockquote __bold__\ndef");
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
        let text = text_to_html("# heading");
        let mut expected = Tag::from(TagType::Paragraph);
        expected.add_tag_w_text(TagType::H1, "heading");
        assert_eq!(text, expected);
    }

    #[test]
    fn test_h1_1() {
        let text = text_to_html("# heading **bold**");
        let mut expected = Tag::from(TagType::Paragraph);
        let mut h1 = Tag::from(TagType::H1);
        h1.add_text("heading ");
        h1.add_tag_w_text(TagType::Bold, "bold");
        expected.add_tag(h1);
        assert_eq!(text, expected);
    }

    #[test]
    fn test_h1_2() {
        let text = text_to_html("# heading **bold**\n# heading **bold**");
        let mut expected = Tag::from(TagType::Paragraph);
        let mut h1 = Tag::from(TagType::H1);
        h1.add_text("heading ");
        h1.add_tag_w_text(TagType::Bold, "bold");
        expected.add_tag(h1.clone());
        expected.add_tag(h1);
        assert_eq!(text, expected);
    }

    #[test]
    fn test_h1_fail() {
        let text = text_to_html("#heading");
        let mut expected = Tag::from(TagType::Paragraph);
        expected.add_text("#heading");
        assert_eq!(text, expected);
    }

    #[test]
    fn test_h2() {
        let text = text_to_html("## heading");
        let mut expected = Tag::from(TagType::Paragraph);
        expected.add_tag_w_text(TagType::H2, "heading");
        assert_eq!(text, expected);
    }

    #[test]
    fn test_h3() {
        let text = text_to_html("### heading");
        let mut expected = Tag::from(TagType::Paragraph);
        expected.add_tag_w_text(TagType::H3, "heading");
        assert_eq!(text, expected);
    }

    #[test]
    fn test_h4() {
        let text = text_to_html("#### heading");
        let mut expected = Tag::from(TagType::Paragraph);
        expected.add_tag_w_text(TagType::H4, "heading");
        assert_eq!(text, expected);
    }

    #[test]
    fn test_h5() {
        let text = text_to_html("##### heading");
        let mut expected = Tag::from(TagType::Paragraph);
        expected.add_tag_w_text(TagType::H5, "heading");
        assert_eq!(text, expected);
    }

    #[test]
    fn test_h6() {
        let text = text_to_html("###### heading");
        let mut expected = Tag::from(TagType::Paragraph);
        expected.add_tag_w_text(TagType::H6, "heading");
        assert_eq!(text, expected);
    }
}
