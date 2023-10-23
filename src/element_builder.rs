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

pub fn text_to_html2(text: &str) -> Vec<Element> {
    todo!()
}
