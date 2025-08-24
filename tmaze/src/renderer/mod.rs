pub mod draw;
pub mod helpers;

use std::{
    io::{self, stdout, Write},
    ops::Index,
    panic, thread,
};

use cmaze::{
    array::Array3D,
    dims::{Dims, Dims3D, Offset},
};
use crossterm::{
    event::Event, execute, style::ContentStyle, terminal, QueueableCommand, SynchronizedUpdate,
};
use draw::{Align, Draw, SizedDrawable};
use unicode_width::UnicodeWidthChar;

use crate::{
    helpers::range_intersection,
    settings::theme::{Style, TerminalColorScheme},
    ui::Rect,
};

use self::helpers::term_size;

pub struct Renderer {
    size: Dims,
    shown: GBuffer,
    hidden: GBuffer,
    full_redraw: bool,
}

impl Renderer {
    pub fn new(scheme: &TerminalColorScheme) -> io::Result<Self> {
        let (w, h) = term_size();
        let size = Dims(w as i32, h as i32);
        let hidden = GBuffer::new(size, scheme);
        let shown = GBuffer::new(size, scheme);

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

    pub fn frame(&mut self) -> &mut GBuffer {
        &mut self.hidden
    }

    pub fn frame_size(&self) -> Dims {
        self.size
    }

    pub fn show(&mut self) -> io::Result<()> {
        let mut tty = stdout();

        tty.sync_update(|tty| {
            use crossterm::style;

            let mut style = ContentStyle::default();
            tty.queue(crossterm::style::ResetColor)?;

            for y in 0..self.size.1 {
                if self.hidden.view()[y] == self.shown.view()[y] && !self.full_redraw {
                    continue;
                }

                tty.queue(crossterm::cursor::MoveTo(0, y as u16))?;

                for x in 0..self.size.0 {
                    if let Cell::Content(c) = &self.hidden.view()[Dims(x, y)] {
                        if style != c.style.into() {
                            let c_style: ContentStyle = c.style.into();
                            if style.background_color != c_style.background_color {
                                tty.queue(style::SetBackgroundColor(
                                    c_style.background_color.unwrap_or(style::Color::Reset),
                                ))?;
                            }
                            if style.foreground_color != c_style.foreground_color {
                                tty.queue(style::SetForegroundColor(
                                    c_style.foreground_color.unwrap_or(style::Color::Reset),
                                ))?;
                            }
                            if style.attributes != c_style.attributes {
                                tty.queue(style::SetAttribute(style::Attribute::Reset))?;
                                if let Some(x) = c_style.foreground_color {
                                    tty.queue(style::SetForegroundColor(x))?;
                                }
                                if let Some(x) = c_style.background_color {
                                    tty.queue(style::SetBackgroundColor(x))?;
                                }
                                tty.queue(style::SetAttributes(c_style.attributes))?;
                            }
                            style = c_style;
                        }
                        tty.queue(style::Print(c.character))?;
                    }
                }
            }

            tty.flush()?;
            self.full_redraw = false;

            std::io::Result::Ok(())
        })??;

        std::mem::swap(&mut self.shown, &mut self.hidden);

        self.hidden.mut_view().clear();

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
    pub style: Style,
}

impl CellContent {
    pub fn styled(c: char, style: Style) -> Self {
        CellContent {
            character: c,
            width: c.width().unwrap_or(1) as u8,
            style,
        }
    }

    pub fn empty() -> Self {
        CellContent {
            character: ' ',
            width: 1,
            style: Default::default(),
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum Cell {
    Placeholder(u8),
    Content(CellContent),
}

impl Cell {
    pub fn styled(c: char, s: Style) -> Self {
        Cell::Content(CellContent::styled(c, s))
    }

    pub fn empty() -> Self {
        Cell::Content(CellContent::empty())
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

    /// Returns `true` if the cell is [`Placeholder`].
    ///
    /// [`Placeholder`]: Cell::Placeholder
    #[must_use]
    pub fn is_placeholder(&self) -> bool {
        matches!(self, Self::Placeholder(..))
    }

    /// Returns `true` if the cell is [`Content`].
    ///
    /// [`Content`]: Cell::Content
    #[must_use]
    pub fn is_content(&self) -> bool {
        matches!(self, Self::Content(..))
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GBuffer(Array3D<Cell>, Box<TerminalColorScheme>);

impl GBuffer {
    pub fn new(size: Dims, scheme: &TerminalColorScheme) -> Self {
        GBuffer(
            Array3D::new(Cell::empty(), size.0 as usize, size.1 as usize, 1),
            Box::new(scheme.clone()),
        )
    }

    pub fn size(&self) -> Dims {
        self.0.size().into()
    }

    pub fn resize(&mut self, new_size: Dims) {
        if self.size() != new_size {
            self.0 = Array3D::new(Cell::empty(), new_size.0 as usize, new_size.1 as usize, 1);
        }
    }

    pub fn contains(&self, pos: Dims) -> bool {
        Rect::sized(self.size()).contains(pos)
    }

    pub fn mut_view(&mut self) -> GMutView<'_> {
        GMutView {
            bounds: Rect::sized_at(Dims::ZERO, self.size()),
            buf: self,
        }
    }

    pub fn view(&self) -> GView<'_> {
        GView {
            buf: self,
            bounds: Rect::sized_at(Dims::ZERO, self.size()),
        }
    }

    pub fn write(&self, to: &mut impl Write) -> io::Result<()> {
        for y in 0..self.size().1 {
            let mut x = 0;
            while x < self.size().0 {
                let Cell::Content(CellContent {
                    character,
                    width,
                    style,
                }) = &self.0[(x, y)]
                else {
                    panic!("Shouldn't encounter a placeholder cell");
                };

                let styled = style.to_cross().apply(*character);
                write!(to, "{styled}")?;
                x += *width as i32;
            }
            writeln!(to)?;
        }
        Ok(())
    }
}

#[derive(Clone, Copy, Debug)]
pub struct GView<'a> {
    buf: &'a GBuffer,
    bounds: Rect,
}

impl GView<'_> {
    pub fn size(&self) -> Dims {
        self.bounds.size()
    }

    pub fn contains(&self, cell_pos: Dims) -> bool {
        Rect::sized(self.size()).contains(cell_pos)
    }
}

impl Draw for GView<'_> {
    fn draw(&self, pos: Dims, frame: &mut GMutView, _: ()) {
        for rel_line in 0..self.size().1 {
            let local_line = rel_line + self.bounds.start.1;
            let mut rel_x = 0;

            while self.buf.0[(rel_x + self.bounds.start.0, local_line)].is_placeholder() {
                if !self.contains(Dims(rel_x + self.bounds.start.0, local_line)) {
                    panic!(
                        "Position out of bounds: ({}, {local_line}) in {:?}",
                        rel_x + self.bounds.start.0,
                        self.bounds
                    );
                }

                rel_x += 1;
            }

            while rel_x + self.bounds.start.0 <= self.bounds.end.0
                && rel_x
                    + self.bounds.start.0
                    + self.buf.0[(rel_x + self.bounds.start.0, local_line)]
                        .content()
                        .unwrap()
                        .width as i32
                    - 1
                    <= self.bounds.end.0
            {
                let CellContent {
                    character,
                    width,
                    style,
                } = *self.buf.0[(rel_x + self.bounds.start.0, local_line)]
                    .content()
                    .unwrap();

                frame.set_content_of(pos + Dims(rel_x, rel_line), character, style);
                rel_x += width as i32;
            }
        }
    }
}

impl SizedDrawable for GView<'_> {
    fn size(&self) -> Dims {
        self.size()
    }
}

impl Index<Dims> for GView<'_> {
    type Output = Cell;

    fn index(&self, pos: Dims) -> &Self::Output {
        #[cfg(debug_assertions)]
        if !self.contains(pos) {
            panic!("Position out of bounds: {pos:?} in {:?}", self.bounds);
        }

        self.buf.0.get(pos).unwrap()
    }
}

impl Index<i32> for GView<'_> {
    type Output = [Cell];

