use std::collections::VecDeque;

/// markdowns
/// text to html, with support for the following
/// - italics
///     - *x*
///     - _x_
/// - bold
///     - **x**
///     - __x__
/// - strikethrough
///     - ~~x~~
/// - code
///     - `int a = 0;`
///     - ```int a = 0;```
/// - multiline code
///     ```
///     int a = 0;
///     int b = 0;
///     ```
/// - multiline code with a language
///     ```rust
///     let a = 0;
///     let b = 0;
///     ```

pub fn text_to_html(text: &str) -> String {
    let mut stack: VecDeque<StackEntry> = VecDeque::new();
    stack.push_back(Markdown::None.into());

    // empty tags or just whitespace are not allowed
    let convert_html = |stack: &mut VecDeque<StackEntry>, tag: &str, md: Markdown| {
        if let Some(entry) = stack.pop_back() {
            if entry.text.trim().is_empty() {
                stack.push_back(StackEntry::new(
                    Markdown::None,
                    format!("{}{}", md.to_string(), entry.text),
                ));
                stack.push_back(md.into());
            } else if let Some(entry2) = stack.back_mut() {
                entry2.text += &format!("<{tag}>{}</{tag}>", entry.text);
            } else {
                unreachable!();
            }
        } else {
            unreachable!();
        }
    };

    let prev_matches = |stack: &VecDeque<StackEntry>, md: Markdown| {
        stack.back().map(|x| x.md == md).unwrap_or_default()
    };

    // fold back of stack into previous entry
    let fold_prev = |stack: &mut VecDeque<StackEntry>| {
        let builder = stack.pop_back().map(|x| x.to_string()).unwrap_or_default();
        if let Some(entry) = stack.back_mut() {
            entry.text += &builder;
        } else {
            unreachable!()
        }
    };

    for char in text.chars() {
        let (prev_md, prev_empty) = stack
            .back()
            .map(|x| (x.md.clone(), x.text.is_empty()))
            .expect("stack should not be nonempty");
        match char {
            '*' => match prev_md {
                Markdown::Star => {
                    if prev_empty {
                        stack.pop_back();
                        if prev_matches(&stack, Markdown::DoubleStar) {
                            // handle double star
                            convert_html(&mut stack, "strong", Markdown::DoubleStar);
                        } else {
                            stack.push_back(Markdown::DoubleStar.into());
                        }
                    } else {
                        // handle star
                        convert_html(&mut stack, "em", Markdown::Star);
                    }
                }
                _ => stack.push_back(Markdown::Star.into()),
            },
            '_' => match prev_md {
                Markdown::Underscore => {
                    if prev_empty {
                        stack.pop_back();
                        if prev_matches(&stack, Markdown::DoubleUnderscore) {
                            // handle double underscore
                            convert_html(&mut stack, "strong", Markdown::DoubleUnderscore);
                        } else {
                            stack.push_back(Markdown::DoubleUnderscore.into());
                        }
                    } else {
                        // handle underscore
                        convert_html(&mut stack, "em", Markdown::Underscore);
                    }
                }
                _ => stack.push_back(Markdown::Underscore.into()),
            },
            '`' => match prev_md {
                Markdown::Backtick => {
                    if prev_empty {
                        stack.pop_back();
                        stack.push_back(Markdown::DoubleBacktick.into());
                    } else {
                        // handle backtick
                        convert_html(&mut stack, "code", Markdown::Backtick);
                    }
                }
                Markdown::DoubleBacktick => {
                    if prev_empty {
                        stack.pop_back();
                        if prev_matches(&stack, Markdown::TripleBacktick) {
                            // handle triple backtick
                            convert_html(&mut stack, "code", Markdown::TripleBacktick);
                        } else if matches!(
                            stack.back().map(|x| x.md.clone()),
                            Some(Markdown::Language { .. })
                        ) {
                            let entry = stack.pop_back().unwrap();
                            if let Markdown::Language { lang, lang_done } = entry.md {
                                if !lang_done {
                                    stack.push_back(StackEntry::new(
                                        Markdown::None,
                                        format!("<code>{}</code>", lang),
                                    ));
                                } else {
                                    stack.push_back(StackEntry::new(
                                        Markdown::None,
                                        format!(
                                            "<code language=\"{}\">{}</code>",
                                            lang, entry.text
                                        ),
                                    ));
                                }
                            } else {
                                unreachable!();
                            }
                        } else {
                            stack.push_back(
                                Markdown::Language {
                                    lang: String::new(),
                                    lang_done: false,
                                }
                                .into(),
                            );
                        }
                    } else {
                        // the pattern looks like this: ``[\w+]`. Make a code segment.
                        let text = stack.pop_back().map(|x| x.text).unwrap_or_default();
                        let html = format!("`<code>{text}</code>");
                        if let Some(entry) = stack.back_mut() {
                            entry.text += &html;
                        } else {
                            unreachable!()
                        }
                    }
                }
                _ => stack.push_back(Markdown::Backtick.into()),
            },
            '~' => match prev_md {
                Markdown::Tilde => {
                    if prev_empty {
                        // now have a double tilde. but is the previous one a double tilde?
                        stack.pop_back();
                        if prev_matches(&stack, Markdown::DoubleTilde) {
                            convert_html(&mut stack, "s", Markdown::DoubleTilde);
                        } else {
                            stack.push_back(Markdown::DoubleTilde.into());
                        }
                    } else {
                        // single tilde means nothing. merge last 2 entries on stack and push new entry afterwards.
                        fold_prev(&mut stack);
                        stack.push_back(Markdown::Tilde.into());
                    }
                }
                _ => {
                    stack.push_back(Markdown::Tilde.into());
                }
            },
            '\n' => match prev_md {
                Markdown::Language { lang, lang_done } if !lang_done => {
                    stack.pop_back();
                    if lang.trim().is_empty() {
                        stack.push_back(Markdown::TripleBacktick.into());
                    } else {
                        stack.push_back(StackEntry::new(
                            Markdown::Language {
                                lang: lang.trim().to_string(),
                                lang_done: true,
                            },
                            String::new(),
                        ));
                    }
                }
                Markdown::TripleBacktick => {
                    if let Some(entry) = stack.back_mut() {
                        entry.text += "\n";
                    }
                }
                _ => {
                    // markdown doesn't wrap around lines. So collapse the stack
                    let mut builder = String::new();
                    while let Some(entry) = stack.pop_back() {
                        builder = entry.to_string() + &builder;
                    }
                    stack.push_back(StackEntry::new(Markdown::None, builder));
                }
            },
            c => {
                if stack
                    .back()
                    .map(|x| matches!(x.md, Markdown::Language { lang_done, .. } if !lang_done))
                    .unwrap_or_default()
                {
                    let prev = stack.pop_back().map(|x| x.md);
                    if let Some(Markdown::Language {
                        mut lang,
                        lang_done: _,
                    }) = prev
                    {
                        if c.is_alphabetic() {
                            lang.push(c);
                            stack.push_back(
                                Markdown::Language {
                                    lang,
                                    lang_done: false,
                                }
                                .into(),
                            );
                        } else {
                            if lang.trim().is_empty() {
                                stack.push_back(StackEntry::new(
                                    Markdown::TripleBacktick,
                                    String::from(c),
                                ));
                            } else {
                                stack.push_back(StackEntry::new(
                                    Markdown::Language {
                                        lang: lang.trim().to_string(),
                                        lang_done: true,
                                    },
                                    String::from(c),
                                ));
                            }
                        }
                    } else {
                        unreachable!();
                    }
                } else if let Some(entry) = stack.back_mut() {
                    entry.text.push(c);
                } else {
                    unreachable!();
                }
            }
        }
    }

    let mut builder = String::new();
    while let Some(entry) = stack.pop_back() {
        builder = entry.to_string() + &builder;
    }
    builder
}

