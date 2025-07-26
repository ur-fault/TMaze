use cmaze::dims::Dims;
use unicode_width::{UnicodeWidthChar as _, UnicodeWidthStr as _};

use crate::settings::theme::Style;

use super::{Cell, Frame};

pub trait Drawable<S = ()> {
    fn draw(&self, pos: Dims, frame: &mut impl Frame, styles: S);
}

impl Drawable<Style> for char {
    fn draw(&self, pos: Dims, frame: &mut impl Frame, style: Style) {
        frame.put_char(pos, *self, style);
    }
}

impl Drawable<Style> for String {
    fn draw(&self, pos: Dims, frame: &mut impl Frame, style: Style) {
        self.as_str().draw(pos, frame, style);
    }
}

impl Drawable<Style> for &'_ str {
    fn draw(&self, pos: Dims, frame: &mut impl Frame, style: Style) {
        let mut x = 0;
        for character in self.chars() {
            x += frame.put_char(Dims(pos.0 + x as i32, pos.1), character, style);
        }
    }
}

impl Drawable for Cell {
    fn draw(&self, pos: Dims, frame: &mut impl Frame, _: ()) {
        frame[pos] = *self;
    }
}

pub enum Align {
    TopLeft,
    TopCenter,
    TopRight,
    CenterLeft,
    Center,
    CenterRight,
    BottomLeft,
    BottomCenter,
    BottomRight,
}

pub trait SizedDrawable<S = ()>: Drawable<S> {
    fn size(&self) -> Dims;

    fn draw_aligned(&self, align: Align, frame: &mut impl Frame, styles: S) {
        let size = self.size();
        let Dims(width, height) = frame.size();

        let pos = match align {
            Align::TopLeft => Dims(0, 0),
            Align::TopCenter => Dims((width - size.0) / 2, 0),
            Align::TopRight => Dims(width - size.0, 0),
            Align::CenterLeft => Dims(0, (height - size.1) / 2),
            Align::Center => Dims((width - size.0) / 2, (height - size.1) / 2),
            Align::CenterRight => Dims(width - size.0, (height - size.1) / 2),
            Align::BottomLeft => Dims(0, height - size.1),
            Align::BottomCenter => Dims((width - size.0) / 2, height - size.1),
            Align::BottomRight => Dims(width - size.0, height - size.1),
        };

        self.draw(pos, frame, styles);
    }
}

impl SizedDrawable<Style> for char {
    fn size(&self) -> Dims {
        Dims(self.width().unwrap_or(1) as i32, 1)
    }
}

impl SizedDrawable<Style> for String {
    fn size(&self) -> Dims {
        Dims(self.width() as i32, 1)
    }
}

impl SizedDrawable<Style> for &'_ str {
    fn size(&self) -> Dims {
        Dims(self.width() as i32, 1)
    }
}

impl SizedDrawable for Cell {
    fn size(&self) -> Dims {
        Dims::ONE
    }
}
