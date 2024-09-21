use crossterm::style::ContentStyle;
use unicode_width::UnicodeWidthStr;

use crate::settings::Settings;

use super::{draw_fn::*, *};

pub struct ProgressBar {
    title: String,
    progress: f64,
    box_style: Option<ContentStyle>,
    text_style: Option<ContentStyle>,
}

impl ProgressBar {
    pub fn new(title: String) -> Self {
        Self {
            title,
            progress: 0.,
            box_style: None,
            text_style: None,
        }
    }

    pub fn box_style(mut self, style: ContentStyle) -> Self {
        self.box_style = Some(style);
        self
    }

    pub fn text_style(mut self, style: ContentStyle) -> Self {
        self.text_style = Some(style);
        self
    }

    pub fn update_progress(&mut self, progress: f64) {
        self.progress = progress;
    }

    pub fn update_title(&mut self, title: String) {
        self.title = title;
    }

    pub fn styles_from_settings(mut self, settings: &Settings) -> Self {
        let colorscheme = settings.get_color_scheme();
        self.box_style = Some(colorscheme.normals());
        self.text_style = Some(colorscheme.texts());
        self
    }
}

impl Screen for ProgressBar {
    fn draw(&self, frame: &mut Frame, color_scheme: &ColorScheme) -> io::Result<()> {
        let progress_size = Dims(self.title.width() as i32 + 2 + 2, 4);
        let pos = center_box_in_screen(progress_size);

        let prg = "█".repeat((self.title.width() as f64 * self.progress) as usize);

        let box_style = self.box_style.unwrap_or(color_scheme.normals());
        let text_style = self.text_style.unwrap_or(color_scheme.texts());

        draw_box(frame, pos, progress_size, box_style);
        frame.draw_styled(pos + Dims(2, 1), self.title.as_str(), text_style);
        frame.draw(pos + Dims(2, 2), prg);

        Ok(())
    }
}
