pub mod constants;

use core::fmt;
use std::ops::Deref;

use crossterm::event::KeyEventKind;

use crate::core::*;
use crate::gameboard::Maze;

pub const fn line_center(container_start: i32, container_end: i32, item_width: i32) -> i32 {
    (container_end - container_start - item_width) / 2 + container_start
}

pub const fn box_center(container_start: Dims, container_end: Dims, box_dims: Dims) -> Dims {
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
    pub const fn from_bools(left: bool, top: bool, right: bool, bottom: bool) -> Self {
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

    pub const fn double(&self) -> char {
        match self {
            Self::Empty => ' ',
            Self::Cross => '╬',
            Self::Horizontal => '═',
            Self::Vertical => '║',
            Self::OpenTop | Self::OpenBottom | Self::OpenLeft | Self::OpenRight => '▪',
            Self::ClosedTop => '╦',
            Self::ClosedBottom => '╩',
            Self::ClosedLeft => '╠',
            Self::ClosedRight => '╣',
            Self::TopLeft => '╝',
            Self::TopRight => '╚',
            Self::BottomLeft => '╗',
            Self::BottomRight => '╔',
        }
    }

    pub const fn round(&self) -> char {
        match self {
            Self::Empty => ' ',
            Self::Cross => '┼',
            Self::Horizontal => '─',
            Self::Vertical => '│',
            Self::OpenTop => '╷',
            Self::OpenBottom => '╵',
            Self::OpenLeft => '╴',
            Self::OpenRight => '╶',
            Self::ClosedLeft => '├',
            Self::ClosedTop => '┬',
            Self::ClosedRight => '┤',
            Self::ClosedBottom => '┴',
            Self::TopLeft => '╯',
            Self::TopRight => '╰',
            Self::BottomLeft => '╮',
            Self::BottomRight => '╭',
        }
    }
}

pub fn maze2screen_3d(pos_on_maze: impl Into<Dims3D>) -> Dims3D {
    let pos_on_maze = pos_on_maze.into();
    Dims3D(pos_on_maze.0 * 2 + 1, pos_on_maze.1 * 2 + 1, pos_on_maze.2)
}

pub fn maze2screen(pos_on_maze: impl Into<Dims3D>) -> Dims {
    let pos_on_maze = pos_on_maze.into();
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

pub enum MbyStaticStr {
    Static(&'static str),
    Owned(String),
}

impl fmt::Display for MbyStaticStr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Static(s) => write!(f, "{}", s),
            Self::Owned(s) => write!(f, "{}", s),
        }
    }
}

impl fmt::Debug for MbyStaticStr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Static(s) => write!(f, "{:?}", s),
            Self::Owned(s) => write!(f, "{:?}", s),
        }
    }
}

impl From<&'static str> for MbyStaticStr {
    fn from(s: &'static str) -> Self {
        Self::Static(s)
    }
}

impl From<String> for MbyStaticStr {
    fn from(s: String) -> Self {
        Self::Owned(s)
    }
}

impl Deref for MbyStaticStr {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        match self {
            Self::Static(s) => s,
            Self::Owned(s) => s,
        }
    }
}

#[macro_export]
macro_rules! lerp {
    (($a:expr) -> ($b:expr) at $t:expr) => {
        $a + ($b - $a) * $t
    };
}

pub const fn yes_no(b: bool, capitalized: bool) -> &'static str {
    match (b, capitalized) {
        (true, true) => "Yes",
        (true, false) => "yes",
        (false, true) => "No",
        (false, false) => "no",
    }
}

pub const fn on_off(val: bool, capitalized: bool) -> &'static str {
    match (val, capitalized) {
        (true, true) => "On",
        (true, false) => "on",
        (false, true) => "Off",
        (false, false) => "off",
    }
}
