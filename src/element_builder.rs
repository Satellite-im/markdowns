use std::collections::VecDeque;

use crate::{Markdown, StackEntry};

#[derive(Clone, Eq, PartialEq, Debug)]
pub enum Tag {
    Span,
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

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Element {
    tag: Tag,
    value: String,
}

impl Element {
    fn new(tag: Tag, value: String) -> Self {
        Self { tag, value }
    }
}

impl From<Tag> for Element {
    fn from(tag: Tag) -> Self {
        Self {
            tag,
            value: String::new(),
        }
    }
}

pub fn text_to_html2(text: &str) -> VecDeque<Element> {
    let mut stack: VecDeque<StackEntry> = VecDeque::new();
    stack.push_back(Markdown::Line.into());

    let mut ret_stack: VecDeque<Element> = VecDeque::new();
    for char in text.chars() {
        let (prev_md, prev_empty) = stack
            .back()
            .map(|x| (x.md.clone(), x.text.is_empty()))
            .expect("stack should not be empty");

        match char {
            c => {
                if let Some(entry) = stack.back_mut() {
                    entry.text.push(c);
                } else {
                    // should never happen
                    stack.push_back(StackEntry::new(Markdown::Line, String::from(c)));
                }
            }
        }
    }

    for stack_entry in stack.drain(..) {
        match stack_entry.md {
            Markdown::Line => ret_stack.push_back(Element::new(Tag::Span, stack_entry.text)),
            _ => todo!(),
        }
    }
    ret_stack
}

#[cfg(test)]
mod test {
    use super::*;

    fn element_to_vecdeque(tag: Tag, value: String) -> VecDeque<Element> {
        let mut m = VecDeque::new();
        m.push_back(Element::new(tag, value));
        m
    }

    #[test]
    fn test_plain_test() {
        let test = text_to_html2("abcd");
        let expected = element_to_vecdeque(Tag::Span, "abcd".into());
        assert_eq!(test, expected);
    }
}
