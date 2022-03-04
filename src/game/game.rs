use std::io::{stdout, Stdout};

use crate::maze::{CellWall, Maze};

use crate::maze::algorithms::*;
use crossterm::{
    event::{poll, read, Event, KeyCode, KeyEvent},
    terminal::size,
};
pub use helpers::{Dims, Dims3D, DimsU};
use masof::{Color, ContentStyle, Renderer};
use std::time::{Duration, Instant};
use substring::Substring;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("CrossTerm error; {0}")]
    CrossTermError(#[from] crossterm::ErrorKind),
    #[error("Renderer error; {0}")]
    DrawBufferError(#[from] masof::renderer::Error),
    #[error("Quit")]
    Quit,
    #[error("FullQuit")]
    FullQuit,
    #[error("EmptyMenu")]
    EmptyMenu,
    #[error("InvalidValue")]
    InvalidValue,
}

mod helpers {
    use crate::maze::Maze;
    use std::time::Duration;

    pub type Dims = (i32, i32);
    pub type Dims3D = (i32, i32, i32);
    pub type DimsU = (usize, usize);

    pub fn menu_size(title: &str, options: &[&str], counted: bool) -> Dims {
        match options.iter().map(|opt| opt.len()).max() {
            Some(l) => (
                ((2 + if counted {
                    (options.len() + 1).to_string().len() + 2
                } else {
                    0
                } + l
                    - 2)
                .max(title.len() + 2)
                    + 2) as i32
                    + 2,
                options.len() as i32 + 2 + 2,
            ),
            None => (0, 0),
        }
    }

    pub fn popup_size(title: &str, texts: &[&str]) -> Dims {
        match texts.iter().map(|text| text.len()).max() {
            Some(l) => (
                2 + 2 + l.max(title.len()) as i32,
                2 + 2 + texts.len() as i32,
            ),
            None => (4 + title.len() as i32, 3),
        }
    }

    pub fn format_duration(dur: Duration) -> String {
        format!(
            "{}m{:.1}s",
            dur.as_secs() / 60,
            (dur.as_secs() % 60) as f32 + dur.subsec_millis() as f32 / 1000f32,
        )
    }

    pub fn line_center(container_start: i32, container_end: i32, item_width: i32) -> i32 {
        (container_end - container_start - item_width) / 2 + container_start
    }

    pub fn box_center(container_start: Dims, container_end: Dims, box_dims: Dims) -> Dims {
        (
            line_center(container_start.0, container_end.0, box_dims.0),
            line_center(container_start.1, container_end.1, box_dims.1),
        )
    }

    pub fn maze_render_size(maze: &Maze) -> Dims {
        let msize = maze.size();
        ((msize.0 * 2 + 1) as i32, (msize.1 * 2 + 1) as i32)
    }

    pub fn double_line_corner(left: bool, top: bool, right: bool, bottom: bool) -> &'static str {
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

    pub fn from_maze_to_real(maze_pos: Dims3D) -> Dims {
        (maze_pos.0 * 2 + 1, maze_pos.1 * 2 + 1)
    }
}

pub struct GameSettings {
    slow: bool,
    show_path: bool,
}

pub struct Game {
    player: Vec<Dims>,
    renderer: Renderer,
    stdout: Stdout,
    style: ContentStyle,
}

impl Game {
    pub fn new() -> Self {
        Game {
            player: vec![],
            renderer: Renderer::default(),
            stdout: stdout(),
            style: ContentStyle::default(),
        }
    }

    pub fn run(mut self) -> Result<(), Error> {
        self.renderer.term_on(&mut self.stdout)?;
        loop {
            match self.run_menu(
                "TMaze",
                &["New Game", "Settings", "Controls", "About", "Quit"],
                0,
                true,
            ) {
                Ok(res) => match res {
                    0 => match self.run_game() {
                        Ok(_) | Err(Error::Quit) => {}
                        Err(_) => break,
                    },

                    1 => {
                        self.run_popup("Not implemented yet", &[])?;
                    }
                    2 => {
                        self.run_popup("Not implemented yet", &[])?;
                    }
                    3 => {
                        self.run_popup("Not implemented yet", &[])?;
                    }
                    4 => break,
                    _ => break,
                },
                Err(Error::Quit) => break,
                Err(_) => break,
            };
        }

        self.renderer.term_off(&mut self.stdout)?;
        Ok(())
    }

    fn run_game(&mut self) -> Result<(), Error> {
        let mut msize: Dims3D = match self.run_menu(
            "Maze size",
            &[
                "10x5x1", "30x10x3", "5x5x5", "100x30x1", "debug", "xtreme",
            ],
            0,
            false,
        )? {
            0 => (10, 5, 1),
            1 => (30, 10, 3),
            2 => (5, 5, 5),
            3 => (100, 30, 1),
            4 => (10, 10, 10),
            5 => (500, 500, 1),
            _ => (0, 0, 0),
        };

        let mut player_pos = (0, 0, 0);
        let goal_pos = (msize.0 - 1, msize.1 - 1, msize.2 - 1);

        let mut player_offset = (0, 0, 0);
        let mut spectator = false;

        let mut maze = {
            let mut last_progress = f64::MIN;
            let generation_func = match self.run_menu(
                "Maze generation algorithm",
                &["Depth-first search", "Randomized Kruskal's"],
                0,
                true,
            )? {
                0 => DepthFirstSearch::new,
                1 => RndKruskals::new,
                _ => panic!(),
            };
            generation_func(
                msize,
                Some(|done, all| {
                    let current_progess = done as f64 / all as f64;
                    if current_progess - last_progress > 0.01 {
                        let res = self.render_progress(
                            &format!("Generating maze ({}x{}) {}/{}", msize.0, msize.1, done, all),
                            current_progess,
                        );
                        last_progress = current_progess;

                        // check for quit keys from user
                        if let Ok(true) = poll(Duration::from_nanos(1)) {
                            if let Ok(Event::Key(KeyEvent { code, modifiers })) = read() {
                                match code {
                                    KeyCode::Esc => {
                                        return Err(Error::Quit);
                                    }
                                    KeyCode::Char('q' | 'Q') => {
                                        return Err(Error::FullQuit);
                                    }
                                    _ => {}
                                }
                            }
                        }
                        res
                    } else {
                        Ok(())
                    }
                }),
            )?
        };
        let mut moves = vec![];
        let start_time = Instant::now();
        let mut move_count = 0;

        self.render_game(
            &maze,
            player_pos,
            player_offset,
            goal_pos,
            (
                &format!("{}x{}x{}", player_pos.0, player_pos.1, player_pos.2),
                if spectator { "Spectator" } else { "Adventure" },
                &format!("{} moves", move_count),
                "",
            ),
            &moves,
        )?;

        loop {
            if let Ok(true) = poll(Duration::from_millis(90)) {
                let event = read();

                fn move_player(
                    maze: &Maze,
                    mut pos: Dims3D,
                    wall: CellWall,
                    slow: bool,
                    mut moves: &mut Vec<(Dims3D, CellWall)>,
                ) -> (Dims3D, i32) {
                    if slow {
                        if maze.get_cells()[pos.2 as usize][pos.1 as usize][pos.0 as usize]
                            .get_wall(wall)
                        {
                            (pos, 0)
                        } else {
                            moves.push(((pos.0, pos.1, pos.2), wall));
                            (
                                (
                                    pos.0 + wall.to_coord().0,
                                    pos.1 + wall.to_coord().1,
                                    pos.2 + wall.to_coord().2,
                                ),
                                1,
                            )
                        }
                    } else {
                        let mut count = 0;
                        loop {
                            let mut cell =
                                &maze.get_cells()[pos.2 as usize][pos.1 as usize][pos.0 as usize];
                            if cell.get_wall(wall) {
                                break (pos, count);
                            }
                            count += 1;
                            moves.push(((pos.0, pos.1, pos.2), wall));
                            pos = (
                                pos.0 + wall.to_coord().0,
                                pos.1 + wall.to_coord().1,
                                pos.2 + wall.to_coord().2,
                            );
                            cell =
                                &maze.get_cells()[pos.2 as usize][pos.1 as usize][pos.0 as usize];

                            let perp = wall.perpendicular_walls();
                            if !cell.get_wall(perp.0)
                                || !cell.get_wall(perp.1)
                                || !cell.get_wall(perp.2)
                                || !cell.get_wall(perp.3)
                            {
                                break (pos, count);
                            }
                        }
                    }
                }

                match event {
                    Ok(Event::Key(KeyEvent { code, modifiers })) => match code {
                        KeyCode::Up | KeyCode::Char('w' | 'W') => {
                            if spectator {
                                player_offset =
                                    (player_offset.0, player_offset.1 + 1, player_offset.2)
                            } else {
                                let pmove = move_player(
                                    &maze,
                                    player_pos,
                                    CellWall::Top,
                                    false,
                                    &mut moves,
                                );
                                player_pos = pmove.0;
                                move_count += pmove.1;
                            }
                        }
                        KeyCode::Down | KeyCode::Char('s' | 'S') => {
                            if spectator {
                                player_offset =
                                    (player_offset.0, player_offset.1 - 1, player_offset.2)
                            } else {
                                let pmove = move_player(
                                    &maze,
                                    player_pos,
                                    CellWall::Bottom,
                                    false,
                                    &mut moves,
                                );
                                player_pos = pmove.0;
                                move_count += pmove.1;
                            }
                        }
                        KeyCode::Left | KeyCode::Char('a' | 'A') => {
                            if spectator {
                                player_offset =
                                    (player_offset.0 + 1, player_offset.1, player_offset.2)
                            } else {
                                let pmove = move_player(
                                    &maze,
                                    player_pos,
                                    CellWall::Left,
                                    false,
                                    &mut moves,
                                );
                                player_pos = pmove.0;
                                move_count += pmove.1;
                            }
                        }
                        KeyCode::Right | KeyCode::Char('d' | 'D') => {
                            if spectator {
                                player_offset =
                                    (player_offset.0 - 1, player_offset.1, player_offset.2)
                            } else {
                                let pmove = move_player(
                                    &maze,
                                    player_pos,
                                    CellWall::Right,
                                    false,
                                    &mut moves,
                                );
                                player_pos = pmove.0;
                                move_count += pmove.1;
                            }
                        }
                        KeyCode::Char('q' | 'Q') => {
                            if spectator {
                                player_offset =
                                    (player_offset.0, player_offset.1, (-player_pos.2).max(player_offset.2 - 1))
                            } else {
                                let pmove = move_player(
                                    &maze,
                                    player_pos,
                                    CellWall::Down,
                                    false,
                                    &mut moves,
                                );
                                player_pos = pmove.0;
                                move_count += pmove.1;
                            }
                        }
                        KeyCode::Char('e' | 'E') => {
                            if spectator {
                                player_offset = (
                                    player_offset.0,
                                    player_offset.1,
                                    (maze.size().2 - player_pos.2 - 1).min(player_offset.2 + 1),
                                )
                            } else {
                                let pmove =
                                    move_player(&maze, player_pos, CellWall::Up, false, &mut moves);
                                player_pos = pmove.0;
                                move_count += pmove.1;
                            }
                        }
                        KeyCode::Char(' ') => {
                            if spectator {
                                player_offset = (0, 0, 0);
                                spectator = false
                            } else {
                                spectator = true
                            }
                        }
                        KeyCode::Enter => {}
                        KeyCode::Esc => break Err(Error::Quit),
                        _ => {}
                    },
                    Err(err) => {
                        break Err(Error::CrossTermError(err));
                    }
                    _ => {}
                }

                self.renderer.event(&event.unwrap());
            }

            let from_start = Instant::now() - start_time;
            self.render_game(
                &maze,
                player_pos,
                player_offset,
                goal_pos,
                (
                    &format!("{}x{}x{}", player_pos.0, player_pos.1, player_pos.2),
                    if spectator { "Spectator" } else { "Adventure" },
                    &format!("{} moves", move_count),
                    &helpers::format_duration(from_start),
                ),
                &moves,
            )?;

            let play_time = Instant::now() - start_time;

            // check if player won
            if player_pos == goal_pos {
                self.run_popup(
                    "You won",
                    &[
                        &format!("Time: {}", helpers::format_duration(play_time)),
                        &format!("Moves: {}", move_count),
                    ],
                )?;
                break Ok(());
            }
        }
    }

    fn render_progress(&mut self, title: &str, progress: f64) -> Result<(), Error> {
        let progress_size = (title.len() as i32 + 2, 4);
        let pos = self.box_center(progress_size)?;

        self.renderer.begin()?;

        self.draw_box(pos, progress_size, self.style);
        if pos.1 + 1 >= 0 {
            self.renderer
                .draw_str(pos.0 as u16 + 1, pos.1 as u16 + 1, title, self.style);
        }
        if pos.1 + 2 >= 0 {
            self.draw_str(
                pos.0 + 1,
                pos.1 + 2,
                &"#".repeat((title.len() as f64 * progress) as usize),
                self.style,
            );
        }

        self.renderer.end(&mut self.stdout)?;

        Ok(())
    }

    fn render_game(
        &mut self,
        maze: &Maze,
        player_pos: Dims3D,
        player_offset: Dims3D,
        goal_pos: Dims3D,
        texts: (&str, &str, &str, &str),
        moves: &[(Dims3D, CellWall)],
    ) -> Result<(), Error> {
        let real_size = helpers::maze_render_size(maze);
        let size = {
            let size = size()?;
            (size.0 as i32, size.1 as i32)
        };
        let is_around_player = real_size.0 > size.0 as i32 || real_size.1 + 2 > size.1 as i32;

        let pos = {
            let pos = if is_around_player {
                let player_real_maze_pos = helpers::from_maze_to_real(player_pos);
                (
                    size.0 as i32 / 2 - player_real_maze_pos.0,
                    size.1 as i32 / 2 - player_real_maze_pos.1,
                )
            } else {
                self.box_center((real_size.0 as i32, real_size.1 as i32))?
            };

            (pos.0 + player_offset.0 * 2, pos.1 + player_offset.1 * 2)
        };

        let floor = player_pos.2 + player_offset.2;

        self.renderer.begin()?;

        // corners
        if pos.1 > 0 {
            self.draw_str(
                pos.0,
                pos.1,
                &format!(
                    "{}{}",
                    helpers::double_line_corner(false, false, true, true),
                    helpers::double_line_corner(true, false, true, false)
                ),
                self.style,
            );
            self.draw_str(
                pos.0 + real_size.0 - 2,
                pos.1,
                &format!(
                    "{}{}",
                    helpers::double_line_corner(true, false, true, false),
                    helpers::double_line_corner(true, false, false, true)
                ),
                self.style,
            );
        }
        if pos.1 + real_size.1 - 2 < size.1 - 3 {
            self.draw_str(
                pos.0,
                pos.1 + real_size.1 - 2,
                &format!("{}", helpers::double_line_corner(false, true, false, true),),
                self.style,
            );
            self.draw_str(
                pos.0 + real_size.0 - 1,
                pos.1 + real_size.1 - 2,
                &format!("{}", helpers::double_line_corner(false, true, false, true),),
                self.style,
            );
        }
        if pos.1 + real_size.1 - 1 < size.1 - 2 {
            self.draw_str(
                pos.0,
                pos.1 + real_size.1 - 1,
                &format!("{}", helpers::double_line_corner(false, true, true, false),),
                self.style,
            );
            self.draw_str(
                pos.0 + real_size.0 - 2,
                pos.1 + real_size.1 - 1,
                &format!(
                    "{}{}",
                    helpers::double_line_corner(true, false, true, false),
                    helpers::double_line_corner(true, true, false, false)
                ),
                self.style,
            );
        }
        // horizontal edge lines
        for x in 0..maze.size().0 - 1 {
            if pos.1 > 0 {
                self.draw_str(
                    x as i32 * 2 + pos.0 + 1,
                    pos.1,
                    &format!(
                        "{}{}",
                        helpers::double_line_corner(true, false, true, false),
                        helpers::double_line_corner(
                            true,
                            false,
                            true,
                            maze.get_cells()[floor as usize][0][x as usize]
                                .get_wall(CellWall::Right),
                        )
                    ),
                    self.style,
                );
            }
            if pos.1 + real_size.1 - 1 < size.1 - 2 {
                self.draw_str(
                    x as i32 * 2 + pos.0 + 1,
                    pos.1 + real_size.1 - 1,
                    &format!(
                        "{}{}",
                        helpers::double_line_corner(true, false, true, false),
                        helpers::double_line_corner(
                            true,
                            maze.get_cells()[floor as usize][maze.size().1 as usize - 1]
                                [x as usize]
                                .get_wall(CellWall::Right),
                            true,
                            false,
                        )
                    ),
                    self.style,
                );
            }
        }

        // vertical edge lines
        for y in 0..maze.size().1 - 1 {
            let ypos = y as i32 * 2 + pos.1 + 1;
            if ypos >= size.1 - 2 {
                break;
            }

            self.draw_str(
                pos.0,
                ypos,
                &format!("{}", helpers::double_line_corner(false, true, false, true)),
                self.style,
            );

            if ypos + 1 < size.1 {
                self.draw_str(
                    pos.0,
                    y as i32 * 2 + pos.1 + 2,
                    &format!(
                        "{}",
                        helpers::double_line_corner(
                            false,
                            true,
                            maze.get_cells()[floor as usize][y as usize][0]
                                .get_wall(CellWall::Bottom),
                            true,
                        )
                    ),
                    self.style,
                );

                self.draw_str(
                    pos.0 + real_size.0 - 1,
                    y as i32 * 2 + pos.1 + 2,
                    &format!(
                        "{}",
                        helpers::double_line_corner(
                            maze.get_cells()[floor as usize][y as usize]
                                [maze.size().0 as usize - 1]
                                .get_wall(CellWall::Bottom),
                            true,
                            false,
                            true,
                        )
                    ),
                    self.style,
                );
            }

            self.draw_str(
                pos.0 + real_size.0 - 1,
                ypos,
                &format!("{}", helpers::double_line_corner(false, true, false, true)),
                self.style,
            );
        }

        for (move_pos, wall) in moves {
            let real_pos = helpers::from_maze_to_real(*move_pos);
            self.draw_char(pos.0 + real_pos.0, pos.1 + real_pos.1, '.', self.style)
        }

        for (iy, row) in maze.get_cells()[floor as usize].iter().enumerate() {
            let ypos = iy as i32 * 2 + 1 + pos.1;
            if ypos >= size.1 - 2 {
                break;
            }

            for (ix, cell) in row.iter().enumerate() {
                let xpos = ix as i32 * 2 + 1 + pos.0;
                if cell.get_wall(CellWall::Right) && ix != maze.size().0 as usize - 1 {
                    self.draw_str(
                        xpos + 1,
                        ypos,
                        helpers::double_line_corner(false, true, false, true),
                        self.style,
                    );
                }
                if ypos + 1 < size.1 as i32 - 2
                    && cell.get_wall(CellWall::Bottom)
                    && iy != maze.size().1 as usize - 1
                {
                    self.draw_str(
                        xpos,
                        ypos + 1,
                        helpers::double_line_corner(true, false, true, false),
                        self.style,
                    );
                }

                if !cell.get_wall(CellWall::Up) && !cell.get_wall(CellWall::Down) {
                    self.draw_char(xpos, ypos, 'X', self.style);
                } else if !cell.get_wall(CellWall::Up) {
                    self.draw_char(xpos, ypos, '/', self.style);
                } else if !cell.get_wall(CellWall::Down) {
                    self.draw_char(xpos, ypos, '\\', self.style);
                }

                if iy == maze.size().1 as usize - 1 || ix == maze.size().0 as usize - 1 {
                    continue;
                }

                let cell2 = &maze.get_cells()[floor as usize][iy + 1][ix + 1];

                if ypos < size.1 as i32 - 3 {
                    self.draw_str(
                        ix as i32 * 2 + 2 + pos.0,
                        iy as i32 * 2 + 2 + pos.1,
                        helpers::double_line_corner(
                            cell.get_wall(CellWall::Bottom),
                            cell.get_wall(CellWall::Right),
                            cell2.get_wall(CellWall::Top),
                            cell2.get_wall(CellWall::Left),
                        ),
                        self.style,
                    );
                }
            }
        }

        if floor == goal_pos.2 {
            self.draw_char(
                goal_pos.0 * 2 + 1 + pos.0,
                goal_pos.1 * 2 + 1 + pos.1,
                '$',
                ContentStyle {
                    foreground_color: Some(Color::DarkYellow),
                    background_color: Default::default(),
                    attributes: Default::default(),
                },
            );
        }

        if floor == player_pos.2 {
            self.draw_char(
                player_pos.0 * 2 + 1 + pos.0,
                player_pos.1 * 2 + 1 + pos.1,
                'O',
                ContentStyle {
                    foreground_color: Some(Color::Green),
                    background_color: Default::default(),
                    attributes: Default::default(),
                },
            );
        }

        // Print texts
        let str_pos_tl = if is_around_player {
            (0, 0)
        } else {
            (pos.0, pos.1 - 1)
        };
        let str_pos_tr = if is_around_player {
            (size.0 as i32 - texts.1.len() as i32, 0)
        } else {
            (pos.0 + real_size.0 - texts.1.len() as i32, pos.1 - 1)
        };
        let str_pos_bl = if is_around_player {
            (0, size.1 as i32 - 2)
        } else {
            (pos.0, pos.1 + real_size.1)
        };
        let str_pos_br = if is_around_player {
            (size.0 as i32 - texts.3.len() as i32, size.1 as i32 - 2)
        } else {
            (
                pos.0 + real_size.0 - texts.3.len() as i32,
                pos.1 + real_size.1,
            )
        };

        self.draw_str(str_pos_tl.0, str_pos_tl.1, texts.0, self.style);
        self.draw_str(str_pos_tr.0, str_pos_tr.1, texts.1, self.style);
        self.draw_str(str_pos_bl.0, str_pos_bl.1, texts.2, self.style);
        self.draw_str(str_pos_br.0, str_pos_br.1, texts.3, self.style);

        self.renderer.end(&mut self.stdout)?;

        Ok(())
    }

    fn run_popup(&mut self, title: &str, texts: &[&str]) -> Result<(), Error> {
        self.render_popup(title, texts)?;

        loop {
            let event = read()?;
            if let Event::Key(KeyEvent { code, modifiers }) = event {
                break Ok(());
            }

            self.renderer.event(&event);

            self.render_popup(title, texts)?;
        }
    }

    fn render_popup(&mut self, title: &str, texts: &[&str]) -> Result<(), Error> {
        self.renderer.begin()?;

        let box_size = helpers::popup_size(title, texts);
        let title_pos = self.box_center((title.len() as i32 + 2, 1))?.0;
        let pos = self.box_center(box_size)?;

        self.draw_box(pos, box_size, self.style);
        self.draw_str(title_pos, pos.1 + 1, &format!(" {} ", title), self.style);

        if texts.len() != 0 {
            self.draw_str(
                pos.0 + 1,
                pos.1 + 2,
                &"─".repeat(box_size.0 as usize - 2),
                self.style,
            );
            for (i, text) in texts.iter().enumerate() {
                self.draw_str(pos.0 + 2, pos.1 + 3 + i as i32, text, self.style);
            }
        }

        self.renderer.end(&mut self.stdout)?;

        Ok(())
    }

    fn run_menu(
        &mut self,
        title: &str,
        options: &[&str],
        default: usize,
        counted: bool,
    ) -> Result<u16, Error> {
        let mut selected: usize = default;
        let opt_count = options.len();

        if opt_count == 0 {
            return Err(Error::EmptyMenu);
        }

        self.render_menu(title, options, selected, counted)?;

        loop {
            let event = read()?;

            match event {
                Event::Key(KeyEvent { code, modifiers }) => match code {
                    KeyCode::Up | KeyCode::Char('w') | KeyCode::Char('W') => {
                        selected = if selected == 0 {
                            opt_count - 1
                        } else {
                            selected - 1
                        }
                    }
                    KeyCode::Down | KeyCode::Char('s') | KeyCode::Char('S') => {
                        selected = (selected + 1) % opt_count
                    }
                    KeyCode::Enter | KeyCode::Char(' ') => return Ok(selected as u16),
                    KeyCode::Char(ch) => match ch {
                        'q' | 'Q' => return Err(Error::FullQuit),
                        '1' if counted && 1 <= opt_count => selected = 1 - 1,
                        '2' if counted && 2 <= opt_count => selected = 2 - 1,
                        '3' if counted && 3 <= opt_count => selected = 3 - 1,
                        '4' if counted && 4 <= opt_count => selected = 4 - 1,
                        '5' if counted && 5 <= opt_count => selected = 5 - 1,
                        '6' if counted && 6 <= opt_count => selected = 6 - 1,
                        '7' if counted && 7 <= opt_count => selected = 7 - 1,
                        '8' if counted && 8 <= opt_count => selected = 8 - 1,
                        '9' if counted && 9 <= opt_count => selected = 9 - 1,
                        _ => {}
                    },
                    KeyCode::Esc => return Err(Error::Quit),
                    _ => {}
                },
                Event::Mouse(_) => {}
                _ => {}
            }

            self.renderer.event(&event);

            self.render_menu(title, options, selected, counted)?;
        }
    }

    fn render_menu(
        &mut self,
        title: &str,
        options: &[&str],
        selected: usize,
        counted: bool,
    ) -> Result<(), Error> {
        let menu_size = helpers::menu_size(title, options, counted);
        let pos = self.box_center(menu_size)?;
        let opt_count = options.len();

        let max_count = opt_count.to_string().len();

        self.renderer.begin()?;

        self.draw_box(pos, menu_size, self.style);

        self.draw_str(pos.0 + 2 + 1, pos.1 + 1, &format!("{}", &title), self.style);
        self.draw_str(
            pos.0 + 1,
            pos.1 + 1 + 1,
            &"─".repeat(menu_size.0 as usize - 2),
            self.style,
        );

        for (i, option) in options.iter().enumerate() {
            let style = if i == selected {
                ContentStyle {
                    background_color: Some(Color::White),
                    foreground_color: Some(Color::Black),
                    attributes: Default::default(),
                }
            } else {
                ContentStyle::default()
            };

            let off_x = if counted {
                i.to_string().len() as u16 + 2
            } else {
                0
            };

            self.draw_str(
                pos.0 + 1,
                i as i32 + pos.1 + 2 + 1,
                &format!(
                    "{} {}{}",
                    if i == selected { ">" } else { " " },
                    if counted {
                        format!(
                            "{}. {}",
                            i + 1,
                            " ".repeat(max_count - (i + 1).to_string().len())
                        )
                    } else {
                        String::from("")
                    },
                    option
                ),
                style,
            );
        }
        self.renderer.end(&mut self.stdout)?;

        Ok(())
    }

    // Helpers

    fn box_center(&self, box_dims: Dims) -> Result<Dims, Error> {
        let size_u16 = size()?;
        Ok(helpers::box_center(
            (0, 0),
            (size_u16.0 as i32, size_u16.1 as i32),
            box_dims,
        ))
    }

    fn draw_box(&mut self, pos: Dims, size: Dims, style: ContentStyle) {
        self.draw_str(
            pos.0,
            pos.1,
            &format!("╭{}╮", "─".repeat(size.0 as usize - 2)),
            style,
        );

        for y in pos.1 + 1..pos.1 + size.1 - 1 {
            self.draw_char(pos.0, y, '│', style);
            self.draw_char(pos.0 + size.0 - 1, y, '│', style);
        }

        self.draw_str(
            pos.0,
            pos.1 + size.1 - 1,
            &format!("╰{}╯", "─".repeat(size.0 as usize - 2)),
            style,
        );
    }

    fn draw_str(&mut self, mut x: i32, y: i32, mut text: &str, style: ContentStyle) {
        if y < 0 {
            return;
        }

        if x < 0 && text.len() as i32 > -x + 1 {
            text = text.substring(-x as usize, text.len() - 1);
            x = 0;
        }

        if x > u16::MAX as i32 || y > u16::MAX as i32 {
            return;
        }

        self.renderer.draw_str(x as u16, y as u16, text, style);
    }

    fn draw_char(&mut self, mut x: i32, y: i32, mut text: char, style: ContentStyle) {
        if y < 0 || x < 0 || x > u16::MAX as i32 || y > u16::MAX as i32 {
            return;
        }

        self.renderer.draw_char(x as u16, y as u16, text, style);
    }
}
