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

pub fn double_line_corner(left: bool, top: bool, right: bool, bottom: bool) -> &'static str {
    match (left, top, right, bottom) {
        (false, false, false, false) => "▪",
        (false, false, false, true) => "▪",
        (false, false, true, false) => "▪",
        (false, false, true, true) => "╔",
        (false, true, false, false) => "▪",
        (false, true, false, true) => "║",
        (false, true, true, false) => "╚",
        (false, true, true, true) => "╠",
        (true, false, false, false) => "▪",
        (true, false, false, true) => "╗",
        (true, false, true, false) => "═",
        (true, false, true, true) => "╦",
        (true, true, false, false) => "╝",
        (true, true, false, true) => "╣",
        (true, true, true, false) => "╩",
        (true, true, true, true) => "╬",
    }
}

#[allow(dead_code)]
pub fn round_line_corner(left: bool, top: bool, right: bool, bottom: bool) -> &'static str {
    match (left, top, right, bottom) {
        (false, false, false, false) => "#",
        (false, false, false, true) => "#",
        (false, false, true, false) => "#",
        (false, false, true, true) => "╭",
        (false, true, false, false) => "#",
        (false, true, false, true) => "│",
        (false, true, true, false) => "╰",
        (false, true, true, true) => "├",
        (true, false, false, false) => "#",
        (true, false, false, true) => "╮",
        (true, false, true, false) => "─",
        (true, false, true, true) => "┬",
        (true, true, false, false) => "╯",
        (true, true, false, true) => "┤",
        (true, true, true, false) => "┴",
        (true, true, true, true) => "┼",
    }
}

pub fn from_maze_to_real(pos_on_maze: Dims3D) -> Dims {
    Dims(pos_on_maze.0 * 2 + 1, pos_on_maze.1 * 2 + 1)
}