    fn index(&self, index: i32) -> &Self::Output {
        if index < 0 || index >= self.size().1 {
            panic!("Index out of bounds: {index}");
        }
        let line_index = self.buf.0.dim_to_idx(Dims3D(0, index, 0)).unwrap();
        &self.buf.0.to_slice()[line_index..line_index + self.size().0 as usize]
    }
}

pub struct GMutView<'a> {
    buf: &'a mut GBuffer,
    bounds: Rect,
}

impl GMutView<'_> {
    pub fn size(&self) -> Dims {
        self.bounds.size()
    }

    fn clear_space(&mut self, pos: Dims, style: Option<Style>) {
        #[cfg(debug_assertions)]
        if !self.contains(pos) {
            return;
        }

        match self.buf.0.get_mut(self.bounds.start + pos).unwrap() {
            Cell::Content(c) => {
                let style = style.unwrap_or(c.style);
                let cell = Cell::styled(' ', style);
                for x in pos.0..pos.0 + c.width as i32 {
                    self.buf.0[self.bounds.start + Dims(x, pos.1)] = cell;
                }
            }
            Cell::Placeholder(w) => {
                let start = self.bounds.start + pos - Dims(*w as i32, 0);
                let cell_content = self.buf.0[start].content().unwrap();
                let width = cell_content.width as i32;
                let style = style.unwrap_or(cell_content.style);
                let cell = Cell::styled(' ', style);
                for x in start.0..start.0 + width {
                    self.buf.0[(x, start.1)] = cell;
                }
            }
        }
    }

    fn set_content_of(&mut self, pos: Dims, chr: char, style: Style) -> usize {
        let width = chr.width().unwrap_or(1) as i32;

        if !self.contains(pos) || !self.contains(Dims(pos.0 + width - 1, pos.1)) {
            if (0..self.size().1).contains(&pos.1) {
                for x in range_intersection(
                    self.bounds.start.0 + pos.0..self.bounds.start.0 + pos.0 + width,
                    self.bounds.start.0..self.bounds.end.0,
                ) {
                    self.clear_space(Dims(x, pos.1), Some(style));
                }
            }

            return width as usize;
        }

        let prev_style = *self.style_of(pos);

        if chr == ' ' && self.content_of(pos).is_some_and(|c| c.width == 1) {
            if style.alpha == 255 {
                self.buf.0[self.bounds.start + pos] = Cell::styled(' ', style);
            } else {
                let chr = self.content_of(pos).unwrap().character;
                let new_style = style.mix(prev_style, &self.buf.1, true);
                self.buf.0[self.bounds.start + pos] = Cell::styled(chr, new_style);
            }
        } else {
            for x in pos.0..pos.0 + width {
                self.clear_space(Dims(x, pos.1), None);
            }

            let new_style = style.mix(prev_style, &self.buf.1, false);
            self.buf.0[self.bounds.start + pos] = Cell::styled(chr, new_style);
            for x in pos.0 + 1..pos.0 + width {
                self.buf.0[self.bounds.start + Dims(x, pos.1)] =
                    Cell::Placeholder((x - pos.0) as u8);
            }
        }

        width as usize
    }

    fn content_start(&self, pos: Dims) -> Dims {
        #[cfg(debug_assertions)]
        if !self.contains(pos) {
            panic!("Position out of bounds: {pos:?} in {:?}", self.bounds);
        }

        match self.buf.0[self.bounds.start + pos] {
            Cell::Content(_) => pos,
            Cell::Placeholder(by) => pos - Dims(by as i32, 0),
        }
    }

    pub fn content_of(&mut self, pos: Dims) -> Option<&mut CellContent> {
        #[cfg(debug_assertions)]
        if !self.contains(pos) {
            panic!("Position out of bounds: {pos:?} in {:?}", self.bounds);
        }

        match &mut self.buf.0[self.bounds.start + pos] {
            Cell::Content(c) => Some(c),
            Cell::Placeholder(_) => None,
        }
    }

    pub fn style_of(&mut self, pos: Dims) -> &mut Style {
        #[cfg(debug_assertions)]
        if !self.contains(pos) {
            panic!("Position out of bounds: {pos:?} in {:?}", self.bounds);
        }

        let start = self.content_start(pos) + self.bounds.start;
        &mut self.buf.0[start].content_mut().unwrap().style
    }

    pub fn fill(
        &mut self,
        CellContent {
            character,
            style,
            width: _,
        }: CellContent,
    ) -> &mut Self {
        for y in 0..self.size().1 {
            let mut x = 0;
            while x < self.size().0 {
                x += self.set_content_of(Dims(x, y), character, style) as i32;
            }
        }
        self
    }

    pub fn fill_rect(&mut self, rect: Rect, content: CellContent) -> &mut Self {
        self.bounds(rect, |f| {
            f.fill(content);
        });
        self
    }

    pub fn clear(&mut self) {
        self.fill(CellContent::empty());
    }

    #[inline]
    pub fn contains(&self, cell_pos: Dims) -> bool {
        Rect::sized(self.size()).contains(cell_pos)
    }

    pub fn ro(&self) -> GView<'_> {
        GView {
            buf: self.buf,
            bounds: self.bounds,
        }
    }
}