#[derive(Clone, Eq, PartialEq)]
enum Markdown {
    // none
    None,
    // italics
    Star,
    // bold
    DoubleStar,
    // italics
    Underscore,
    // bold
    DoubleUnderscore,
    // code
    Backtick,
    // nothing
    DoubleBacktick,
    // multiline code
    TripleBacktick,
    // multiline code - a triple backtick with a language
    Language { lang: String, lang_done: bool },
    // nothing
    Tilde,
    // strikethrough
    DoubleTilde,
}

impl ToString for Markdown {
    fn to_string(&self) -> String {
        match self {
            Markdown::None => String::new(),
            Markdown::Star => String::from("*"),
            Markdown::DoubleStar => String::from("**"),
            Markdown::Underscore => String::from("_"),
            Markdown::DoubleUnderscore => String::from("__"),
            Markdown::Backtick => String::from("`"),
            Markdown::DoubleBacktick => String::from("``"),
            Markdown::TripleBacktick => String::from("```"),
            Markdown::Language { lang, lang_done: _ } => format!("```{lang}"),
            Markdown::Tilde => String::from("~"),
            Markdown::DoubleTilde => String::from("~~"),
        }
    }
}

struct StackEntry {
    md: Markdown,
    text: String,
}

impl ToString for StackEntry {
    fn to_string(&self) -> String {
        self.md.to_string() + &self.text
    }
}

