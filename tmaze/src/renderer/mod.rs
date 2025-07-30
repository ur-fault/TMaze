pub mod drawable;
pub mod helpers;

use std::{
    io::{self, stdout, Write},
    ops::IndexMut,
    panic, thread,
};

use cmaze::dims::{Dims, Offset};
use crossterm::{event::Event, execute, style::ContentStyle, terminal, QueueableCommand};
use drawable::{Align, SizedDrawable};
use unicode_width::UnicodeWidthChar;

use crate::{settings::theme::Style, ui::Rect};

use self::{drawable::Drawable, helpers::term_size};

pub struct Renderer {
    size: Dims,
    shown: FrameBuffer,
    hidden: FrameBuffer,
    full_redraw: bool,
}

impl Renderer {
    pub fn new() -> io::Result<Self> {
        let (w, h) = term_size();
        let size = Dims(w as i32, h as i32);
        let hidden = FrameBuffer::new(size);
        let shown = FrameBuffer::new(size);

        let mut ren = Renderer {
            size,
            shown,
            hidden,
            full_redraw: true,
        };

        ren.turn_on()?;

        Ok(ren)
    }

    fn turn_on(&mut self) -> io::Result<()> {
        self.register_panic_hook();

        crossterm::terminal::enable_raw_mode()?;
        crossterm::execute!(
            stdout(),
            crossterm::cursor::Hide,
            crossterm::terminal::EnterAlternateScreen,
            crossterm::event::EnableMouseCapture,
        )?;

        self.on_resize(None);

        Ok(())
    }

    fn turn_off(&mut self) -> io::Result<()> {
        self.unregiser_panic_hook();

        crossterm::execute!(
            stdout(),
            crossterm::cursor::Show,
            crossterm::terminal::LeaveAlternateScreen,
            crossterm::event::DisableMouseCapture,
        )?;
        crossterm::terminal::disable_raw_mode()?;
        Ok(())
    }

    fn register_panic_hook(&self) {
        let prev = panic::take_hook();
        panic::set_hook(Box::new(move |info| {
            let mut stdout = stdout();

            execute!(
                stdout,
                crossterm::terminal::LeaveAlternateScreen,
                crossterm::cursor::Show,
                crossterm::event::DisableMouseCapture,
            )
            .unwrap();

            crossterm::terminal::disable_raw_mode().unwrap();

            prev(info)
        }));
    }

    fn unregiser_panic_hook(&self) {
        if !thread::panicking() {
            let _ = panic::take_hook();
        }
    }

    fn on_resize(&mut self, size: Option<Dims>) {
        self.size = size.unwrap_or_else(|| terminal::size().unwrap().into());
        self.shown.resize(self.size);
        self.hidden.resize(self.size);
        self.full_redraw = true;
    }

    pub fn on_event(&mut self, event: &Event) {
        if let Event::Resize(x, y) = event {
            self.on_resize(Some((*x, *y).into()))
        }
    }

    pub fn frame(&mut self) -> &mut FrameBuffer {
        &mut self.hidden
    }

    pub fn frame_size(&self) -> Dims {
        self.size
    }

    pub fn show(&mut self) -> io::Result<()> {
        let mut tty = stdout();

        let mut style = ContentStyle::default();
        tty.queue(crossterm::style::ResetColor)?;

        for y in 0..self.size.1 {
            if self.hidden[y] == self.shown[y] && !self.full_redraw {
                continue;
            }

            tty.queue(crossterm::cursor::MoveTo(0, y as u16))?;

            for x in 0..self.size.0 {
                if let Cell::Content(c) = &self.hidden[y][x as usize] {
                    if style != c.style {
                        if style.background_color != c.style.background_color {
                            match c.style.background_color {
                                Some(x) => {
                                    tty.queue(crossterm::style::SetBackgroundColor(x))?;
                                }
                                None => {
                                    tty.queue(crossterm::style::SetBackgroundColor(
                                        crossterm::style::Color::Reset,
                                    ))?;
                                }
                            }
                        }
                        if style.foreground_color != c.style.foreground_color {
                            match c.style.foreground_color {
                                Some(x) => {
                                    tty.queue(crossterm::style::SetForegroundColor(x))?;
                                }
                                None => {
                                    tty.queue(crossterm::style::SetForegroundColor(
                                        crossterm::style::Color::Reset,
                                    ))?;
                                }
                            }
                        }
                        if style.attributes != c.style.attributes {
                            tty.queue(crossterm::style::SetAttribute(
                                crossterm::style::Attribute::Reset,
                            ))?;
                            if let Some(x) = c.style.foreground_color {
                                tty.queue(crossterm::style::SetForegroundColor(x))?;
                            }
                            if let Some(x) = c.style.background_color {
                                tty.queue(crossterm::style::SetBackgroundColor(x))?;
                            }
                            tty.queue(crossterm::style::SetAttributes(c.style.attributes))?;
                        }
                        style = c.style;
                    }
                    tty.queue(crossterm::style::Print(c.character))?;
                }
            }
        }

        tty.flush()?;
        self.full_redraw = false;

        std::mem::swap(&mut self.shown, &mut self.hidden);

        self.hidden.clear();

        Ok(())
    }
}