impl GMutView<'_> {
    pub fn draw<S>(&mut self, pos: Dims, content: impl Draw<S>, styles: S) {
        content.draw(pos, self, styles);
    }

    pub fn draw_aligned<S>(&mut self, align: Align, content: impl SizedDrawable<S>, styles: S) {
        content.draw_aligned(align, self, styles);
    }
}

impl GMutView<'_> {
    fn apply_bounds(&mut self, new: Rect) -> Rect {
        assert!(
            new.start.0 >= self.bounds.start.0 && new.start.1 >= self.bounds.start.1,
            "new x: {}, y: {}, old x: {}, y: {}",
            new.start.0,
            new.start.1,
            self.bounds.start.0,
            self.bounds.start.1
        );
        assert!(
            new.end.0 <= self.bounds.end.0 && new.end.1 <= self.bounds.end.1,
            "new x: {}, y: {}, old x: {}, y: {}",
            new.end.0,
            new.end.1,
            self.bounds.end.0,
            self.bounds.end.1
        );
        let old = self.bounds;
        self.bounds = new;
        old
    }

    #[inline]
    fn subview(&mut self, f: impl FnOnce(Rect) -> Rect, content: impl FnOnce(&mut GMutView)) {
        let old = self.apply_bounds(f(self.bounds));
        content(self);
        self.bounds = old;
    }

    #[inline]
    pub fn bounds(&mut self, bounds: Rect, content: impl FnOnce(&mut GMutView)) -> &mut Self {
        self.subview(
            |r| Rect::sized_at(bounds.start + r.start, bounds.size()),
            content,
        );
        self
    }

    #[inline]
    pub fn centered(&mut self, size: Dims, content: impl FnOnce(&mut GMutView)) -> &mut Self {
        self.subview(|r| r.centered(size), content);
        self
    }

    #[inline]
    pub fn top(&mut self, len: i32, content: impl FnOnce(&mut GMutView)) -> &mut Self {
        self.subview(|r| r.split_y(Offset::Abs(len)).0, content);
        self
    }

    #[inline]
    pub fn bottom(&mut self, len: i32, content: impl FnOnce(&mut GMutView)) -> &mut Self {
        self.subview(|r| r.split_y_end(Offset::Abs(len)).1, content);
        self
    }

    #[inline]
    pub fn left(&mut self, len: i32, content: impl FnOnce(&mut GMutView)) -> &mut Self {
        self.subview(|r| r.split_x(Offset::Abs(len)).0, content);
        self
    }

    #[inline]
    pub fn right(&mut self, len: i32, content: impl FnOnce(&mut GMutView)) -> &mut Self {
        self.subview(|r| r.split_x_end(Offset::Abs(len)).1, content);
        self
    }

    #[inline]
    pub fn off_top(&mut self, by: i32, content: impl FnOnce(&mut GMutView)) -> &mut Self {
        self.subview(|r| r.split_y(Offset::Abs(by)).1, content);
        self
    }

    #[inline]
    pub fn off_bottom(&mut self, by: i32, content: impl FnOnce(&mut GMutView)) -> &mut Self {
        self.subview(|r| r.split_y_end(Offset::Abs(by)).0, content);
        self
    }

    #[inline]
    pub fn off_left(&mut self, by: i32, content: impl FnOnce(&mut GMutView)) -> &mut Self {
        self.subview(|r| r.split_x(Offset::Abs(by)).1, content);
        self
    }

    #[inline]
    pub fn off_right(&mut self, by: i32, content: impl FnOnce(&mut GMutView)) -> &mut Self {
        self.subview(|r| r.split_x_end(Offset::Abs(by)).0, content);
        self
    }

    #[inline]
    pub fn pad(&mut self, padding: Padding, content: impl FnOnce(&mut GMutView)) -> &mut Self {
        self.subview(
            |r| Rect {
                start: r.start + Dims(padding.left, padding.top),
                end: r.end - Dims(padding.right, padding.bottom),
            },
            content,
        );
        self
    }

    #[inline]
    pub fn split<T>(
        &mut self,
        ratio: Offset,
        vertical: bool,
        payload: &mut T, // to allow passing &mut T to the closures, e.g. &mut self
        first: impl FnOnce(&mut GMutView, &mut T),
        second: impl FnOnce(&mut GMutView, &mut T),
    ) -> &mut Self {
        self.subview(
            |r| {
                if vertical {
                    r.split_y(ratio).0
                } else {
                    r.split_x(ratio).0
                }
            },
            |f| {
                first(f, payload);
            },
        );
        self.subview(
            |r| {
                if vertical {
                    r.split_y(ratio).1
                } else {
                    r.split_x(ratio).1
                }
            },
            |f| second(f, payload),
        );

        self
    }

    #[inline]
    pub fn inside(&mut self, content: impl FnOnce(&mut GMutView)) -> &mut Self {
        self.pad(Padding::from(1), content);
        self
    }

    #[inline]
    pub fn border(&mut self, style: Style) -> &mut Self {
        Rect::sized(self.size()).render(self, style);
        self
    }
}

