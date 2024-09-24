use cmaze::dims::Dims;
use unicode_width::UnicodeWidthStr;

use crate::{
    helpers::strings,
    renderer::{Cell, Frame},
    settings::theme::{Style, Theme, ThemeResolver},
};

use super::Rect;

#[derive(Debug)]
pub struct Button {
    pub text: String,
    pub pos: Dims,
    pub size: Dims,
    pub disable_highlight: bool,
    pub set: bool,
    pub disabled: bool,
}

impl Button {
    pub fn new(text: String, pos: Dims, size: Dims) -> Self {
        assert!(size.1 >= 1);

        Self {
            text,
            pos,
            size,
            // normal_style: None,
            // content_style: None,
            // highlight_style: None,
            disable_highlight: false,
            set: false,
            disabled: false,
        }
    }

    pub fn disable_highlight(mut self, disable_highlight: bool) -> Self {
        self.disable_highlight = disable_highlight;
        self
    }

    pub fn set(mut self, set: bool) -> Self {
        self.set = set;
        self
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }
}

impl Button {
    pub fn draw_colored(&self, frame: &mut Frame, theme: &Theme) {
        let set = self.set && !self.disabled && !self.disable_highlight;

        let normal = theme.get("ui_button_border");
        let highlight = theme.get("ui_button_highlight");
        let content = if !self.disabled {
            theme.get("ui_button_text")
        } else {
            normal
        };

        let inverted_bg = Style::invert(if set { highlight } else { normal });

        // Box
        frame.draw_styled(
            self.pos,
            Rect::sized(self.size),
            if set { highlight } else { normal },
        );

        // Background
        frame.fill_rect(
            self.pos + Dims(1, 1),
            self.size - Dims(2, 2),
            Cell::styled(' ', if set { inverted_bg } else { content }),
        );

        // Text (content)
        let text_rect = Rect::sized_at(self.pos + Dims(1, 1), self.size - Dims(2, 2))
            .centered(Dims(self.text.width() as i32, 1));
        let text = strings::trim_center(self.text.as_str(), text_rect.size().0 as usize);
        let style = if set { highlight.invert() } else { content };

        frame.draw_styled(text_rect.start, text, style);
    }

    pub fn detect_over(&self, pos: Dims) -> bool {
        pos.0 >= self.pos.0
            && pos.0 < self.pos.0 + self.size.0
            && pos.1 >= self.pos.1
            && pos.1 < self.pos.1 + self.size.1
    }

    pub fn size(&self) -> Dims {
        self.size
    }
}

pub fn button_theme_resolver() -> ThemeResolver {
    let mut resolver = ThemeResolver::new();
    resolver
        .link("ui_button_border", "border")
        .link("ui_button_highlight", "highlight")
        .link("ui_button_text", "text");

    resolver
}
