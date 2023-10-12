use std::collections::VecDeque;

/// markdowns
/// text to html, with support for the following
/// - italics
/// - bold
/// - strikethrough
/// - code
/// - multiline code
/// - multiline code with a language
///

pub fn text_to_html(text: &str) -> String {
    let mut stack: VecDeque<StackEntry> = VecDeque::new();
    stack.push_back(Markdown::Line.into());

    // empty tags or just whitespace are not allowed
    let convert_html = |stack: &mut VecDeque<StackEntry>, tag: &str, md: Markdown| {
        let entry = match stack.pop_back() {
            Some(x) => x,
            None => {
                // should never happen
                stack.push_back(md.into());
                return;
            }
        };

        if std::mem::discriminant(&md) != std::mem::discriminant(&entry.md) {
            // if there was an error and the 2 were not the same type, put it back.
            stack.push_back(entry);
            stack.push_back(md.into());
            return;
        }

        if entry.text.trim().is_empty() {
            stack.push_back(StackEntry::new(
                Markdown::Line,
                format!("{}{}", md.to_string(), entry.text),
            ));
            stack.push_back(md.into());
        } else {
            // this is specifically designed to work with prismjs. doing this string parsing stuff
            // simplifies the state machine a lot.
            // get the text and either append it to the previous element or push it on the stack.
            let text = match tag {
                "language" => {
                    let default = ("text".to_string(), entry.text.clone());
                    let (language, text) = match entry.text.find('\n') {
                        Some(x) => {
                            let before = entry.text[0..x].to_string();
                            let after: String = entry.text.chars().skip(x + 1).collect();
                            match before.trim() {
                                x if !x.is_empty() => (x.to_string(), after),
                                _ => default,
                            }
                        }
                        None => match entry.text.find(' ') {
                            Some(x) => {
                                let before = entry.text[0..x].to_string();
                                let after: String = entry.text.chars().skip(x + 1).collect();
                                match before.trim() {
                                    x if !x.is_empty() => (x.to_string(), after),
                                    _ => default,
                                }
                            }
                            None => default,
                        },
                    };
                    format!(
                        "<pre><code class=\"language-{language}\">{}</code></pre>",
                        text.trim()
                    )
                }
                "code" => {
                    format!(
                        "<pre><code class=\"language-text\">{}</code></pre>",
                        entry.text.trim()
                    )
                }
                _ => format!("<{tag}>{}</{tag}>", entry.text.trim()),
            };

            if let Some(to_append) = stack.back_mut() {
                to_append.text += &text;
            } else {
                // should never happen
                stack.push_back(StackEntry::new(Markdown::Line, text));
            }
        }
    };

    let prev_matches = |stack: &VecDeque<StackEntry>, md: Markdown| {
        stack.back().map(|x| x.md == md).unwrap_or_default()
    };

    // fold back of stack into previous entry
    let fold_prev = |stack: &mut VecDeque<StackEntry>| {
        let text = stack.pop_back().map(|x| x.to_string()).unwrap_or_default();
        if let Some(entry) = stack.back_mut() {
            entry.text += &text;
        } else {
            // should never happen
            stack.push_back(StackEntry::new(Markdown::Line, text));
        }
    };

    for char in text.chars() {
        let (prev_md, prev_empty) = stack
            .back()
            .map(|x| (x.md.clone(), x.text.is_empty()))
            .expect("stack should not be empty");
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
                            convert_html(&mut stack, "language", Markdown::TripleBacktick);
                        } else {
                            stack.push_back(Markdown::TripleBacktick.into());
                        }
                    } else {
                        // the pattern looks like this: ``[\w+]`. Make a code segment.
                        let text = stack.pop_back().map(|x| x.text).unwrap_or_default();
                        let html =
                            format!("`<pre><code class=\"language-text\">{text}</code></pre>");
                        if let Some(entry) = stack.back_mut() {
                            entry.text += &html;
                        } else {
                            // should never happen
                            stack.push_back(StackEntry::new(Markdown::Line, html));
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
            '#' => match prev_md {
                Markdown::Line => {
                    if prev_empty {
                        stack.pop_back();
                        stack.push_back(Markdown::H1.into());
                    } else if let Some(entry) = stack.back_mut() {
                        entry.text.push('#');
                    }
                }
                Markdown::H1 => {
                    if prev_empty {
                        stack.pop_back();
                        stack.push_back(Markdown::H2.into());
                    } else if let Some(entry) = stack.back_mut() {
                        entry.text.push('#');
                    }
                }
                Markdown::H2 => {
                    if prev_empty {
                        stack.pop_back();
                        stack.push_back(Markdown::H3.into());
                    } else if let Some(entry) = stack.back_mut() {
                        entry.text.push('#');
                    }
                }
                Markdown::H3 => {
                    if prev_empty {
                        stack.pop_back();
                        stack.push_back(Markdown::H4.into());
                    } else if let Some(entry) = stack.back_mut() {
                        entry.text.push('#');
                    }
                }
                Markdown::H4 => {
                    if prev_empty {
                        stack.pop_back();
                        stack.push_back(Markdown::H5.into());
                    } else if let Some(entry) = stack.back_mut() {
                        entry.text.push('#');
                    }
                }
                Markdown::H5 => {
                    if let Some(entry) = stack.back_mut() {
                        entry.text.push('#');
                    }
                }
                _ => stack.push_back(Markdown::H1.into()),
            },
            '\n' => match prev_md {
                Markdown::TripleBacktick => {
                    if let Some(entry) = stack.back_mut() {
                        entry.text += "\n";
                    }
                }
                _ => {
                    if let Some(entry) = stack.back_mut() {
                        entry.text.push('\n');
                    }
                    stack.push_back(Markdown::Line.into());
                }
            },
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

    let mut builder = String::new();
    while let Some(entry) = stack.pop_back() {
        builder = entry.to_string() + &builder;
    }
    builder
}

#[derive(Clone, Eq, PartialEq)]
enum Markdown {
    // a line of text
    Line,
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
    // nothing
    Tilde,
    // strikethrough
    DoubleTilde,
    // octothorpe becomes heading
    H1,
    // double octothorpe
    H2,
    // 3x octothorpe
    H3,
    // 4x octothorpe
    H4,
    // 5x octothorpe
    H5,
}

impl ToString for Markdown {
    fn to_string(&self) -> String {
        match self {
            Markdown::Line => String::new(),
            Markdown::Star => String::from("*"),
            Markdown::DoubleStar => String::from("**"),
            Markdown::Underscore => String::from("_"),
            Markdown::DoubleUnderscore => String::from("__"),
            Markdown::Backtick => String::from("`"),
            Markdown::DoubleBacktick => String::from("``"),
            Markdown::TripleBacktick => String::from("```"),
            Markdown::Tilde => String::from("~"),
            Markdown::DoubleTilde => String::from("~~"),
            Markdown::H1 => String::from("#"),
            Markdown::H2 => String::from("##"),
            Markdown::H3 => String::from("###"),
            Markdown::H4 => String::from("####"),
            Markdown::H5 => String::from("#####"),
        }
    }
}

struct StackEntry {
    md: Markdown,
    text: String,
}

impl ToString for StackEntry {
    fn to_string(&self) -> String {
        let get_heading_text = |tag: &str| {
            if self.text.len() < 2 {
                return self.md.to_string() + &self.text;
            }

            let first = self.text[0..1].to_string();
            let second = self.text[1..].trim();

            if first.trim().is_empty() && !second.is_empty() {
                format!("<{tag}>{second}</{tag}>")
            } else {
                self.md.to_string() + &self.text
            }
        };

        match self.md {
            Markdown::H1 => get_heading_text("h1"),
            Markdown::H2 => get_heading_text("h2"),
            Markdown::H3 => get_heading_text("h3"),
            Markdown::H4 => get_heading_text("h4"),
            Markdown::H5 => get_heading_text("h5"),
            _ => self.md.to_string() + &self.text,
        }
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
        let expected = "<pre><code class=\"language-text\">hello world</code></pre>";
        assert_eq!(text_to_html(test_str).as_str(), expected);
    }

    #[test]
    fn test_language1() {
        let test_str = "```rust hello world```";
        let expected = "<pre><code class=\"language-rust\">hello world</code></pre>";
        assert_eq!(text_to_html(test_str).as_str(), expected);
    }

    #[test]
    fn test_language2() {
        let test_str = "```rust\n hello world```";
        let expected = "<pre><code class=\"language-rust\">hello world</code></pre>";
        assert_eq!(text_to_html(test_str).as_str(), expected);
    }

    #[test]
    fn test_language3() {
        let test_str = r#"```rust
        hello world
        ```"#;
        let expected = "<pre><code class=\"language-rust\">hello world</code></pre>";
        assert_eq!(text_to_html(test_str).as_str(), expected);
    }

    #[test]
    fn test_h1() {
        let test_str = "# heading";
        let expected = "<h1>heading</h1>";
        assert_eq!(text_to_html(test_str).as_str(), expected);

        let test_str = "#heading";
        let expected = "#heading";
        assert_eq!(text_to_html(test_str).as_str(), expected);

        let test_str = "# # heading";
        let expected = "<h1># heading</h1>";
        assert_eq!(text_to_html(test_str).as_str(), expected);
    }

    #[test]
    fn test_h2() {
        let test_str = "## heading";
        let expected = "<h2>heading</h2>";
        assert_eq!(text_to_html(test_str).as_str(), expected);

        let test_str = "##heading";
        let expected = "##heading";
        assert_eq!(text_to_html(test_str).as_str(), expected);

        let test_str = "## ## heading";
        let expected = "<h2>## heading</h2>";
        assert_eq!(text_to_html(test_str).as_str(), expected);
    }

    #[test]
    fn test_h3() {
        let test_str = "### heading";
        let expected = "<h3>heading</h3>";
        assert_eq!(text_to_html(test_str).as_str(), expected);

        let test_str = "###heading";
        let expected = "###heading";
        assert_eq!(text_to_html(test_str).as_str(), expected);

        let test_str = "### ### heading";
        let expected = "<h3>### heading</h3>";
        assert_eq!(text_to_html(test_str).as_str(), expected);
    }

    #[test]
    fn test_h4() {
        let test_str = "#### heading";
        let expected = "<h4>heading</h4>";
        assert_eq!(text_to_html(test_str).as_str(), expected);

        let test_str = "####heading";
        let expected = "####heading";
        assert_eq!(text_to_html(test_str).as_str(), expected);

        let test_str = "#### #### heading";
        let expected = "<h4>#### heading</h4>";
        assert_eq!(text_to_html(test_str).as_str(), expected);
    }

    #[test]
    fn test_h5() {
        let test_str = "##### heading";
        let expected = "<h5>heading</h5>";
        assert_eq!(text_to_html(test_str).as_str(), expected);

        let test_str = "#####heading";
        let expected = "#####heading";
        assert_eq!(text_to_html(test_str).as_str(), expected);

        let test_str = "##### ##### heading";
        let expected = "<h5>##### heading</h5>";
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
        let expected = "hello world `<pre><code class=\"language-text\">h</code></pre>ello <strong>world</strong> ~hello world";
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
        let expected = "` ` <pre><code class=\"language-text\">test</code></pre>";
        assert_eq!(text_to_html(test_str).as_str(), expected);
    }

    #[test]
    fn test_empty_triple_backtick() {
        let test_str = "``` ``` ```test```";
        let expected = "``` ``` <pre><code class=\"language-text\">test</code></pre>";
        assert_eq!(text_to_html(test_str).as_str(), expected);
    }

    #[test]
    fn test_empty_strikethrough() {
        let test_str = "~~ ~~ ~~test~~";
        let expected = "~~ ~~ <s>test</s>";
        assert_eq!(text_to_html(test_str).as_str(), expected);
    }
}
