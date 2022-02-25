use crate::maze::{CellWall, Maze};
use crossterm::{cursor::MoveTo, execute};
// use std::io::{stdin, stdout}
use masof::Renderer;

fn double_line_corner(left: bool, top: bool, right: bool, bottom: bool) -> &'static str {
    match (left, top, right, bottom) {
        (false, false, false, false) => "#",
        (false, false, false, true) => "#",
        (false, false, true, false) => "#",
        (false, false, true, true) => "╔",
        (false, true, false, false) => "#",
        (false, true, false, true) => "║",
        (false, true, true, false) => "╚",
        (false, true, true, true) => "╠",
        (true, false, false, false) => "#",
        (true, false, false, true) => "╗",
        (true, false, true, false) => "═",
        (true, false, true, true) => "╦",
        (true, true, false, false) => "╝",
        (true, true, false, true) => "╣",
        (true, true, true, false) => "╩",
        (true, true, true, true) => "╬",
    }
}

pub fn ct_print_maze(maze: &Maze, path: &[(usize, usize)], last_: Option<(usize, usize)>) {}

pub fn print_maze(maze: &Maze, path: &[(usize, usize)], last_: Option<(usize, usize)>) {
    if let Some(last) = last_ {}

    print!("╔");
    for x in 0..maze.get_cells()[0].len() - 1 {
        print!(
            "═{}",
            if maze.get_cells()[0][x].get_wall(&CellWall::Right) {
                "╦"
            } else {
                "═"
            }
        )
    }
    print!("═╗\n");
    for (y, line) in maze.get_cells().iter().enumerate() {
        print!("║");
        for x in 0..line.len() {
            print!(
                "{}{}",
                if path[0].0 == x && path[0].1 == y {
                    "O"
                } else {
                    " "
                },
                if line[x].get_wall(&CellWall::Right) {
                    "║"
                } else {
                    " "
                }
            );
        }
        print!(
            "\n{}",
            if y == maze.get_cells().len() - 1 {
                "╚"
            } else if line[0].get_wall(&CellWall::Bottom) {
                "╠"
            } else {
                "║"
            }
        );
        for x in 0..line.len() {
            print!(
                "{}{}",
                if line[x].get_wall(&CellWall::Bottom) {
                    "═"
                } else {
                    " "
                },
                {
                    let cell = &line[x];
                    if maze.is_in_bounds(
                        cell.get_coord().0 as isize + 1,
                        cell.get_coord().1 as isize + 1,
                    ) {
                        let cell2 = &maze.get_cells()[y + 1][x + 1];
                        double_line_corner(
                            cell.get_wall(&CellWall::Bottom),
                            cell.get_wall(&CellWall::Right),
                            cell2.get_wall(&CellWall::Top),
                            cell2.get_wall(&CellWall::Left),
                        )
                    } else {
                        if maze.is_in_bounds(0, y as isize + 1) {
                            double_line_corner(
                                cell.get_wall(&CellWall::Bottom),
                                cell.get_wall(&CellWall::Right),
                                false,
                                true,
                            )
                        } else if maze.is_in_bounds(x as isize + 1, 0) {
                            double_line_corner(
                                cell.get_wall(&CellWall::Bottom),
                                cell.get_wall(&CellWall::Right),
                                true,
                                false,
                            )
                        } else {
                            double_line_corner(
                                cell.get_wall(&CellWall::Bottom),
                                cell.get_wall(&CellWall::Right),
                                false,
                                false,
                            )
                        }
                    }
                }
            );
        }
        print!("\n");
    }
}
