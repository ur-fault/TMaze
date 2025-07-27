use cmaze::dims::Dims;
use unicode_width::{UnicodeWidthChar as _, UnicodeWidthStr as _};

use crate::settings::theme::Style;

use super::{Cell, Frame};

pub trait Drawable<S = ()> {
    fn draw(&self, pos: Dims, frame: &mut dyn Frame, styles: S);
}

impl<D: Drawable> Drawable for &D {
    fn draw(&self, pos: Dims, frame: &mut dyn Frame, styles: ()) {
        (**self).draw(pos, frame, styles);
    }
}

impl Drawable<Style> for char {
    fn draw(&self, pos: Dims, frame: &mut dyn Frame, styles: Style) {
        frame.put_char(pos, *self, styles);
    }
}

impl Drawable<Style> for String {
    fn draw(&self, pos: Dims, frame: &mut dyn Frame, styles: Style) {
        self.as_str().draw(pos, frame, styles);
    }
}

impl Drawable<Style> for &'_ str {
    fn draw(&self, pos: Dims, frame: &mut dyn Frame, styles: Style) {
        let mut x = 0;
        for character in self.chars() {
            x += frame.put_char(Dims(pos.0 + x as i32, pos.1), character, styles);
        }
    }
}

impl Drawable for Cell {
    fn draw(&self, pos: Dims, frame: &mut dyn Frame, _styles: ()) {
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

    fn align(&self, align: Align, frame_size: Dims) -> Dims {
        let Dims(width, height) = frame_size;
        let size = self.size();

        match align {
            Align::TopLeft => Dims(0, 0),
            Align::TopCenter => Dims((width - size.0) / 2, 0),
            Align::TopRight => Dims(width - size.0, 0),
            Align::CenterLeft => Dims(0, (height - size.1) / 2),
            Align::Center => Dims((width - size.0) / 2, (height - size.1) / 2),
            Align::CenterRight => Dims(width - size.0, (height - size.1) / 2),
            Align::BottomLeft => Dims(0, height - size.1),
            Align::BottomCenter => Dims((width - size.0) / 2, height - size.1),
            Align::BottomRight => Dims(width - size.0, height - size.1),
        }
    }

    fn draw_aligned(&self, align: Align, frame: &mut dyn Frame, styles: S) {
        let pos = self.align(align, frame.size());
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

pub struct Styled<T, S>(pub T, pub S);

impl<T: Drawable<S>, S: Clone> Drawable for Styled<T, S> {
    fn draw(&self, pos: Dims, frame: &mut dyn Frame, _: ()) {
        self.0.draw(pos, frame, self.1.clone());
    }
}

impl<T: SizedDrawable<S>, S: Clone> SizedDrawable for Styled<T, S> {
    fn size(&self) -> Dims {
        self.0.size()
    }
}
