use unic_emoji_char::{
    is_emoji, is_emoji_component, is_emoji_modifier, is_emoji_modifier_base, is_emoji_presentation,
};

pub fn replace_emojis(input: &str) -> String {
    let mut builder = String::new();
    let mut stack = String::new();

    for char in input.chars() {
        match char {
            ' ' => {
                builder += process_stack(&stack);
                stack.clear();
                builder.push(char);
            }
            _ => stack.push(char),
        }
    }

    builder += process_stack(&stack);
    builder
}

fn process_stack<'a>(stack: &'a str) -> &'a str {
    match stack {
        ":)" => "🙂",
        ":(" => " 🙁",
        ">:)" => "😈",
        ">:(" => "😠",
        ":/" => "🫤",
        ";)" => "😉",
        ":D" => "😁",
        "xD" => "😆",
        ":p" => "😛",
        ";p" => "😜",
        "xp" => "😝",
        ":|" => "😐",
        ":O" => "😮",
        _ => stack,
    }
}

// if this has to be changed, don't want to have to rewrite the unit tests
pub fn wrap_single_emoji_in_span(input: &str, tag: &str, class: &str) -> String {
    let input = input.trim();
    let mut indices = unic_segment::GraphemeIndices::new(input);
    let first_grapheme_is_emoji = if let Some((_, grapheme)) = indices.next() {
        !grapheme.chars().any(|char| {
            !(is_emoji(char)
                || is_emoji_component(char)
                || is_emoji_modifier(char)
                || is_emoji_modifier_base(char)
                || is_emoji_presentation(char)
                // some emojis are multiple emojis joined by this character
                || char == '\u{200d}')
        })
    } else {
        false
    };

    if first_grapheme_is_emoji && indices.next().is_none() {
        format!("<{tag} class=\"{class}\">{input}</{tag}>")
    } else {
        input.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn transform_single_emoji(input: &str) -> String {
        wrap_single_emoji_in_span(input, "span", "single-emoji")
    }

    #[test]
    fn test_single_no_emoji() {
        let input = "abc";
        let expected = "abc";
        assert_eq!(&transform_single_emoji(input), expected);
    }

    #[test]
    fn test_single_emoji() {
        let input = "😮";
        let expected = "<span class=\"single-emoji\">😮</span>";
        assert_eq!(&transform_single_emoji(input), expected);
    }

    #[test]
    fn test_single_emoji2() {
        let input = "😮  ";
        let expected = "<span class=\"single-emoji\">😮</span>";
        assert_eq!(&transform_single_emoji(input), expected);
    }

    #[test]
    fn test_double_emoji() {
        let input = "😮😮";
        let expected = "😮😮";
        assert_eq!(&transform_single_emoji(input), expected);
    }

    #[test]
    fn test_comples_emoji() {
        let input = "👨‍👩‍👦‍👦";
        let expected = "<span class=\"single-emoji\">👨‍👩‍👦‍👦</span>";
        assert_eq!(&transform_single_emoji(input), expected);
    }

    #[test]
    fn test_emoji_and_words() {
        let input = "👨‍👩‍👦‍👦abc";
        let expected = "👨‍👩‍👦‍👦abc";
        assert_eq!(&transform_single_emoji(input), expected);
    }
}
