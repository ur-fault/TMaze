use std::{cell::RefCell, ops::DerefMut};

use crate::core::*;
use crate::renderer::{Frame, Renderer};

use crossterm::style::ContentStyle;

pub use substring::Substring;

pub fn draw_box<'a>(
    mut frame: impl DerefMut<Target = &'a mut Frame>,
    pos: Dims,
    size: Dims,
    style: ContentStyle,
) {
    draw_str(
        &mut *frame,
        pos.0,
        pos.1,
        &format!("╭{}╮", "─".repeat(size.0 as usize - 2)),
        style,
    );

    for y in pos.1 + 1..pos.1 + size.1 - 1 {
        draw_char(&mut *frame, pos.0, y, '│', style);
        draw_char(&mut *frame, pos.0 + size.0 - 1, y, '│', style);
    }

    draw_str(
        frame,
        pos.0,
        pos.1 + size.1 - 1,
        &format!("╰{}╯", "─".repeat(size.0 as usize - 2)),
        style,
    );
}

pub fn draw_str<'a>(
    mut frame: impl DerefMut<Target = &'a mut Frame>,
    mut x: i32,
    y: i32,
    mut text: &str,
    style: ContentStyle,
) {
    if y < 0 {
        return;
    }

    if x < 0 && text.len() as i32 > -x + 1 {
        text = text.substring(-x as usize, text.len() - 1);
        x = 0;
    }

    if x > u16::MAX as i32 || y > u16::MAX as i32 {
        return;
    }

    frame.draw((x as u16, y as u16), (text, style));
}

pub fn draw_char<'a>(
    mut frame: impl DerefMut<Target = &'a mut Frame>,
    x: i32,
    y: i32,
    text: char,
    style: ContentStyle,
) {
    if y < 0 || x < 0 || x > u16::MAX as i32 || y > u16::MAX as i32 {
        return;
    }

    // renderer.draw_char(x as u16, y as u16, text, style);
    frame.draw((x as u16, y as u16), (text, style));
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Rect {
    pub start: Dims,
    pub end: Dims,
}

impl Rect {
    pub fn new(start: Dims, end: Dims) -> Self {
        Self { start, end }
    }

    pub fn new_sized(start: Dims, size: Dims) -> Self {
        Self::new(start, Dims(start.0 + size.0, start.1 + size.1))
    }

    pub fn size(&self) -> Dims {
        Dims(self.end.0 - self.start.0, self.end.1 - self.start.1) + Dims(1, 1)
    }

    pub fn contains(&self, pos: Dims) -> bool {
        pos.0 >= self.start.0 && pos.0 <= self.end.0 && pos.1 >= self.start.1 && pos.1 <= self.end.1
    }

    pub fn trim_absolute<'a>(&'a self, text: &'a impl AsRef<str>, mut pos: Dims) -> (&str, Dims) {
        let mut text = text.as_ref();
        let size = self.size();

        if pos.1 < self.start.1 || pos.1 > self.end.1 {
            return ("", pos);
        }

        if pos.0 < self.start.0 {
            let offset = self.start.0 - pos.0;
            text = text.substring(offset as usize, text.chars().count());
            pos = Dims(self.start.0, pos.1);
        }

        if text.chars().count() as i32 + pos.0 > self.end.0 {
            let x = size.0 - (pos.0 - self.start.0);
            let x = x.max(0) as usize;
            text = text.substring(0, x);
        }

        (text, pos)
    }

    #[allow(dead_code)]
    pub fn trim_relative<'a>(&'a self, text: &'a impl AsRef<str>, pos: Dims) -> (&str, Dims) {
        let (text, pos) = self.trim_absolute(text, pos + self.start);
        (text, pos - self.start)
    }

    pub fn with_margin(&self, margin: Dims) -> Self {
        Self {
            start: self.start + margin,
            end: self.end - margin,
        }
    }
}

#[derive(Clone, Copy)]
pub struct DrawContext<'a> {
    pub frame: &'a RefCell<&'a mut Frame>,
    pub style: ContentStyle,
    pub rect: Option<Rect>,
}

#[allow(dead_code)]
impl<'a> DrawContext<'a> {
    pub fn draw_char(&mut self, pos: Dims, text: char) {
        self.draw_char_styled(pos, text, self.style);
    }

    pub fn draw_str(&mut self, pos: Dims, text: &str) {
        self.draw_str_styled(pos, text, self.style);
    }

    pub fn draw_box(&mut self, pos: Dims, size: Dims) {
        draw_box(self.frame.borrow_mut(), pos, size, self.style);
    }

    pub fn draw_char_styled(&mut self, pos: Dims, text: char, style: ContentStyle) {
        if self.rect.as_ref().map_or(true, |f| f.contains(pos)) {
            draw_char(self.frame.borrow_mut(), pos.0, pos.1, text, style);
        }
    }

    pub fn draw_str_styled(&mut self, pos: Dims, text: &str, style: ContentStyle) {
        let (text, pos) = self
            .rect
            .as_ref()
            .map_or((text, pos), |f| f.trim_absolute(&text, pos));
        draw_str(self.frame.borrow_mut(), pos.0, pos.1, text, style);
    }

    pub fn draw_box_styled(&mut self, pos: Dims, size: Dims, style: ContentStyle) {
        draw_box(self.frame.borrow_mut(), pos, size, style);
    }
}

#[cfg(test)]
mod tests {
    use super::{Dims, Rect};

    #[test]
    fn frame_trim_absolute() {
        let frame = Rect::new_sized(Dims(0, 0), Dims(3, 1));
        let (text, ..) = frame.trim_absolute(&"123456", Dims(0, 0));
        assert_eq!(text, "123");

        let (text, ..) = frame.trim_absolute(&"123456", Dims(1, 0));
        assert_eq!(text, "12");

        let (text, ..) = frame.trim_absolute(&"123456", Dims(-1, 0));
        assert_eq!(text, "234");

        let (text, ..) = frame.trim_absolute(&"123456", Dims(-4, 0));
        assert_eq!(text, "56");

        let (text, ..) = frame.trim_absolute(&"123456", Dims(-3, 0));
        assert_eq!(text, "456");
    }
}
