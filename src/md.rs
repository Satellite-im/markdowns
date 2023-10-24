#[derive(Clone, Eq, PartialEq)]
pub enum Markdown {
    // a line of text
    Line,
    NewLine,
    // italics
    Star,
    // bold
    DoubleStar,
    // italics
    Underscore,
    // bold
    DoubleUnderscore,
    // escaping
    Backslash,
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
    // block quote
    GreaterThan,
    BlockQuote,
}

impl ToString for Markdown {
    fn to_string(&self) -> String {
        match self {
            Markdown::Line | Markdown::BlockQuote => String::new(),
            Markdown::NewLine => String::from("\n"),
            Markdown::Star => String::from("*"),
            Markdown::DoubleStar => String::from("**"),
            Markdown::Underscore => String::from("_"),
            Markdown::DoubleUnderscore => String::from("__"),
            Markdown::Backtick => String::from("`"),
            Markdown::Backslash => String::from("\\"),
            Markdown::DoubleBacktick => String::from("``"),
            Markdown::TripleBacktick => String::from("```"),
            Markdown::Tilde => String::from("~"),
            Markdown::DoubleTilde => String::from("~~"),
            Markdown::H1 => String::from("#"),
            Markdown::H2 => String::from("##"),
            Markdown::H3 => String::from("###"),
            Markdown::H4 => String::from("####"),
            Markdown::H5 => String::from("#####"),
            Markdown::GreaterThan => String::from(">"),
        }
    }
}