impl Drop for Renderer {
    fn drop(&mut self) {
        let _ = self.turn_off();
    }
}

pub struct MouseGuard;

impl MouseGuard {
    pub fn new() -> io::Result<Self> {
        crossterm::execute!(stdout(), crossterm::event::DisableMouseCapture)?;
        Ok(MouseGuard)
    }
}

impl Drop for MouseGuard {
    fn drop(&mut self) {
        let _ = crossterm::execute!(stdout(), crossterm::event::EnableMouseCapture);
    }
}

#[derive(Debug, Copy, Clone, Default, PartialEq, Eq)]
pub struct CellContent {
    pub character: char,
    pub width: u8,
    pub style: ContentStyle,
}

#[derive(Debug, Copy, Clone, Default, PartialEq, Eq)]
pub enum Cell {
    #[default]
    Empty,
    Content(CellContent),
}

impl Cell {
    pub fn styled(c: char, s: Style) -> Self {
        Cell::Content(CellContent {
            character: c,
            width: c.width().unwrap_or(1) as u8,
            style: s.into(),
        })
    }

    pub fn new(c: char) -> Self {
        Cell::styled(c, Style::default())
    }

    pub fn content(&self) -> Option<&CellContent> {
        match self {
            Cell::Content(c) => Some(c),
            _ => None,
        }
    }

    pub fn content_mut(&mut self) -> Option<&mut CellContent> {
        match self {
            Cell::Content(c) => Some(c),
            _ => None,
        }
    }
}

pub trait Frame: IndexMut<Dims, Output = Cell> {
    fn size(&self) -> Dims;

    // These 2 methods are used as a source of truth for the frame. Ideally, all other methods
    // should use these methods to access the cells.

    fn try_ref(&self, pos: Dims) -> Option<&Cell>;

    fn try_ref_mut(&mut self, pos: Dims) -> Option<&mut Cell>;

    fn try_set(&mut self, pos: Dims, to: Cell) -> bool {
        match self.try_ref_mut(pos) {
            Some(cell) => *cell = to,
            None => return false,
        }

        true
    }

    fn put_char(&mut self, pos @ Dims(x, y): Dims, character: char, style: Style) -> usize {
        let width = character.width().unwrap_or(1) as i32;
        if x < 0 || self.size().0 <= x || y < 0 || self.size().1 <= y {
            return width as usize;
        }

        if width == 0 {
            return 0;
        }

        let cell = Cell::styled(character, style);

        self[pos] = cell;

        for x in x + 1..x + width {
            self[Dims(x, y)] = Cell::Empty;
        }

        width as usize
    }

    fn fill(&mut self, cell: Cell) {
        for y in 0..self.size().1 {
            for x in 0..self.size().0 {
                self[Dims(x, y)] = cell;
            }
        }
    }

    fn fill_rect(&mut self, pos: Dims, size: Dims, cell: Cell) {
        for y in pos.1..pos.1 + size.1 {
            for x in pos.0..pos.0 + size.0 {
                if x < 0 || x >= self.size().0 || y < 0 || y >= self.size().1 {
                    continue;
                }
                self[Dims(x, y)] = cell;
            }
        }
    }

    fn clear(&mut self) {
        self.fill(Cell::new(' '));
    }

    fn imview(&self) -> FrameView<'_>;

