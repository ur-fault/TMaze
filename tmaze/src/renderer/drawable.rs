use cmaze::dims::Dims;
use unicode_width::{UnicodeWidthChar as _, UnicodeWidthStr as _};

use crate::settings::theme::Style;

use super::{Cell, GMutView};

pub trait Drawable<S = ()> {
    fn draw(&self, pos: Dims, frame: &mut GMutView, styles: S);
}

impl<D: Drawable> Drawable for &D {
    fn draw(&self, pos: Dims, frame: &mut GMutView, styles: ()) {
        (**self).draw(pos, frame, styles);
    }
}

impl Drawable<Style> for char {
    fn draw(&self, pos: Dims, frame: &mut GMutView, styles: Style) {
        frame.put_char(pos, *self, styles);
    }
}

impl Drawable<Style> for String {
    fn draw(&self, pos: Dims, frame: &mut GMutView, styles: Style) {
        self.as_str().draw(pos, frame, styles);
    }
}

impl Drawable<Style> for &'_ str {
    fn draw(&self, pos: Dims, frame: &mut GMutView, styles: Style) {
        let mut x = 0;
        for character in self.chars() {
            x += frame.put_char(Dims(pos.0 + x as i32, pos.1), character, styles);
        }
    }
}

impl Drawable for Cell {
    fn draw(&self, pos: Dims, frame: &mut GMutView, _styles: ()) {
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
        let Dims(fw, fh) = frame_size;
        let Dims(sw, sh) = self.size();

        use Align::*;
        match align {
            TopLeft => Dims(0, 0),
            TopCenter => Dims((fw - sw) / 2, 0),
            TopRight => Dims(fw - sw, 0),
            CenterLeft => Dims(0, (fh - sh) / 2),
            Center => Dims((fw - sw) / 2, (fh - sh) / 2),
            CenterRight => Dims(fw - sw, (fh - sh) / 2),
            BottomLeft => Dims(0, fh - sh),
            BottomCenter => Dims((fw - sw) / 2, fh - sh),
            BottomRight => Dims(fw - sw, fh - sh),
        }
    }

    fn draw_aligned(&self, align: Align, frame: &mut GMutView, styles: S) {
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
    fn draw(&self, pos: Dims, frame: &mut GMutView, _: ()) {
        self.0.draw(pos, frame, self.1.clone());
    }
}

impl<T: SizedDrawable<S>, S: Clone> SizedDrawable for Styled<T, S> {
    fn size(&self) -> Dims {
        self.0.size()
    }
}
