use crossterm::style::ContentStyle;
use unicode_width::UnicodeWidthStr;

use super::{draw::*, *};

pub struct ProgressBar {
    title: String,
    progress: f64,
    box_style: ContentStyle,
    text_style: ContentStyle,
}

impl ProgressBar {
    pub fn new(title: String) -> Self {
        Self {
            title,
            progress: 0.,
            box_style: ContentStyle::default(),
            text_style: ContentStyle::default(),
        }
    }

    pub fn box_style(mut self, style: ContentStyle) -> Self {
        self.box_style = style;
        self
    }

    pub fn text_style(mut self, style: ContentStyle) -> Self {
        self.text_style = style;
        self
    }

    pub fn update_progress(&mut self, progress: f64) {
        self.progress = progress;
    }

    pub fn update_title(&mut self, title: String) {
        self.title = title;
    }
}

impl Screen for ProgressBar {
    fn draw(&self, frame: &mut Frame) -> io::Result<()> {
        let progress_size = Dims(self.title.width() as i32 + 2 + 2, 4);
        let pos = center_box_in_screen(progress_size);

        let prg = "█".repeat((self.title.width() as f64 * self.progress) as usize);

        draw_box(frame, pos, progress_size, self.box_style);
        frame.draw_styled(pos + Dims(2, 1), self.title.as_str(), self.text_style);
        frame.draw(pos + Dims(2, 2), prg);

        Ok(())
    }
}