    fn view(&mut self) -> FrameViewMut<'_>;
}

// These methods are used to create views of the frame, allowing for more complex layouts.

impl Drawable for FrameViewMut<'_> {
    fn draw(&self, pos: Dims, frame: &mut dyn Frame, _: ()) {
        for y in 0..self.size().1 {
            for x in 0..self.size().0 {
                frame.try_set(Dims(pos.0 + x, pos.1 + y), self[Dims(x, y)]);
            }
        }
    }
}

pub struct FrameBuffer {
    buffer: Vec<Vec<Cell>>,
    size: Dims,
}

impl FrameBuffer {
    pub fn new(size: Dims) -> Self {
        assert!(size.0 > 0 && size.1 > 0);
        let mut buffer = Vec::with_capacity(size.1 as usize);
        for _ in 0..size.1 {
            buffer.push(vec![Cell::new(' '); size.0 as usize]);
        }
        FrameBuffer { buffer, size }
    }

    pub fn resize(&mut self, size: Dims) {
        if self.size == size {
            return;
        }

        self.size = size;
        self.buffer.resize(size.1 as usize, Vec::new());
        for row in self.buffer.iter_mut() {
            row.resize(size.0 as usize, Cell::new(' '));
        }
    }

    fn check_pos(&self, pos: Dims) -> Option<()> {
        if (pos.0 < 0 || pos.0 >= self.size.0) || (pos.1 < 0 || pos.1 >= self.size.1) {
            return None;
        }
        Some(())
    }
}

impl Frame for FrameBuffer {
    fn size(&self) -> Dims {
        self.size
    }

    fn try_ref(&self, pos: Dims) -> Option<&Cell> {
        self.check_pos(pos)?;
        Some(&self.buffer[pos.1 as usize][pos.0 as usize])
    }

    fn try_ref_mut(&mut self, pos: Dims) -> Option<&mut Cell> {
        self.check_pos(pos)?;
        Some(&mut self.buffer[pos.1 as usize][pos.0 as usize])
    }

    fn imview<'a>(&'a self) -> FrameView<'a> {
        FrameView {
            bounds: Rect::sized_at(Dims(0, 0), self.size()),
            frame: self,
        }
    }
    fn view<'a>(&'a mut self) -> FrameViewMut<'a> {
        FrameViewMut {
            bounds: Rect::sized_at(Dims(0, 0), self.size()),
            frame: self,
        }
    }
}

impl std::ops::Index<Dims> for FrameBuffer {
    type Output = Cell;

    fn index(&self, pos: Dims) -> &Self::Output {
        self.try_ref(pos).expect("Position out of bounds")
    }
}

impl std::ops::IndexMut<Dims> for FrameBuffer {
    fn index_mut(&mut self, pos: Dims) -> &mut Self::Output {
        self.try_ref_mut(pos).expect("Position out of bounds")
    }
}

impl std::ops::Index<i32> for FrameBuffer {
    type Output = [Cell];

    fn index(&self, index: i32) -> &Self::Output {
        &self.buffer[index as usize]
    }
}

#[derive(Clone, Copy)]
pub struct FrameView<'a> {
    frame: &'a dyn Frame,
    bounds: Rect,
}

impl FrameView<'_> {
    pub fn size(&self) -> Dims {
        self.bounds.size()
    }
}

impl std::ops::Index<Dims> for FrameView<'_> {
    type Output = Cell;

    fn index(&self, pos: Dims) -> &Self::Output {
        self.frame.try_ref(pos + self.bounds.start).unwrap()
    }
}

impl Drawable for FrameView<'_> {
    fn draw(&self, pos: Dims, frame: &mut dyn Frame, _: ()) {
        for y in 0..self.size().1 {
            for x in 0..self.size().0 {
                frame.try_set(Dims(pos.0 + x, pos.1 + y), self[Dims(x, y)]);
            }
        }
    }
}

pub struct FrameViewMut<'a> {
    frame: &'a mut dyn Frame,
    bounds: Rect,
    // TODO: clip: bool,
}

