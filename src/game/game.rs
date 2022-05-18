use std::io::{stdout, Stdout};
use std::time::Duration;

use crossterm::{
    event::{poll, read, Event, KeyCode, KeyEvent},
    terminal::size,
};
use masof::{Color, ContentStyle, Renderer};

use crate::maze::{algorithms::*, Cell};
use crate::maze::{CellWall, Maze};
use crate::tmcore::*;
use crate::{helpers, ui};
use pausable_clock::PausableClock;

pub struct Game {
    renderer: Renderer,
    stdout: Stdout,
    style: ContentStyle,
}

impl Game {
    pub fn new() -> Self {
        Game {
            renderer: Renderer::default(),
            stdout: stdout(),
            style: ContentStyle::default(),
        }
    }

    pub fn run(mut self) -> Result<(), Error> {
        self.renderer.term_on(&mut self.stdout)?;
        let mut game_restart_reqested = false;

        loop {
            if game_restart_reqested {
                game_restart_reqested = false;
                match self.run_game() {
                    Ok(_) | Err(Error::Quit) => {}
                    Err(Error::NewGame) => {
                        game_restart_reqested = true;
                    }
                    Err(_) => break,
                }
                continue;
            }

            match ui::menu(
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
                        Err(Error::NewGame) => {
                            game_restart_reqested = true;
                        }
                        Err(_) => break,
                    },

                    1 => {
                        ui::popup(
                            &mut self.renderer,
                            self.style,
                            &mut self.stdout,
                            "Not implemented yet",
                            &[],
                        )?;
                    }
                    2 => {
                        ui::popup(
                            &mut self.renderer,
                            self.style,
                            &mut self.stdout,
                            "Controls",
                            &[
                                "WASD and arrows: move",
                                "Space: spectaror mode",
                                "Q, F and L: move down",
                                "E, R and P: move up",
                                "Escape: pause",
                            ],
                        )?;
                    }
                    3 => {
                        ui::popup(
                            &mut self.renderer,
                            self.style,
                            &mut self.stdout,
                            "About",
                            &[
                                "This is simple maze solving game",
                                "Supported algorithms:",
                                "    - Depth-first search",
                                "    - Kruskal's algorithm",
                                "Supports 3D mazes",
                                "",
                                "Created by:",
                                "    - morsee",
                            ],
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
        let maze_mode: GameMode = *ui::choice_menu(
            &mut self.renderer,
            self.style,
            &mut self.stdout,
            "Maze size",
            &[
                ((10, 5, 1, false), "10x5"),
                ((30, 10, 1, false), "30x10"),
                ((60, 20, 1, false), "60x20"),
                ((5, 5, 5, false), "5x5x5"),
                ((10, 10, 10, false), "10x10x10"),
                ((300, 100, 1, false), "300x100"),
                ((10, 10, 5, true), "10x10x5 Tower"),
                ((40, 15, 10, true), "40x15x10 Tower"),
            ],
            0,
            false,
        )?;
        let msize: Dims3D = (maze_mode.0, maze_mode.1, maze_mode.2);
        let is_tower = maze_mode.3;

        let mut player_pos = (0, 0, 0);
        let goal_pos = (msize.0 - 1, msize.1 - 1, msize.2 - 1);

        let mut player_offset = (0, 0, 0);
        let mut spectator = false;

        let maze = {
            let mut last_progress = f64::MIN;
            let generation_func = match ui::menu(
                &mut self.renderer,
                self.style,
                &mut self.stdout,
                "Maze generation algorithm",
                &["Randomized Kruskal's", "Depth-first search"],
                0,
                true,
            )? {
                0 => RndKruskals::generate,
                1 => DepthFirstSearch::generate,
                _ => panic!(),
            };
            generation_func(
                msize,
                is_tower,
                Some(|done, all| {
                    let current_progess = done as f64 / all as f64;
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
                    if current_progess - last_progress > 0.01 {
                        let res = ui::render_progress(
                            &mut self.renderer,
                            self.style,
                            &mut self.stdout,
                            &format!(
                                "Generating maze ({}x{}x{}) {}/{}",
                                msize.0, msize.1, msize.2, done, all
                            ),
                            current_progess,
                        );
                        last_progress = current_progess;

                        res
                    } else {
                        Ok(())
                    }
                }),
            )?
        };
        let mut moves = vec![];
        let clock = PausableClock::default();
        let start_time = clock.now();
        let mut move_count = 0;

        self.render_game(
            &maze,
            player_pos,
            player_offset,
            goal_pos,
            is_tower,
            (
                &format!(
                    "{}x{}x{}",
                    player_pos.0 + 1,
                    player_pos.1 + 1,
                    player_pos.2 + 1
                ),
                if spectator { "Spectator" } else { "Adventure" },
                &format!("{} moves", move_count),
                "",
            ),
            1,
            &moves,
        )?;

        loop {
            if let Ok(true) = poll(Duration::from_millis(90)) {
                let event = read();

                fn get_new_player_pos(
                    maze: &Maze,
                    mut pos: Dims3D,
                    wall: CellWall,
                    slow: bool,
                    moves: &mut Vec<(Dims3D, CellWall)>,
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

                let mut move_player = |wall: CellWall| {
                    if spectator {
                        player_offset = {
                            let off = match wall {
                                CellWall::Top => (0, 1, 0),
                                CellWall::Bottom => (0, -1, 0),
                                CellWall::Left => (1, 0, 0),
                                CellWall::Right => (-1, 0, 0),
                                CellWall::Up => (0, 0, 1),
                                CellWall::Down => (0, 0, -1),
                            };

                            (
                                player_offset.0 + off.0,
                                player_offset.1 + off.1,
                                (-player_pos.2).max(
                                    (maze.size().2 - player_pos.2 - 1).min(player_offset.2 + off.2),
                                ),
                            )
                        };
                    } else {
                        let pmove = get_new_player_pos(&maze, player_pos, wall, false, &mut moves);
                        player_pos = pmove.0;
                        move_count += pmove.1;

                        if is_tower
                            && !maze.get_cells()[pmove.0 .2 as usize][pmove.0 .1 as usize]
                                [pmove.0 .0 as usize]
                                .get_wall(CellWall::Up)
                        {
                            player_pos.2 += 1;
                            move_count += 1;
                        }
                    }
                };

                match event {
                    Ok(Event::Key(KeyEvent { code, modifiers: _ })) => match code {
                        KeyCode::Up | KeyCode::Char('w' | 'W') => {
                            move_player(CellWall::Top);
                        }
                        KeyCode::Down | KeyCode::Char('s' | 'S') => {
                            move_player(CellWall::Bottom);
                        }
                        KeyCode::Left | KeyCode::Char('a' | 'A') => {
                            move_player(CellWall::Left);
                        }
                        KeyCode::Right | KeyCode::Char('d' | 'D') => {
                            move_player(CellWall::Right);
                        }
                        KeyCode::Char('f' | 'F' | 'q' | 'Q' | 'l' | 'L') => {
                            move_player(CellWall::Down);
                        }
                        KeyCode::Char('r' | 'R' | 'e' | 'E' | 'p' | 'P') => {
                            move_player(CellWall::Up);
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
                        KeyCode::Esc => {
                            clock.pause();
                            match ui::menu(
                                &mut self.renderer,
                                self.style,
                                &mut self.stdout,
                                "Paused",
                                &["Resume", "Main Menu", "Quit"],
                                0,
                                false,
                            )? {
                                0 => {}
                                1 => break Err(Error::Quit),
                                2 => break Err(Error::FullQuit),
                                _ => {}
                            }
                            clock.resume();
                        }
                        _ => {}
                    },
                    Err(err) => {
                        break Err(Error::CrossTermError(err));
                    }
                    _ => {}
                }

                self.renderer.event(&event.unwrap());
            }

            let from_start = start_time.elapsed(&clock);
            self.render_game(
                &maze,
                player_pos,
                player_offset,
                goal_pos,
                is_tower,
                (
                    &format!(
                        "{}x{}x{}",
                        player_pos.0 + 1,
                        player_pos.1 + 1,
                        player_pos.2 + 1
                    ),
                    if spectator { "Spectator" } else { "Adventure" },
                    &format!("{} moves", move_count),
                    &ui::format_duration(from_start),
                ),
                1,
                &moves,
            )?;

            let play_time = start_time.elapsed(&clock);

            // check if player won
            if player_pos == goal_pos {
                if let KeyCode::Char('r' | 'R') = ui::popup(
                    &mut self.renderer,
                    self.style,
                    &mut self.stdout,
                    "You won",
                    &[
                        &format!("Time: {}", ui::format_duration(play_time)),
                        &format!("Moves: {}", move_count),
                        &format!("Size: {}x{}x{}", msize.0, msize.1, msize.2),
                        "",
                        "R for new game",
                    ],
                )? {
                    break Err(Error::NewGame);
                }
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
        ups_as_goal: bool,
        texts: (&str, &str, &str, &str),
        text_horizontal_margin: i32,
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

        // drawing stairs
        let draw_stairs =
            |renderer: &mut Renderer, cell: &Cell, style: ContentStyle, pos: (i32, i32)| {
                if !cell.get_wall(CellWall::Up) && !cell.get_wall(CellWall::Down) {
                    ui::draw_char(renderer, pos.0, pos.1, '⥮', style);
                } else if !cell.get_wall(CellWall::Up) {
                    ui::draw_char(
                        renderer,
                        pos.0,
                        pos.1,
                        '↑',
                        if ups_as_goal {
                            ContentStyle {
                                foreground_color: Some(Color::DarkYellow),
                                background_color: Default::default(),
                                attributes: Default::default(),
                            }
                        } else {
                            style
                        },
                    );
                } else if !cell.get_wall(CellWall::Down) {
                    ui::draw_char(renderer, pos.0, pos.1, '↓', style);
                }
            };

        // drawing maze itself
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

                draw_stairs(&mut self.renderer, cell, self.style, (xpos, ypos));

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

            draw_stairs(
                &mut self.renderer,
                &maze.get_cells()[floor as usize][player_pos.1 as usize][player_pos.0 as usize],
                ContentStyle {
                    foreground_color: Some(Color::Green),
                    background_color: Default::default(),
                    attributes: Default::default(),
                },
                (player_pos.0 * 2 + 1 + pos.0, player_pos.1 * 2 + 1 + pos.1),
            );
        }

        // Print texts
        let str_pos_tl = (text_horizontal_margin, 0);
        let str_pos_tr = (
            size.0 as i32 - text_horizontal_margin - texts.1.len() as i32,
            0,
        );
        let str_pos_bl = (text_horizontal_margin, size.1 as i32 - 2);
        let str_pos_br = (
            size.0 as i32 - text_horizontal_margin - texts.3.len() as i32,
            size.1 as i32 - 2,
        );

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
