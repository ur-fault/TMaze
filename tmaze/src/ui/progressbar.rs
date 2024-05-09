use std::cell::RefCell;

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
        let pos = box_center_screen(progress_size);

        let mut context = DrawContext {
            frame: &RefCell::new(frame),
            style: self.box_style,
            rect: None,
        };

        context.draw_box(pos, progress_size);
        context.draw_str_styled(pos + Dims(2, 1), &self.title, self.text_style);
        context.draw_str(
            pos + Dims(2, 2),
            &"█".repeat((self.title.width() as f64 * self.progress) as usize),
        );

        Ok(())
    }
}
