use cmaze::core::Dims;
use crossterm::style::ContentStyle;
use unicode_width::UnicodeWidthStr;

use crate::{
    helpers::strings,
    renderer::{Cell, Frame},
    settings::{ColorScheme, Settings},
};

use super::{invert_style, merge_styles, Rect};

#[derive(Debug)]
pub struct Button {
    pub text: String,
    pub pos: Dims,
    pub size: Dims,
    pub normal_style: Option<ContentStyle>,
    pub content_style: Option<ContentStyle>,
    pub highlight_style: Option<ContentStyle>,
    pub set: bool,
    pub disabled: bool,
}

impl Button {
    pub fn new(text: &str, pos: Dims, size: Dims) -> Self {
        assert!(size.1 >= 1);

        let text = strings::trim_center(text, size.0 as usize - 2);

        Self {
            text: text.to_string(),
            pos,
            size,
            normal_style: None,
            content_style: None,
            highlight_style: None,
            set: false,
            disabled: false,
        }
    }

    pub fn normal_style(mut self, style: ContentStyle) -> Self {
        self.normal_style = Some(style);
        self
    }

    pub fn content_style(mut self, style: ContentStyle) -> Self {
        self.content_style = Some(style);
        self
    }

    pub fn highlight_style(mut self, style: ContentStyle) -> Self {
        self.highlight_style = Some(style);
        self
    }

    pub fn load_styles_from_settings(&mut self, settings: &Settings) {
        let colorscheme = settings.get_color_scheme();
        self.normal_style = Some(colorscheme.normals());
        self.content_style = Some(colorscheme.texts());
        self.highlight_style = Some(colorscheme.highlights());
    }

    pub fn styles_from_settings(mut self, settings: &Settings) -> Self {
        self.load_styles_from_settings(settings);
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
    pub fn draw_colored(&self, frame: &mut Frame, color_scheme: &ColorScheme) {
        let set = self.set && !self.disabled;

        let normal = self.normal_style.unwrap_or(color_scheme.normals());
        let highlight = self.highlight_style.unwrap_or(color_scheme.highlights());
        let content = if !self.disabled {
            self.content_style.unwrap_or(color_scheme.texts())
        } else {
            normal
        };

        let inverted_bg = invert_style(if set { highlight } else { normal });

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
        let text = self.text.as_str();
        let style = if set {
            merge_styles(invert_style(highlight), normal)
        } else {
            content
        };

        frame.draw_styled(text_rect.start, text, style);
    }

    pub fn draw(&self, frame: &mut Frame) {
        self.draw_colored(frame, &ColorScheme::default());
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