impl FrameViewMut<'_> {
    pub fn draw<S>(&mut self, pos: Dims, content: impl Drawable<S>, styles: S) {
        content.draw(pos, self, styles);
    }

    pub fn draw_aligned<S>(&mut self, align: Align, content: impl SizedDrawable<S>, styles: S)
    where
        Self: Sized,
    {
        content.draw_aligned(align, self, styles);
    }
    #[inline]
    pub fn bounds(
        &mut self,
        bounds: Rect,
        content: impl FnOnce(&mut FrameViewMut<'_>),
    ) -> &mut Self {
        content(&mut FrameViewMut {
            frame: self,
            bounds,
        });
        self
    }

    #[inline]
    pub fn centered(
        &mut self,
        size: Dims,
        content: impl FnOnce(&mut FrameViewMut<'_>),
    ) -> &mut Self {
        let start_x = (self.size().0 - size.0) / 2;
        let start_y = (self.size().1 - size.1) / 2;
        content(&mut FrameViewMut {
            frame: self,
            bounds: Rect::sized_at(Dims(start_x, start_y), size),
        });
        self
    }

    #[inline]
    pub fn top(&mut self, len: i32, content: impl FnOnce(&mut FrameViewMut)) -> &mut Self {
        let size = Dims(self.size().0, len);
        content(&mut FrameViewMut {
            frame: self,
            bounds: Rect::sized_at(Dims(0, 0), size),
        });
        self
    }

    #[inline]
    pub fn bottom(&mut self, len: i32, content: impl FnOnce(&mut FrameViewMut)) -> &mut Self {
        let start_y = self.size().1 - len;
        let size = Dims(self.size().0, len);
        content(&mut FrameViewMut {
            frame: self,
            bounds: Rect::sized_at(Dims(0, start_y), size),
        });
        self
    }

    #[inline]
    pub fn left(&mut self, len: i32, content: impl FnOnce(&mut FrameViewMut)) -> &mut Self {
        let size = Dims(len, self.size().1);
        content(&mut FrameViewMut {
            frame: self,
            bounds: Rect::sized_at(Dims(0, 0), size),
        });
        self
    }

    #[inline]
    pub fn right(&mut self, len: i32, content: impl FnOnce(&mut FrameViewMut)) -> &mut Self {
        let start_x = self.size().0 - len;
        let size = Dims(len, self.size().1);
        content(&mut FrameViewMut {
            frame: self,
            bounds: Rect::sized_at(Dims(start_x, 0), size),
        });
        self
    }

    #[inline]
    pub fn off_top(&mut self, by: i32, content: impl FnOnce(&mut FrameViewMut)) -> &mut Self {
        let start_y = by;
        let size = Dims(self.size().0, self.size().1 - by);
        content(&mut FrameViewMut {
            frame: self,
            bounds: Rect::sized_at(Dims(0, start_y), size),
        });
        self
    }

    #[inline]
    pub fn off_bottom(&mut self, by: i32, content: impl FnOnce(&mut FrameViewMut)) -> &mut Self {
        let start_y = self.size().1 - by;
        let size = Dims(self.size().0, self.size().1 - by);
        content(&mut FrameViewMut {
            frame: self,
            bounds: Rect::sized_at(Dims(0, start_y), size),
        });
        self
    }

    #[inline]
    pub fn off_left(&mut self, by: i32, content: impl FnOnce(&mut FrameViewMut)) -> &mut Self {
        let start_x = by;
        let size = Dims(self.size().0 - by, self.size().1);
        content(&mut FrameViewMut {
            frame: self,
            bounds: Rect::sized_at(Dims(start_x, 0), size),
        });
        self
    }

    #[inline]
    pub fn off_right(&mut self, by: i32, content: impl FnOnce(&mut FrameViewMut)) -> &mut Self {
        let start_x = self.size().0 - by;
        let size = Dims(self.size().0 - by, self.size().1);
        content(&mut FrameViewMut {
            frame: self,
            bounds: Rect::sized_at(Dims(start_x, 0), size),
        });
        self
    }

    #[inline]
    pub fn pad(&mut self, padding: Padding, content: impl FnOnce(&mut FrameViewMut)) -> &mut Self {
        let size = self.size();
        let start_x = padding.left;
        let start_y = padding.top;
        let inner_size = Dims(
            size.0 - padding.left - padding.right,
            size.1 - padding.top - padding.bottom,
        );

        content(&mut FrameViewMut {
            frame: self,
            bounds: Rect::sized_at(Dims(start_x, start_y), inner_size),
        });
        self
    }

    #[inline]
    pub fn split<T>(
        &mut self,
        ratio: Offset,
        vertical: bool,
        payload: &mut T, // to allow passing &mut T to the closures, e.g. &mut self
        first: impl FnOnce(&mut FrameViewMut, &mut T),
        second: impl FnOnce(&mut FrameViewMut, &mut T),
    ) -> &mut Self {
        let (f, s) = if vertical {
            Rect::sized(self.size()).split_y(ratio)
        } else {
            Rect::sized(self.size()).split_x(ratio)
        };

        first(
            &mut FrameViewMut {
                frame: self,
                bounds: f,
            },
            payload,
        );
        second(
            &mut FrameViewMut {
                frame: self,
                bounds: s,
            },
            payload,
        );

        self
    }

    #[inline]
    pub fn inside(&mut self, content: impl FnOnce(&mut FrameViewMut)) -> &mut Self {
        self.pad(Padding::from(1), content);
        self
    }

    #[inline]
    pub fn border(&mut self, style: Style) -> &mut Self {
        Rect::sized(self.size()).render(self, style);
        self
    }
}

impl Frame for FrameViewMut<'_> {
    fn size(&self) -> Dims {
        self.bounds.size()
    }

    fn try_ref(&self, pos: Dims) -> Option<&Cell> {
        let pos = pos + self.bounds.start;
        if !self.bounds.contains(pos) {
            return None;
        }
        self.frame.try_ref(pos)
    }

    fn try_ref_mut(&mut self, pos: Dims) -> Option<&mut Cell> {
        let pos = pos + self.bounds.start;
        if !self.bounds.contains(pos) {
            return None;
        }
        self.frame.try_ref_mut(pos)
    }

    fn imview(&self) -> FrameView<'_> {
        FrameView {
            bounds: self.bounds,
            frame: self.frame,
        }
    }

    fn view<'a>(&'a mut self) -> FrameViewMut<'a> {
        FrameViewMut {
            bounds: self.bounds,
            frame: self.frame,
        }
    }
}

