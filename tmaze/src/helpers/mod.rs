use crate::core::*;
use crate::maze::Maze;

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
    Dims((msize.0 * 2 + 1) as i32, (msize.1 * 2 + 1) as i32)
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

    pub fn double_line_bools(left: bool, top: bool, right: bool, bottom: bool) -> Self {
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

pub fn from_maze_to_real(pos_on_maze: Dims3D) -> Dims {
    Dims(pos_on_maze.0 * 2 + 1, pos_on_maze.1 * 2 + 1)
}
