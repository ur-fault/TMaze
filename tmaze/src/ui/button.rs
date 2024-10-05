use cmaze::dims::Dims;
use unicode_width::UnicodeWidthStr;

use crate::{
    helpers::strings,
    renderer::{Cell, Frame},
    settings::theme::{Style, Theme, ThemeResolver},
};

use super::Rect;

#[derive(Debug)]
pub struct ButtonStyles {
    pub border: &'static str,
    pub highlight: &'static str,
    pub text: &'static str,

    pub disabled_border: &'static str,
    pub disabled_text: &'static str,
}

impl ButtonStyles {
    pub fn extract(&self, theme: &Theme) -> [Style; 5] {
        [
            theme[self.border],
            theme[self.highlight],
            theme[self.text],
            theme[self.disabled_border],
            theme[self.disabled_text],
        ]
    }
}

impl Default for ButtonStyles {
    fn default() -> Self {
        Self {
            border: "ui_button_border",
            highlight: "ui_button_highlight",
            text: "ui_button_text",

            disabled_border: "ui_button_disabled_border",
            disabled_text: "ui_button_disabled_text",
        }
    }
}

#[derive(Debug)]
pub struct Button {
    pub text: String,
    pub pos: Dims,
    pub size: Dims,
    pub disable_highlight: bool,
    pub set: bool,
    pub disabled: bool,
    pub styles: ButtonStyles,
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
            styles: ButtonStyles::default(),
        }
    }

    pub fn disable_highlight(mut self, disable_highlight: bool) -> Self {
        self.disable_highlight = disable_highlight;
        self
    }

    pub fn with_styles(mut self, styles: ButtonStyles) -> Self {
        self.styles = styles;
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

    fn apply_styles(&self, theme: &Theme) -> AppliedStyles {
        let disabled = self.disabled;
        let set = self.set && !disabled && !self.disable_highlight;

        let [normal, highlight, content, disabled_border, disabled_text] =
            self.styles.extract(theme);

        let normal = if disabled {
            disabled_border
        } else if set {
            highlight
        } else {
            normal
        };

        let content = if disabled { disabled_text } else { content };

        let inverted_bg = Style::invert(if set { highlight } else { normal });

        AppliedStyles {
            normal,
            content: if set { inverted_bg } else { content },
        }
    }
}

impl Button {
    pub fn draw_colored(&self, frame: &mut Frame, theme: &Theme) {
        // let set = self.set && !self.disabled && !self.disable_highlight;

        let AppliedStyles { normal, content } = self.apply_styles(theme);

        // Box
        frame.draw(self.pos, Rect::sized(self.size), normal);

        // Background
        frame.fill_rect(
            self.pos + Dims(1, 1),
            self.size - Dims(2, 2),
            Cell::styled(' ', content),
        );

        // Text (content)
        let text_rect = Rect::sized_at(self.pos + Dims(1, 1), self.size - Dims(2, 2))
            .centered(Dims(self.text.width() as i32, 1));
        let text = strings::trim_center(self.text.as_str(), text_rect.size().0 as usize);
        let style = content;

        frame.draw(text_rect.start, text, style);
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

struct AppliedStyles {
    normal: Style,
    content: Style,
}

pub fn button_theme_resolver() -> ThemeResolver {
    let mut resolver = ThemeResolver::new();
    resolver
        .link("ui.button.border", "border")
        .link("ui.button.highlight", "highlight")
        .link("ui.button.text", "text")
        .link("ui.button.disabled.border", "disabled.border")
        .link("ui.button.disabled.text", "disabled.text");

    resolver
}
