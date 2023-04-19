use crossterm::style::{Color, ContentStyle};

pub fn term_size() -> (u16, u16) {
    let (w, h) = crossterm::terminal::size().unwrap_or((100, 100));
    (w, h)
}

pub struct ContentStyleBuilder {
    style: ContentStyle,
}

pub fn style() -> ContentStyleBuilder {
    ContentStyleBuilder {
        style: ContentStyle::default(),
    }
}

impl ContentStyleBuilder {
    pub fn f(mut self, color: Color) -> Self {
        self.style.foreground_color = Some(color);
        self
    }

    pub fn b(mut self, color: Color) -> Self {
        self.style.background_color = Some(color);
        self
    }

    pub fn u(mut self, color: Color) -> Self {
        self.style.underline_color = Some(color);
        self
    }

    pub fn a(mut self, attr: crossterm::style::Attribute) -> Self {
        self.style.attributes.toggle(attr);
        self
    }

    pub fn build(self) -> ContentStyle {
        self.style
    }
}
