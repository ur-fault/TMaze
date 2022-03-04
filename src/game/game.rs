use std::io::{stdout, Stdout};
use std::time::{Duration, Instant};

use crossterm::{
    event::{poll, read, Event, KeyCode, KeyEvent},
    terminal::size,
};
use masof::{Color, ContentStyle, Renderer};

use crate::maze::algorithms::*;
use crate::maze::{CellWall, Maze};
use crate::tmcore::*;
use crate::{helpers, ui};

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
            match ui::run_menu(
                &mut self.renderer,
                self.style,
                &mut self.stdout,
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
                        ui::run_popup(
                            &mut self.renderer,
                            self.style,
                            &mut self.stdout,
                            "Not implemented yet",
                            &[],
                        )?;
                    }
                    2 => {
                        ui::run_popup(
                            &mut self.renderer,
                            self.style,
                            &mut self.stdout,
                            "Not implemented yet",
                            &[],
                        )?;
                    }
                    3 => {
                        ui::run_popup(
                            &mut self.renderer,
                            self.style,
                            &mut self.stdout,
                            "Not implemented yet",
                            &[],
                        )?;
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
        let mut msize: Dims3D = match ui::run_menu(
            &mut self.renderer,
            self.style,
            &mut self.stdout,
            "Maze size",
            &[
                "10x5", "30x10x3", "5x5x5", "100x30", "300x100", "debug", "xtreme",
            ],
            0,
            false,
        )? {
            0 => (10, 5, 1),
            1 => (30, 10, 3),
            2 => (5, 5, 5),
            3 => (100, 30, 1),
            4 => (300, 100, 1),
            5 => (10, 10, 10),
            6 => (500, 500, 1),
            _ => (0, 0, 0),
        };

        let mut player_pos = (0, 0, 0);
        let goal_pos = (msize.0 - 1, msize.1 - 1, msize.2 - 1);

        let mut player_offset = (0, 0, 0);
        let mut spectator = false;

        let mut maze = {
            let mut last_progress = f64::MIN;
            let generation_func = match ui::run_menu(
                &mut self.renderer,
                self.style,
                &mut self.stdout,
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
                        let res = ui::render_progress(
                            &mut self.renderer,
                            self.style,
                            &mut self.stdout,
                            &format!("Generating maze ({}x{}) {}/{}", msize.0, msize.1, done, all),
                            current_progess,
                        );
                        last_progress = current_progess;

                        // check for quit keys from user
                        if let Ok(true) = poll(Duration::from_nanos(1)) {
                            if let Ok(Event::Key(KeyEvent { code, modifiers: _ })) = read() {
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
                                player_offset = (
                                    player_offset.0,
                                    player_offset.1,
                                    (-player_pos.2).max(player_offset.2 - 1),
                                )
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
                    &ui::format_duration(from_start),
                ),
                &moves,
            )?;

            let play_time = Instant::now() - start_time;

            // check if player won
            if player_pos == goal_pos {
                ui::run_popup(
                    &mut self.renderer,
                    self.style,
                    &mut self.stdout,
                    "You won",
                    &[
                        &format!("Time: {}", ui::format_duration(play_time)),
                        &format!("Moves: {}", move_count),
                    ],
                )?;
                break Ok(());
            }
        }
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
                ui::box_center_screen((real_size.0 as i32, real_size.1 as i32))?
            };

            (pos.0 + player_offset.0 * 2, pos.1 + player_offset.1 * 2)
        };

        let floor = player_pos.2 + player_offset.2;

        self.renderer.begin()?;

        // corners
        if pos.1 > 0 {
            ui::draw_str(
                &mut self.renderer,
                pos.0,
                pos.1,
                &format!(
                    "{}{}",
                    helpers::double_line_corner(false, false, true, true),
                    helpers::double_line_corner(true, false, true, false)
                ),
                self.style,
            );
            ui::draw_str(
                &mut self.renderer,
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
            ui::draw_str(
                &mut self.renderer,
                pos.0,
                pos.1 + real_size.1 - 2,
                &format!("{}", helpers::double_line_corner(false, true, false, true),),
                self.style,
            );
            ui::draw_str(
                &mut self.renderer,
                pos.0 + real_size.0 - 1,
                pos.1 + real_size.1 - 2,
                &format!("{}", helpers::double_line_corner(false, true, false, true),),
                self.style,
            );
        }
        if pos.1 + real_size.1 - 1 < size.1 - 2 {
            ui::draw_str(
                &mut self.renderer,
                pos.0,
                pos.1 + real_size.1 - 1,
                &format!("{}", helpers::double_line_corner(false, true, true, false),),
                self.style,
            );
            ui::draw_str(
                &mut self.renderer,
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
                ui::draw_str(
                    &mut self.renderer,
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
                ui::draw_str(
                    &mut self.renderer,
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

            ui::draw_str(
                &mut self.renderer,
                pos.0,
                ypos,
                &format!("{}", helpers::double_line_corner(false, true, false, true)),
                self.style,
            );

            if ypos + 1 < size.1 {
                ui::draw_str(
                    &mut self.renderer,
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

                ui::draw_str(
                    &mut self.renderer,
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

            ui::draw_str(
                &mut self.renderer,
                pos.0 + real_size.0 - 1,
                ypos,
                &format!("{}", helpers::double_line_corner(false, true, false, true)),
                self.style,
            );
        }

        // Drawing visited places (moves)
        for (move_pos, _) in moves {
            if move_pos.2 == floor {
                let real_pos = helpers::from_maze_to_real(*move_pos);
                ui::draw_char(
                    &mut self.renderer,
                    pos.0 + real_pos.0,
                    pos.1 + real_pos.1,
                    '.',
                    self.style,
                );
            }
        }

        for (iy, row) in maze.get_cells()[floor as usize].iter().enumerate() {
            let ypos = iy as i32 * 2 + 1 + pos.1;
            if ypos >= size.1 - 2 {
                break;
            }

            for (ix, cell) in row.iter().enumerate() {
                let xpos = ix as i32 * 2 + 1 + pos.0;
                if cell.get_wall(CellWall::Right) && ix != maze.size().0 as usize - 1 {
                    ui::draw_str(
                        &mut self.renderer,
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
                    ui::draw_str(
                        &mut self.renderer,
                        xpos,
                        ypos + 1,
                        helpers::double_line_corner(true, false, true, false),
                        self.style,
                    );
                }

                if !cell.get_wall(CellWall::Up) && !cell.get_wall(CellWall::Down) {
                    ui::draw_char(&mut self.renderer, xpos, ypos, 'X', self.style);
                } else if !cell.get_wall(CellWall::Up) {
                    ui::draw_char(&mut self.renderer, xpos, ypos, '/', self.style);
                } else if !cell.get_wall(CellWall::Down) {
                    ui::draw_char(&mut self.renderer, xpos, ypos, '\\', self.style);
                }

                if iy == maze.size().1 as usize - 1 || ix == maze.size().0 as usize - 1 {
                    continue;
                }

                let cell2 = &maze.get_cells()[floor as usize][iy + 1][ix + 1];

                if ypos < size.1 as i32 - 3 {
                    ui::draw_str(
                        &mut self.renderer,
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
            ui::draw_char(
                &mut self.renderer,
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
            ui::draw_char(
                &mut self.renderer,
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

        ui::draw_str(
            &mut self.renderer,
            str_pos_tl.0,
            str_pos_tl.1,
            texts.0,
            self.style,
        );
        ui::draw_str(
            &mut self.renderer,
            str_pos_tr.0,
            str_pos_tr.1,
            texts.1,
            self.style,
        );
        ui::draw_str(
            &mut self.renderer,
            str_pos_bl.0,
            str_pos_bl.1,
            texts.2,
            self.style,
        );
        ui::draw_str(
            &mut self.renderer,
            str_pos_br.0,
            str_pos_br.1,
            texts.3,
            self.style,
        );

        self.renderer.end(&mut self.stdout)?;

        Ok(())
    }
}