impl std::ops::Index<Dims> for FrameViewMut<'_> {
    type Output = Cell;

    fn index(&self, pos: Dims) -> &Self::Output {
        self.try_ref(pos).expect("Position out of bounds")
    }
}

impl std::ops::IndexMut<Dims> for FrameViewMut<'_> {
    fn index_mut(&mut self, pos: Dims) -> &mut Self::Output {
        self.try_ref_mut(pos).expect("Position out of bounds")
    }
}

pub struct Padding {
    pub top: i32,
    pub right: i32,
    pub bottom: i32,
    pub left: i32,
}

impl Padding {
    pub const NONE: Self = Padding {
        top: 0,
        right: 0,
        bottom: 0,
        left: 0,
    };

    pub fn new(top: i32, right: i32, bottom: i32, left: i32) -> Self {
        Padding {
            top,
            right,
            bottom,
            left,
        }
    }

    pub fn hor(padding: i32) -> Self {
        Padding {
            top: 0,
            right: padding,
            bottom: 0,
            left: padding,
        }
    }

    pub fn ver(padding: i32) -> Self {
        Padding {
            top: padding,
            right: 0,
            bottom: padding,
            left: 0,
        }
    }

    pub fn all(padding: i32) -> Self {
        Padding {
            top: padding,
            right: padding,
            bottom: padding,
            left: padding,
        }
    }
}

impl From<i32> for Padding {
    fn from(padding: i32) -> Self {
        Padding::all(padding)
    }
}

impl From<(i32, i32)> for Padding {
    fn from((ver, hor): (i32, i32)) -> Self {
        Padding {
            top: ver,
            right: hor,
            bottom: ver,
            left: hor,
        }
    }
}

impl From<(i32, i32, i32, i32)> for Padding {
    fn from((top, right, bottom, left): (i32, i32, i32, i32)) -> Self {
        Padding {
            top,
            right,
            bottom,
            left,
        }
    }
}

impl From<Dims> for Padding {
    fn from(dims: Dims) -> Self {
        Padding {
            top: dims.1,
            right: dims.0,
            bottom: dims.1,
            left: dims.0,
        }
    }
}
