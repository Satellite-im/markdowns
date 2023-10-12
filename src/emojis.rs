pub(crate) fn replace_emojis(input: &str) -> String {
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
