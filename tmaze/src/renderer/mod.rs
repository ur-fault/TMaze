pub mod drawable;
pub mod helpers;

use std::io::{stdout, Write};

use crossterm::{event::Event, style::ContentStyle, QueueableCommand};
use unicode_width::UnicodeWidthChar;

use crate::ui::CRResult;

use self::{drawable::Drawable, helpers::term_size};

pub type Pos = (u16, u16);

pub struct Renderer {
    size: (u16, u16),
    shown: Frame,
    hidden: Frame,
    full_redraw: bool,
}

impl Renderer {
    pub fn new() -> CRResult<Self> {
        let size = term_size();
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

    fn turn_on(&mut self) -> CRResult<()> {
        crossterm::terminal::enable_raw_mode()?;
        crossterm::execute!(
            stdout(),
            crossterm::cursor::Hide,
            crossterm::terminal::EnterAlternateScreen,
        )?;

        self.on_resize(None)?;

        Ok(())
    }

    fn turn_off(&mut self) -> CRResult<()> {
        crossterm::execute!(
            stdout(),
            crossterm::cursor::Show,
            crossterm::terminal::LeaveAlternateScreen,
        )?;
        crossterm::terminal::disable_raw_mode()?;
        Ok(())
    }

    fn on_resize(&mut self, size: Option<Pos>) -> CRResult<()> {
        self.size = size.unwrap_or_else(|| crossterm::terminal::size().unwrap());
        self.shown.resize(self.size);
        self.hidden.resize(self.size);
        self.full_redraw = true;

        Ok(())
    }

    pub fn on_event(&mut self, event: &Event) -> CRResult<()> {
        if let Event::Resize(x, y) = event {
            self.on_resize(Some((*x, *y)))?
        }

        Ok(())
    }

    pub fn frame(&mut self) -> &mut Frame {
        &mut self.hidden
    }

    pub fn render(&mut self) -> CRResult<()> {
        let mut tty = stdout();

        let mut style = ContentStyle::default();
        tty.queue(crossterm::style::ResetColor)?;

        for y in 0..self.size.1 {
            if self.hidden[y] == self.shown[y] && !self.full_redraw {
                continue;
            }

            tty.queue(crossterm::cursor::MoveTo(0, y))?;

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
    pub fn styled(c: char, s: ContentStyle) -> Self {
        Cell::Content(CellContent {
            character: c,
            width: c.width().unwrap_or(1) as u8,
            style: s,
        })
    }

    pub fn new(c: char) -> Self {
        Cell::styled(c, ContentStyle::default())
    }
}

pub struct Frame {
    pub buffer: Vec<Vec<Cell>>,
    pub size: Pos,
}

impl Frame {
    pub fn new(size: Pos) -> Self {
        let mut buffer = Vec::new();
        for _ in 0..size.1 {
            buffer.push(vec![Cell::new(' '); size.0 as usize]);
        }
        Frame { buffer, size }
    }

    pub fn put_char_styled(&mut self, (x, y): Pos, character: char, style: ContentStyle) {
        if x >= self.size.0 || y >= self.size.1 {
            return;
        }

        let width = character.width().unwrap_or(1) as u16;
        if width == 0 {
            return;
        }

        let cell = Cell::styled(character, style);

        self.buffer[y as usize][x as usize] = cell;

        for i in x + 1..x + width {
            self.buffer[y as usize][i as usize] = Cell::Empty;
        }
    }

    pub fn put_char(&mut self, pos: Pos, character: char) {
        self.put_char_styled(pos, character, ContentStyle::default());
    }

    pub fn set(&mut self, pos: Pos, cell: Cell) {
        self.buffer[pos.1 as usize][pos.0 as usize] = cell;
    }

    pub fn draw(&mut self, pos: Pos, content: impl Drawable) {
        content.draw(pos, self);
    }

    pub fn resize(&mut self, size: Pos) {
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
}

impl std::ops::Index<Pos> for Frame {
    type Output = Cell;

    fn index(&self, index: Pos) -> &Self::Output {
        &self.buffer[index.1 as usize][index.0 as usize]
    }
}

impl std::ops::Index<u16> for Frame {
    type Output = [Cell];

    fn index(&self, index: u16) -> &Self::Output {
        &self.buffer[index as usize]
    }
}
