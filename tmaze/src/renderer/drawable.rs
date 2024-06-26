use cmaze::core::Dims;
use crossterm::style::ContentStyle;

use super::{Cell, Frame};

pub trait Drawable {
    fn draw(&self, pos: Dims, frame: &mut Frame);
    fn draw_with_style(&self, pos: Dims, frame: &mut Frame, style: ContentStyle);
}

impl Drawable for char {
    fn draw(&self, pos: Dims, frame: &mut Frame) {
        frame.put_char(pos, *self);
    }

    fn draw_with_style(&self, pos: Dims, frame: &mut Frame, style: ContentStyle) {
        frame.put_char_styled(pos, *self, style);
    }
}

impl Drawable for String {
    fn draw(&self, pos: Dims, frame: &mut Frame) {
        self.as_str().draw(pos, frame);
    }

    fn draw_with_style(&self, pos: Dims, frame: &mut Frame, style: ContentStyle) {
        self.as_str().draw_with_style(pos, frame, style);
    }
}

impl<'a> Drawable for &'a str {
    fn draw(&self, pos: Dims, frame: &mut Frame) {
        self.draw_with_style(pos, frame, ContentStyle::default());
    }

    fn draw_with_style(&self, pos: Dims, frame: &mut Frame, style: ContentStyle) {
        // TODO: Custom iterator which returns (total_width, char)
        for (i, character) in self.chars().enumerate() {
            frame.put_char_styled(Dims(pos.0 + i as i32, pos.1), character, style);
        }
    }
}

impl Drawable for Cell {
    fn draw(&self, pos: Dims, frame: &mut Frame) {
        frame.set(pos, *self);
    }

    fn draw_with_style(&self, pos: Dims, frame: &mut Frame, style: ContentStyle) {
        let mut cell = *self;
        if let Cell::Content(content) = &mut cell {
            content.style = style;
        }

        frame.set(pos, cell);
    }
}

impl<D: Drawable> Drawable for (D, ContentStyle) {
    fn draw(&self, pos: Dims, frame: &mut Frame) {
        self.0.draw_with_style(pos, frame, self.1);
    }

    fn draw_with_style(&self, pos: Dims, frame: &mut Frame, style: ContentStyle) {
        self.0.draw_with_style(pos, frame, style);
    }
}
