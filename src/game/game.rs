use std::io::{stdout, Stdout};
use std::path::PathBuf;
use std::time::Duration;

use crossterm::{
    event::{poll, read, Event, KeyCode, KeyEvent},
    terminal::size,
};
use masof::{ContentStyle, Renderer};

use crate::maze::{algorithms::*, Cell};
use crate::maze::{CellWall, Maze};
use crate::settings::{CameraMode, MazeGenAlgo, Settings};
use crate::tmcore::*;
use crate::{helpers, ui};
use dirs::preference_dir;
use pausable_clock::PausableClock;

pub struct Game {
    renderer: Renderer,
    stdout: Stdout,
    settings: Settings,
    last_edge_follow_offset: Dims,
    settings_file_path: PathBuf,
}

impl Game {
    pub fn new() -> Self {
        let settings_path = preference_dir().unwrap().join("tmaze").join("settings.ron");
        Game {
            renderer: Renderer::default(),
            stdout: stdout(),
            settings: Settings::load(settings_path.clone()),
            last_edge_follow_offset: (0, 0),
            settings_file_path: settings_path,
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
                self.settings.color_scheme.normals(),
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
                            self.settings.color_scheme.normals(),
                            "Settings",
                            &[
                                "Settings file is located at:",
                                &format!(" {}", self.settings_file_path.to_str().unwrap()),
                            ],
                        )?;
                    }
                    2 => {
                        ui::popup(
                            &mut self.renderer,
                            self.settings.color_scheme.normals(),
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
                            self.settings.color_scheme.normals(),
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
                                "",
                                "Version:",
                                &format!("    {}", env!("CARGO_PKG_VERSION")),
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
        let (maze_mode, generation_func) = self.get_game_properities()?;
        let msize: Dims3D = (maze_mode.0, maze_mode.1, maze_mode.2);
        let is_tower = maze_mode.3;

        let mut player_pos = (0, 0, 0);
        let goal_pos = (msize.0 - 1, msize.1 - 1, msize.2 - 1);

        let mut camera_offset = (0, 0, 0);
        let mut spectator = false;

        let maze = {
            let mut last_progress = f64::MIN;
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
                            self.settings.color_scheme.normals(),
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
            camera_offset,
            self.settings.camera_mode,
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
                        camera_offset = {
                            let off = match wall {
                                CellWall::Top => (0, 1, 0),
                                CellWall::Bottom => (0, -1, 0),
                                CellWall::Left => (1, 0, 0),
                                CellWall::Right => (-1, 0, 0),
                                CellWall::Up => (0, 0, 1),
                                CellWall::Down => (0, 0, -1),
                            };

                            (
                                camera_offset.0 + off.0,
                                camera_offset.1 + off.1,
                                (-player_pos.2).max(
                                    (maze.size().2 - player_pos.2 - 1).min(camera_offset.2 + off.2),
                                ),
                            )
                        };
                    } else {
                        let pmove = get_new_player_pos(
                            &maze,
                            player_pos,
                            wall,
                            self.settings.slow,
                            &mut moves,
                        );
                        player_pos = pmove.0;
                        move_count += pmove.1;

                        if !self.settings.disable_tower_auto_up
                            && is_tower
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
                                camera_offset = (0, 0, 0);
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
                                self.settings.color_scheme.normals(),
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
                camera_offset,
                self.settings.camera_mode,
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
                    self.settings.color_scheme.normals(),
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
        camera_offset: Dims3D,
        camera_mode: CameraMode,
        goal_pos: Dims3D,
        ups_as_goal: bool,
        texts: (&str, &str, &str, &str),
        text_horizontal_margin: i32,
        moves: &[(Dims3D, CellWall)],
    ) -> Result<(), Error> {
        let maze_render_size = helpers::maze_render_size(maze);
        let size = {
            let size = size()?;
            (size.0 as i32, size.1 as i32)
        };
        let is_around_player =
            maze_render_size.0 > size.0 as i32 || maze_render_size.1 + 2 > size.1 as i32;

        let pos = {
            let pos = if is_around_player {
                let player_real_maze_pos = helpers::from_maze_to_real(player_pos);

                match camera_mode {
                    CameraMode::CloseFollow => (
                        size.0 / 2 - player_real_maze_pos.0,
                        size.1 / 2 - player_real_maze_pos.1,
                    ),
                    CameraMode::EdgeFollow(margin_x, margin_y) => {
                        let current_player_real_pos = (
                            self.last_edge_follow_offset.0 + player_real_maze_pos.0,
                            self.last_edge_follow_offset.1 + player_real_maze_pos.1,
                        );

                        if current_player_real_pos.0 < margin_x
                            || current_player_real_pos.0 > size.0 - margin_x
                        {
                            self.last_edge_follow_offset.0 = size.0 / 2 - player_real_maze_pos.0;
                        }

                        if current_player_real_pos.1 < margin_y
                            || current_player_real_pos.1 > size.1 - margin_y
                        {
                            self.last_edge_follow_offset.1 = size.1 / 2 - player_real_maze_pos.1;
                        }
                        self.last_edge_follow_offset
                    }
                }
            } else {
                ui::box_center_screen((maze_render_size.0 as i32, maze_render_size.1 as i32))?
            };

            (pos.0 + camera_offset.0 * 2, pos.1 + camera_offset.1 * 2)
        };

        let floor = player_pos.2 + camera_offset.2;

        self.renderer.begin()?;

        let draw_corner_double =
            |self_: &mut Game, x, y, c1: (bool, bool, bool, bool), c2: (bool, bool, bool, bool)| {
                ui::draw_str(
                    &mut self_.renderer,
                    x,
                    y,
                    &format!(
                        "{}{}",
                        helpers::double_line_corner(c1.0, c1.1, c1.2, c1.3),
                        helpers::double_line_corner(c2.0, c2.1, c2.2, c2.3)
                    ),
                    self_.settings.color_scheme.normals(),
                )
            };

        let draw_corner_single = |self_: &mut Game, x, y, c: (bool, bool, bool, bool)| {
            ui::draw_str(
                &mut self_.renderer,
                x,
                y,
                &format!("{}", helpers::double_line_corner(c.0, c.1, c.2, c.3),),
                self_.settings.color_scheme.normals(),
            )
        };

        // corners
        if pos.1 > 0 {
            draw_corner_double(
                self,
                pos.0,
                pos.1,
                (false, false, true, true),
                (true, false, true, false),
            );
            draw_corner_double(
                self,
                pos.0 + maze_render_size.0 - 2,
                pos.1,
                (true, false, true, false),
                (true, false, false, true),
            );
        }

        if pos.1 + maze_render_size.1 - 2 < size.1 - 3 {
            draw_corner_single(
                self,
                pos.0,
                pos.1 + maze_render_size.1 - 2,
                (false, true, false, true),
            );
            draw_corner_single(
                self,
                pos.0 + maze_render_size.0 - 1,
                pos.1 + maze_render_size.1 - 2,
                (false, true, false, true),
            );
        }
        if pos.1 + maze_render_size.1 - 1 < size.1 - 2 {
            draw_corner_single(
                self,
                pos.0,
                pos.1 + maze_render_size.1 - 1,
                (false, true, true, false),
            );
            draw_corner_double(
                self,
                pos.0 + maze_render_size.0 - 2,
                pos.1 + maze_render_size.1 - 1,
                (true, false, true, false),
                (true, true, false, false),
            );
        }
        // horizontal edge lines
        for x in 0..maze.size().0 - 1 {
            if pos.1 > 0 {
                draw_corner_double(
                    self,
                    x as i32 * 2 + pos.0 + 1,
                    pos.1,
                    (true, false, true, false),
                    (
                        true,
                        false,
                        true,
                        maze.get_cells()[floor as usize][0][x as usize].get_wall(CellWall::Right),
                    ),
                );
            }

            if pos.1 + maze_render_size.1 - 1 < size.1 - 2 {
                draw_corner_double(
                    self,
                    x as i32 * 2 + pos.0 + 1,
                    pos.1 + maze_render_size.1 - 1,
                    (true, false, true, false),
                    (
                        true,
                        maze.get_cells()[floor as usize][maze.size().1 as usize - 1][x as usize]
                            .get_wall(CellWall::Right),
                        true,
                        false,
                    ),
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
                self.settings.color_scheme.normals(),
            );

            if ypos + 1 < size.1 {
                draw_corner_single(
                    self,
                    pos.0,
                    y as i32 * 2 + pos.1 + 2,
                    (
                        false,
                        true,
                        maze.get_cells()[floor as usize][y as usize][0].get_wall(CellWall::Bottom),
                        true,
                    ),
                );

                draw_corner_single(
                    self,
                    pos.0 + maze_render_size.0 - 1,
                    y as i32 * 2 + pos.1 + 2,
                    (
                        maze.get_cells()[floor as usize][y as usize][maze.size().0 as usize - 1]
                            .get_wall(CellWall::Bottom),
                        true,
                        false,
                        true,
                    ),
                );
            }

            draw_corner_single(
                self,
                pos.0 + maze_render_size.0 - 1,
                y as i32 * 2 + pos.1 + 1,
                (false, true, false, true),
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
                    self.settings.color_scheme.normals(),
                );
            }
        }

        // drawing stairs
        let draw_stairs = |renderer: &mut Renderer,
                           cell: &Cell,
                           style: ContentStyle,
                           pos: (i32, i32),
                           force_style: bool| {
            if !cell.get_wall(CellWall::Up) && !cell.get_wall(CellWall::Down) {
                ui::draw_char(renderer, pos.0, pos.1, '⥮', style);
            } else if !cell.get_wall(CellWall::Up) {
                ui::draw_char(
                    renderer,
                    pos.0,
                    pos.1,
                    '↑',
                    if ups_as_goal && !force_style {
                        self.settings.color_scheme.goals()
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
                        self.settings.color_scheme.normals(),
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
                        self.settings.color_scheme.normals(),
                    );
                }

                draw_stairs(
                    &mut self.renderer,
                    cell,
                    self.settings.color_scheme.normals(),
                    (xpos, ypos),
                    false,
                );

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
                        self.settings.color_scheme.normals(),
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
                self.settings.color_scheme.goals(),
            );
        }

        if floor == player_pos.2 {
            ui::draw_char(
                &mut self.renderer,
                player_pos.0 * 2 + 1 + pos.0,
                player_pos.1 * 2 + 1 + pos.1,
                'O',
                self.settings.color_scheme.players(),
            );

            draw_stairs(
                &mut self.renderer,
                &maze.get_cells()[floor as usize][player_pos.1 as usize][player_pos.0 as usize],
                self.settings.color_scheme.players(),
                (player_pos.0 * 2 + 1 + pos.0, player_pos.1 * 2 + 1 + pos.1),
                true,
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
            self.settings.color_scheme.normals(),
        );
        ui::draw_str(
            &mut self.renderer,
            str_pos_tr.0,
            str_pos_tr.1,
            texts.1,
            self.settings.color_scheme.normals(),
        );
        ui::draw_str(
            &mut self.renderer,
            str_pos_bl.0,
            str_pos_bl.1,
            texts.2,
            self.settings.color_scheme.normals(),
        );
        ui::draw_str(
            &mut self.renderer,
            str_pos_br.0,
            str_pos_br.1,
            texts.3,
            self.settings.color_scheme.normals(),
        );

        self.renderer.end(&mut self.stdout)?;

        Ok(())
    }

    fn get_game_properities<T: FnMut(usize, usize) -> Result<(), Error>>(
        &mut self,
    ) -> Result<
        (
            GameMode,
            fn((i32, i32, i32), bool, Option<T>) -> Result<Maze, Error>,
        ),
        Error,
    > {
        Ok((
            *ui::choice_menu(
                &mut self.renderer,
                self.settings.color_scheme.normals(),
                "Maze size",
                &self
                    .settings
                    .mazes
                    .iter()
                    .map(|maze| {
                        (
                            (maze.width as i32, maze.height as i32, maze.depth as i32, maze.tower),
                            maze.title.as_str(),
                        )
                    })
                    .collect::<Vec<_>>(),
                0,
                false,
            )?,
            if self.settings.dont_ask_for_maze_algo {
                match self.settings.default_maze_gen_algo {
                    MazeGenAlgo::RandomKruskals => RndKruskals::generate,
                    MazeGenAlgo::DepthFirstSearch => DepthFirstSearch::generate,
                }
            } else {
                match ui::menu(
                    &mut self.renderer,
                    self.settings.color_scheme.normals(),
                    "Maze generation algorithm",
                    &["Randomized Kruskal's", "Depth-first search"],
                    match self.settings.default_maze_gen_algo {
                        MazeGenAlgo::RandomKruskals => 0,
                        MazeGenAlgo::DepthFirstSearch => 1,
                    },
                    true,
                )? {
                    0 => RndKruskals::generate,
                    1 => DepthFirstSearch::generate,
                    _ => panic!(),
                }
            },
        ))
    }
}
