pub mod constants;

use core::fmt;

use crossterm::event::KeyEventKind;
use crossterm::style::{ContentStyle, Color};
use fyodor::ui::menu::Menu;

use crate::core::*;
use crate::gameboard::Maze;
use crate::settings::Settings;

pub fn line_center(container_start: i32, container_end: i32, item_width: i32) -> i32 {
    (container_end - container_start - item_width) / 2 + container_start
}

pub fn box_center(container_start: Dims, container_end: Dims, box_dims: Dims) -> Dims {
    Dims(
        line_center(container_start.0, container_end.0, box_dims.0),
        line_center(container_start.1, container_end.1, box_dims.1),
    )
}

pub fn maze_render_size(maze: &Maze) -> Dims {
    let msize = maze.size();
    Dims(msize.0, msize.1) * 2 + Dims(1, 1)
}

pub fn value_if<T: Default>(cond: bool, fun: impl FnOnce() -> T) -> T {
    if cond {
        fun()
    } else {
        T::default()
    }
}

pub fn value_if_else<T>(cond: bool, fun: impl FnOnce() -> T, else_fun: impl FnOnce() -> T) -> T {
    if cond {
        fun()
    } else {
        else_fun()
    }
}

pub enum LineDir {
    Empty,
    Cross,
    Horizontal,
    Vertical,
    OpenLeft,
    OpenTop,
    OpenRight,
    OpenBottom,
    ClosedLeft,
    ClosedTop,
    ClosedRight,
    ClosedBottom,
    TopLeft,
    TopRight,
    BottomLeft,
    BottomRight,
}

impl LineDir {
    pub fn double_line(&self) -> &'static str {
        match self {
            Self::Empty => " ",
            Self::Cross => "╬",
            Self::Horizontal => "═",
            Self::Vertical => "║",
            Self::OpenTop | Self::OpenBottom | Self::OpenLeft | Self::OpenRight => "▪",
            Self::ClosedTop => "╦",
            Self::ClosedBottom => "╩",
            Self::ClosedLeft => "╠",
            Self::ClosedRight => "╣",
            Self::TopLeft => "╝",
            Self::TopRight => "╚",
            Self::BottomLeft => "╗",
            Self::BottomRight => "╔",
        }
    }

    pub fn from_bools(left: bool, top: bool, right: bool, bottom: bool) -> Self {
        match (left, top, right, bottom) {
            (false, false, false, false) => Self::Empty,
            (true, true, true, true) => Self::Cross,
            (true, false, true, false) => Self::Horizontal,
            (false, true, false, true) => Self::Vertical,
            (false, true, false, false) => Self::OpenTop,
            (false, false, false, true) => Self::OpenBottom,
            (true, false, false, false) => Self::OpenLeft,
            (false, false, true, false) => Self::OpenRight,
            (true, false, true, true) => Self::ClosedTop,
            (true, true, true, false) => Self::ClosedBottom,
            (false, true, true, true) => Self::ClosedLeft,
            (true, true, false, true) => Self::ClosedRight,
            (true, true, false, false) => Self::TopLeft,
            (false, true, true, false) => Self::TopRight,
            (true, false, false, true) => Self::BottomLeft,
            (false, false, true, true) => Self::BottomRight,
        }
    }

    #[allow(dead_code)]
    pub fn single_round_line(&self) -> &'static str {
        match self {
            Self::Empty => " ",
            Self::Cross => "┼",
            Self::Horizontal => "─",
            Self::Vertical => "│",
            Self::OpenTop | Self::OpenBottom | Self::OpenLeft | Self::OpenRight => "#",
            Self::ClosedLeft => "├",
            Self::ClosedTop => "┬",
            Self::ClosedRight => "┤",
            Self::ClosedBottom => "┴",
            Self::TopLeft => "╯",
            Self::TopRight => "╰",
            Self::BottomLeft => "╮",
            Self::BottomRight => "╭",
        }
    }
}

pub fn maze_pos_to_real(pos_on_maze: Dims3D) -> Dims {
    Dims(pos_on_maze.0 * 2 + 1, pos_on_maze.1 * 2 + 1)
}

pub fn is_release(k: KeyEventKind) -> bool {
    k == KeyEventKind::Release
}

pub trait ToDebug: fmt::Debug {
    fn to_debug(&self) -> String {
        format!("{:?}", self)
    }
}

impl<T: fmt::Debug> ToDebug for T {}

pub fn dims2fyodor(dims: Dims) -> fyodor::Dims {
    fyodor::Dims::new(dims.0, dims.1)
}

pub fn fyodor2dims(fyodor: fyodor::Dims) -> Dims {
    Dims(fyodor.x, fyodor.y)
}

pub fn fg_style(color: Color) -> ContentStyle {
    ContentStyle {
        foreground_color: Some(color),
        ..Default::default()
    }
}

pub fn make_menu<T>(title: impl Into<String>, items: Vec<T>, settings: &Settings) -> Menu<T> {
    let mut menu = Menu::new(title.into()).with_items(items);
    let scheme = settings.get_color_scheme();
    menu.box_style = fg_style(scheme.normal);
    menu.text_style = fg_style(scheme.text);
    menu.item_style = fg_style(scheme.text);

    menu
}
