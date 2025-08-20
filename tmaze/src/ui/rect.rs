use cmaze::dims::*;

use substring::Substring as _;

use crate::{
    helpers::box_center,
    renderer::{draw::Draw, GMutView},
    settings::theme::{Style, TerminalColorScheme, ThemeResolver},
};

use super::draw_box;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Rect {
    pub start: Dims,
    pub end: Dims,
}

impl Rect {
    pub fn new(start: Dims, end: Dims) -> Self {
        Self { start, end }
    }

    pub fn sized_at(start: Dims, size: Dims) -> Self {
        Self::new(start, Dims(start.0 + size.0, start.1 + size.1) - Dims(1, 1))
    }

    pub fn sized(size: Dims) -> Self {
        Self::sized_at(Dims(0, 0), size)
    }

    pub fn size(&self) -> Dims {
        Dims(self.end.0 - self.start.0, self.end.1 - self.start.1) + Dims(1, 1)
    }

    pub fn contains(&self, pos: Dims) -> bool {
        (self.start.0..=self.end.0).contains(&pos.0) && (self.start.1..=self.end.1).contains(&pos.1)
    }

    // TODO: make it generic over `Borrow`
    pub fn trim_absolute<'a>(
        &'a self,
        text: &'a impl AsRef<str>,
        mut pos: Dims,
    ) -> (&'a str, Dims) {
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

    pub fn trim_relative<'a>(&'a self, text: &'a impl AsRef<str>, pos: Dims) -> (&'a str, Dims) {
        let (text, pos) = self.trim_absolute(text, pos + self.start);
        (text, pos - self.start)
    }
}

impl Rect {
    pub fn centered(&self, inner: Dims) -> Self {
        let pos = box_center(self.start, self.end, inner);
        Self::sized_at(pos, inner)
    }

    pub fn centered_x(&self, inner: Dims) -> Self {
        let pos = Dims(self.start.0 + (self.size().0 - inner.0) / 2, self.start.1);
        Self::sized_at(pos, inner)
    }

    pub fn centered_y(&self, inner: Dims) -> Self {
        let pos = Dims(self.start.0, self.start.1 + (self.size().1 - inner.1) / 2);
        Self::sized_at(pos, inner)
    }

    pub fn margin(&self, margin: Dims) -> Self {
        Self {
            start: self.start + margin,
            end: self.end - margin,
        }
    }

    pub fn offset(&self, offset: Dims) -> Self {
        Self {
            start: self.start + offset,
            end: self.end + offset,
        }
    }
}

impl Rect {
    pub fn split_x(&self, ratio: Offset) -> (Self, Self) {
        let chars = ratio.to_abs(self.size().0);
        let left = Rect::sized_at(self.start, Dims(chars, self.size().1));
        let right = Rect::sized_at(
            Dims(self.start.0 + chars, self.start.1),
            Dims(self.size().0 - chars, self.size().1),
        );
        (left, right)
    }

    pub fn split_x_end(&self, ratio: Offset) -> (Self, Self) {
        let chars = self.size().0 - ratio.to_abs(self.size().0);
        let left = Rect::sized_at(self.start, Dims(chars, self.size().1));
        let right = Rect::sized_at(
            Dims(self.start.0 + chars, self.start.1),
            Dims(self.size().0 - chars, self.size().1),
        );
        (left, right)
    }

    pub fn split_y(&self, ratio: Offset) -> (Self, Self) {
        let chars = ratio.to_abs(self.size().1);
        let top = Rect::sized_at(self.start, Dims(self.size().0, chars));
        let bottom = Rect::sized_at(
            Dims(self.start.0, self.start.1 + chars),
            Dims(self.size().0, self.size().1 - chars),
        );
        (top, bottom)
    }

    pub fn split_y_end(&self, ratio: Offset) -> (Self, Self) {
        let Dims(width, height) = self.size();
        let chars = height - ratio.to_abs(height);

        let top = Rect::sized_at(self.start, Dims(width, chars));
        let bottom = Rect::sized_at(
            Dims(self.start.0, self.start.1 + chars),
            Dims(width, height - chars),
        );
        (top, bottom)
    }
}

impl Rect {
    pub fn render(&self, frame: &mut GMutView, style: Style, scheme: &TerminalColorScheme) {
        draw_box(frame, self.start, self.size(), style, scheme);
    }
}

impl Draw<Style> for Rect {
    fn draw(&self, pos: Dims, frame: &mut GMutView, style: Style, scheme: &TerminalColorScheme) {
        draw_box(frame, pos + self.start, self.size(), style, scheme);
    }
}

pub fn rect_theme_resolver() -> ThemeResolver {
    ThemeResolver::new()
}

#[cfg(test)]
mod tests {
    use super::{Dims, Rect};

    #[test]
    fn frame_trim_absolute() {
        let frame = Rect::sized(Dims(3, 1));
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
