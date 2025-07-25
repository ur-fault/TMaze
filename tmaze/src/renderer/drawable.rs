use cmaze::dims::Dims;

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
