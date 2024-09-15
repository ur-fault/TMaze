use cmaze::core::Dims;
use crossterm::style::ContentStyle;
use unicode_width::UnicodeWidthStr;

use crate::{
    renderer::{Cell, Frame},
    settings::ColorScheme,
};

use super::{invert_style, merge_styles, Rect};

#[derive(Debug)]
pub struct Button {
    pub text: String,
    pub pos: Dims,
    pub size: Dims,
    pub normal: Option<ContentStyle>,
    pub highlight: Option<ContentStyle>,
    pub set: bool,
}

impl Button {
    pub fn new(text: &str, pos: Dims, size: Dims) -> Self {
        assert!(size.0 >= text.width() as i32 + 2);
        assert!(size.1 >= 3);

        Self {
            text: text.to_string(),
            pos,
            size,
            normal: None,
            highlight: None,
            set: false,
        }
    }

    pub fn normal_style(mut self, style: ContentStyle) -> Self {
        self.normal = Some(style);
        self
    }

    pub fn highlight_style(mut self, style: ContentStyle) -> Self {
        self.highlight = Some(style);
        self
    }

    pub fn draw_colored(&self, frame: &mut Frame, color_scheme: &ColorScheme) {
        let normal = self.normal.unwrap_or(color_scheme.normals());
        let highlight = self.highlight.unwrap_or(color_scheme.highlights());

        frame.draw_styled(
            self.pos,
            Rect::sized(Dims(0, 0), self.size),
            if self.set { highlight } else { normal },
        );

        frame.fill_rect(
            self.pos + Dims(1, 1),
            self.size - Dims(2, 2),
            Cell::styled(' ', if self.set { highlight } else { normal }),
        );

        let text_rect = Rect::sized(self.pos + Dims(1, 1), self.size - Dims(2, 2))
            .centered(Dims(self.text.width() as i32, 1));
        let text = format!("{}", self.text);
        let style = if self.set {
            merge_styles(invert_style(highlight), normal)
        } else {
            merge_styles(invert_style(normal), highlight)
        };

        frame.draw_styled(text_rect.start, text, style);
    }

    pub fn draw(&self, frame: &mut Frame) {
        self.draw_colored(frame, &ColorScheme::default());
    }

    pub fn detect_over(&self, pos: Dims) -> bool {
        if pos.0 >= self.pos.0
            && pos.0 < self.pos.0 + self.size.0
            && pos.1 >= self.pos.1
            && pos.1 < self.pos.1 + self.size.1
        {
            true
        } else {
            false
        }
    }

    pub fn size(&self) -> Dims {
        self.size
    }
}