#[derive(Clone, Copy, Debug)]
pub struct AlphaView<'a>(pub GView<'a>, pub u8);

impl Draw for AlphaView<'_> {
    fn draw(&self, pos: Dims, frame: &mut GMutView, _: ()) {
        let alpha = self.1;
        for rel_line in 0..self.0.size().1 {
            let local_line = rel_line + self.0.bounds.start.1;
            let mut rel_x = 0;

            while self.0.buf.0[(rel_x + self.0.bounds.start.0, local_line)].is_placeholder() {
                if !self
                    .0
                    .contains(Dims(rel_x + self.0.bounds.start.0, local_line))
                {
                    panic!(
                        "Position out of 0.bounds: ({}, {local_line}) in {:?}",
                        rel_x + self.0.bounds.start.0,
                        self.0.bounds
                    );
                }

                rel_x += 1;
            }

            while rel_x + self.0.bounds.start.0 <= self.0.bounds.end.0
                && rel_x
                    + self.0.bounds.start.0
                    + self.0.buf.0[(rel_x + self.0.bounds.start.0, local_line)]
                        .content()
                        .unwrap()
                        .width as i32
                    - 1
                    <= self.0.bounds.end.0
            {
                let CellContent {
                    character,
                    width,
                    mut style,
                } = *self.0.buf.0[(rel_x + self.0.bounds.start.0, local_line)]
                    .content()
                    .unwrap();

                style.alpha = alpha;

                frame.set_content_of(pos + Dims(rel_x, rel_line), character, style);
                rel_x += width as i32;
            }
        }
    }
}

impl SizedDrawable for AlphaView<'_> {
    fn size(&self) -> Dims {
        self.0.size()
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