impl From<Markdown> for StackEntry {
    fn from(value: Markdown) -> Self {
        Self {
            md: value,
            text: String::new(),
        }
    }
}

impl StackEntry {
    fn new(md: Markdown, text: String) -> Self {
        Self { md, text }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_nothing() {
        let nothing = "";
        assert_eq!(text_to_html(nothing).as_str(), nothing);
    }

    #[test]
    fn test_something() {
        let test_str = "hello world";
        assert_eq!(text_to_html(test_str).as_str(), test_str);
    }

    #[test]
    fn test_star() {
        let test_str = "*hello world*";
        let expected = "<em>hello world</em>";
        assert_eq!(text_to_html(test_str).as_str(), expected);
    }

    #[test]
    fn test_double_star() {
        let test_str = "**hello world**";
        let expected = "<strong>hello world</strong>";
        assert_eq!(text_to_html(test_str).as_str(), expected);
    }

    #[test]
    fn test_underscore() {
        let test_str = "_hello world_";
        let expected = "<em>hello world</em>";
        assert_eq!(text_to_html(test_str).as_str(), expected);
    }

    #[test]
    fn test_double_underscore() {
        let test_str = "__hello world__";
        let expected = "<strong>hello world</strong>";
        assert_eq!(text_to_html(test_str).as_str(), expected);
    }

    #[test]
    fn test_double_tilde() {
        let test_str = "~~hello world~~";
        let expected = "<s>hello world</s>";
        assert_eq!(text_to_html(test_str).as_str(), expected);
    }

    #[test]
    fn test_backtick() {
        let test_str = "`hello world`";
        let expected = "<code>hello world</code>";
        assert_eq!(text_to_html(test_str).as_str(), expected);
    }

    #[test]
    fn test_language1() {
        let test_str = "```rust hello world```";
        let expected = "<code language=\"rust\"> hello world</code>";
        assert_eq!(text_to_html(test_str).as_str(), expected);
    }

    #[test]
    fn test_language2() {
        let test_str = "```rust\n hello world```";
        let expected = "<code language=\"rust\"> hello world</code>";
        assert_eq!(text_to_html(test_str).as_str(), expected);
    }

    #[test]
    fn test_multiple1() {
        let test_str = "hello world *hello world* __hello *world*__";
        let expected = "hello world <em>hello world</em> <strong>hello <em>world</em></strong>";
        assert_eq!(text_to_html(test_str).as_str(), expected);
    }

    #[test]
    fn test_multiple2() {
        let test_str = "hello world *hello world* __hello *world ~~world~~*__";
        let expected =
            "hello world <em>hello world</em> <strong>hello <em>world <s>world</s></em></strong>";
        assert_eq!(text_to_html(test_str).as_str(), expected);
    }

    #[test]
    fn test_partial1() {
        let test_str = "hello world ``h`ello **world** ~hello world";
        let expected = "hello world `<code>h</code>ello <strong>world</strong> ~hello world";
        assert_eq!(text_to_html(test_str).as_str(), expected);
    }

    #[test]
    fn test_empty_star() {
        let test_str = "* * *test*";
        let expected = "* * <em>test</em>";
        assert_eq!(text_to_html(test_str).as_str(), expected);
    }

    #[test]
    fn test_empty_double_star() {
        let test_str = "** ** **test**";
        let expected = "** ** <strong>test</strong>";
        assert_eq!(text_to_html(test_str).as_str(), expected);
    }

    #[test]
    fn test_empty_underscore() {
        let test_str = "_ _ _test_";
        let expected = "_ _ <em>test</em>";
        assert_eq!(text_to_html(test_str).as_str(), expected);
    }

    #[test]
    fn test_empty_double_underscore() {
        let test_str = "__ __ __test__";
        let expected = "__ __ <strong>test</strong>";
        assert_eq!(text_to_html(test_str).as_str(), expected);
    }

    #[test]
    fn test_empty_backtick() {
        let test_str = "` ` `test`";
        let expected = "` ` <code>test</code>";
        assert_eq!(text_to_html(test_str).as_str(), expected);
    }

    #[test]
    fn test_empty_triple_backtick() {
        let test_str = "``` ``` ```test```";
        let expected = "``` ``` <code>test</code>";
        assert_eq!(text_to_html(test_str).as_str(), expected);
    }

    #[test]
    fn test_empty_strikethrough() {
        let test_str = "~~ ~~ ~~test~~";
        let expected = "~~ ~~ <s>test</s>";
        assert_eq!(text_to_html(test_str).as_str(), expected);
    }
}
