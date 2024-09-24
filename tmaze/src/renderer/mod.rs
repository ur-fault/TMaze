pub mod drawable;
pub mod helpers;

use std::{
    io::{self, stdout, Write},
    panic, thread,
};

use cmaze::dims::Dims;
use crossterm::{event::Event, execute, style::ContentStyle, terminal, QueueableCommand};
use unicode_width::UnicodeWidthChar;

use crate::settings::theme::Style;

use self::{drawable::Drawable, helpers::term_size};

pub struct Renderer {
    size: Dims,
    shown: Frame,
    hidden: Frame,
    full_redraw: bool,
}

impl Renderer {
    pub fn new() -> io::Result<Self> {
        let (w, h) = term_size();
        let size = Dims(w as i32, h as i32);
        let hidden = Frame::new(size);
        let shown = Frame::new(size);

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

    pub fn frame(&mut self) -> &mut Frame {
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

pub struct Frame {
    pub(crate) buffer: Vec<Vec<Cell>>,
    pub(crate) size: Dims,
}

impl Frame {
    pub fn new(size: Dims) -> Self {
        let mut buffer = Vec::new();
        for _ in 0..size.1 {
            buffer.push(vec![Cell::new(' '); size.0 as usize]);
        }
        Frame { buffer, size }
    }

    pub fn put_char_styled(&mut self, Dims(x, y): Dims, character: char, style: Style) {
        if x < 0 || self.size.0 <= x || y < 0 || self.size.1 <= y {
            return;
        }

        let width = character.width().unwrap_or(1) as i32;
        if width == 0 {
            return;
        }

        let cell = Cell::styled(character, style);

        self.buffer[y as usize][x as usize] = cell;

        for i in x + 1..x + width {
            self.buffer[y as usize][i as usize] = Cell::Empty;
        }
    }

    pub fn put_char(&mut self, pos: Dims, character: char) {
        self.put_char_styled(pos, character, Style::default());
    }

    pub fn try_set(&mut self, pos: Dims, cell: Cell) -> bool {
        if (pos.0 < 0 || pos.0 >= self.size.0) || (pos.1 < 0 || pos.1 >= self.size.1) {
            return false;
        }

        self.buffer[pos.1 as usize][pos.0 as usize] = cell;
        true
    }

    pub fn set(&mut self, pos: Dims, cell: Cell) {
        self.buffer[pos.1 as usize][pos.0 as usize] = cell;
    }

    pub fn draw(&mut self, pos: Dims, content: impl Drawable) {
        content.draw(pos, self);
    }

    pub fn draw_styled(&mut self, pos: Dims, content: impl Drawable, style: Style) {
        content.draw_with_style(pos, self, style);
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

    pub fn clear(&mut self) {
        for row in self.buffer.iter_mut() {
            for cell in row.iter_mut() {
                *cell = Cell::new(' ');
            }
        }
    }

    pub fn fill(&mut self, cell: Cell) {
        for row in self.buffer.iter_mut() {
            for c in row.iter_mut() {
                *c = cell;
            }
        }
    }

    pub fn fill_rect(&mut self, pos: Dims, size: Dims, cell: Cell) {
        for y in pos.1..pos.1 + size.1 {
            for x in pos.0..pos.0 + size.0 {
                if x < 0 || x >= self.size.0 || y < 0 || y >= self.size.1 {
                    continue;
                }
                self.buffer[y as usize][x as usize] = cell;
            }
        }
    }
}

impl std::ops::Index<Dims> for Frame {
    type Output = Cell;

    fn index(&self, index: Dims) -> &Self::Output {
        &self.buffer[index.1 as usize][index.0 as usize]
    }
}

impl std::ops::IndexMut<Dims> for Frame {
    fn index_mut(&mut self, index: Dims) -> &mut Self::Output {
        &mut self.buffer[index.1 as usize][index.0 as usize]
    }
}

impl std::ops::Index<i32> for Frame {
    type Output = [Cell];

    fn index(&self, index: i32) -> &Self::Output {
        &self.buffer[index as usize]
    }
}

impl Drawable for &Frame {
    fn draw(&self, pos: Dims, frame: &mut Frame) {
        for y in 0..self.size.1 {
            for x in 0..self.size.0 {
                frame.try_set(Dims(pos.0 + x, pos.1 + y), self[Dims(x, y)]);
            }
        }
    }

    fn draw_with_style(&self, pos: Dims, frame: &mut Frame, _: Style) {
        // we ignore the style
        self.draw(pos, frame);
    }
}
