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

    let prev_matches = |stack: &VecDeque<StackEntry>, md: Markdown| {
        stack.back().map(|x| x.md == md).unwrap_or_default()
    };

    let push_char = |stack: &mut VecDeque<StackEntry>, c: char| {
        if let Some(entry) = stack.back_mut() {
            entry.text.push(c);
        } else {
            // should never happen
            stack.push_back(StackEntry::new(Markdown::Line, String::from(c)));
        }
    };

    let clear_stack = |stack: &mut VecDeque<StackEntry>, ret_stack: &mut VecDeque<Element>| {
        for stack_entry in stack.drain(..) {
            match stack_entry.md {
                Markdown::Line => ret_stack.push_back(Element::new(Tag::Span, stack_entry.text)),
                _ => todo!(),
            }
        }
    };

    let mut ret_stack: VecDeque<Element> = VecDeque::new();
    for char in text.chars() {
        let (prev_md, prev_text) = stack
            .back()
            .map(|x| (x.md.clone(), x.text.clone()))
            .expect("stack should not be empty");
        let prev_empty = prev_text.is_empty();

        match prev_md {
            Markdown::Star => match char {
                '*' => {
                    stack.pop_back();
                    if prev_empty {
                        if prev_matches(&stack, Markdown::DoubleStar) {
                            let p2 = stack.pop_back().unwrap();
                            clear_stack(&mut stack, &mut ret_stack);
                            ret_stack.push_back(Element::new(Tag::Bold, p2.text));
                        } else {
                             stack.push_back(Markdown::DoubleStar.into());
                        }
                    } else {
                        clear_stack(&mut stack, &mut ret_stack);
                        ret_stack.push_back(Element::new(Tag::Italics, prev_text));
                    }
                }
                _ => stack.push_back(Markdown::Star.into()),
            },
            _ => match char {
                '*' => stack.push_back(Markdown::Star.into()),
                _ => push_char(&mut stack, char),
            },
        }
    }
    clear_stack(&mut stack, &mut ret_stack);
    ret_stack
}

#[cfg(test)]
mod test {
    use super::*;

    fn element_to_vecdeque(tag: Tag, value: &str) -> VecDeque<Element> {
        let mut m = VecDeque::new();
        m.push_back(Element::new(tag, value.into()));
        m
    }

    fn vec_to_vecdeque(mut v: Vec<(Tag, &str)>) -> VecDeque<Element> {
        VecDeque::from_iter(
            v.drain(..)
                .map(|(tag, value)| Element::new(tag, value.into())),
        )
    }

    #[test]
    fn test_plain_test() {
        let test = text_to_html2("abcd");
        let expected = element_to_vecdeque(Tag::Span, "abcd");
        assert_eq!(test, expected);
    }

    #[test]
    fn test_plain_bold1() {
        let test = text_to_html2("abcd**bold**");
        let expected = vec_to_vecdeque(vec![(Tag::Span, "abcd"), (Tag::Bold, "bold")]);
        assert_eq!(test, expected);
    }
}
