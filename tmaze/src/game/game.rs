use std::io::{stdout, Stdout};
use std::path::PathBuf;
use std::time::Duration;

use cmaze::game::{Game, GameProperities, GameState};
use crossterm::{
    event::{poll, read, Event, KeyCode, KeyEvent},
    terminal::size,
};
use masof::Renderer;

use crate::helpers::LineDir;
use crate::maze::{algorithms::*, Cell};
use crate::maze::{CellWall, Maze};
use crate::settings::{CameraMode, MazeGenAlgo, Settings};
use crate::ui::MenuError;
use crate::{helpers, ui, ui::CrosstermError};
use cmaze::core::*;
use dirs::preference_dir;

#[derive(Debug)]
pub enum GameError {
    CrosstermError(CrosstermError),
    EmptyMaze,
    Back,
    FullQuit,
    NewGame,
}

impl From<MenuError> for GameError {
    fn from(error: MenuError) -> Self {
        match error {
            MenuError::CrosstermError(error) => Self::CrosstermError(error),
            MenuError::EmptyMenu => Self::EmptyMaze,
            MenuError::Exit => Self::Back,
            MenuError::FullQuit => Self::FullQuit,
        }
    }
}

impl From<CrosstermError> for GameError {
    fn from(error: CrosstermError) -> Self {
        Self::CrosstermError(error)
    }
}

impl From<crossterm::ErrorKind> for GameError {
    fn from(error: crossterm::ErrorKind) -> Self {
        Self::CrosstermError(CrosstermError::from(error))
    }
}

impl From<masof::renderer::Error> for GameError {
    fn from(error: masof::renderer::Error) -> Self {
        Self::CrosstermError(CrosstermError::from(error))
    }
}

pub struct App {
    renderer: Renderer,
    stdout: Stdout,
    settings: Settings,
    last_edge_follow_offset: Dims,
    settings_file_path: PathBuf,
}

impl App {
    pub fn new() -> Self {
        let settings_path = preference_dir().unwrap().join("tmaze").join("settings.ron");
        App {
            renderer: Renderer::default(),
            stdout: stdout(),
            settings: Settings::load(settings_path.clone()),
            last_edge_follow_offset: Dims(0, 0),
            settings_file_path: settings_path,
        }
    }

    pub fn run(mut self) -> Result<(), GameError> {
        self.renderer.term_on(&mut self.stdout)?;
        let mut game_restart_reqested = false;

        loop {
            if game_restart_reqested {
                game_restart_reqested = false;
                match self.run_game() {
                    Ok(_) | Err(GameError::Back) => {}
                    Err(GameError::NewGame) => {
                        game_restart_reqested = true;
                    }
                    Err(_) => break,
                }
                continue;
            }

            match ui::menu(
                &mut self.renderer,
                self.settings.color_scheme.normals(),
                self.settings.color_scheme.texts(),
                "TMaze",
                &["New Game", "Settings", "Controls", "About", "Quit"],
                None,
                true,
            ) {
                Ok(res) => match res {
                    0 => match self.run_game() {
                        Ok(_) | Err(GameError::Back) => {}
                        Err(GameError::NewGame) => {
                            game_restart_reqested = true;
                        }
                        Err(_) => break,
                    },

                    1 => {
                        ui::popup(
                            &mut self.renderer,
                            self.settings.color_scheme.normals(),
                            self.settings.color_scheme.texts(),
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
                            self.settings.color_scheme.texts(),
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
                            self.settings.color_scheme.texts(),
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
                Err(MenuError::Exit) => break,
                Err(_) => break,
            };
        }

        self.renderer.term_off(&mut self.stdout)?;
        Ok(())
    }

    fn run_game(&mut self) -> Result<(), GameError> {
        let props = self.get_game_properities()?;
        self.run_game_with_props(props)
    }

    fn run_game_with_props(
        &mut self,
        game_props: (
            GameMode,
            fn(Dims3D, bool) -> Result<MazeGeneratorComunication, GenerationErrorInstant>,
        ),
    ) -> Result<(), GameError> {
        let (
            GameMode {
                size: msize,
                is_tower,
            },
            _,
        ) = game_props;

        let mut game = {
            let mut last_progress = f64::MIN;
            let res = Game::new_threaded(GameProperities {
                game_mode: game_props.0,
                generator: game_props.1,
            });

            let (handle, stop_flag, progress) = match res {
                Ok(com) => com,
                Err(GenerationErrorInstant::InvalidSize(dims)) => {
                    ui::popup(
                        &mut self.renderer,
                        self.settings.color_scheme.normals(),
                        self.settings.color_scheme.texts(),
                        "Error",
                        &[
                            "Invalid maze size",
                            &format!(" {}x{}x{}", dims.0, dims.1, dims.2),
                        ],
                    )?;
                    return Err(GameError::EmptyMaze);
                }
            };

            for (done, from) in progress.iter() {
                let current_progress = done as f64 / from as f64;

                if let Ok(true) = poll(Duration::from_nanos(1)) {
                    if let Ok(Event::Key(KeyEvent { code, modifiers: _ })) = read() {
                        match code {
                            KeyCode::Esc => {
                                stop_flag.stop();
                                let _ = handle.join().unwrap();
                                return Err(GameError::Back);
                            }
                            KeyCode::Char('q' | 'Q') => {
                                stop_flag.stop();
                                let _ = handle.join().unwrap();
                                return Err(GameError::FullQuit);
                            }
                            _ => {}
                        }
                    }
                }

                if current_progress - last_progress > 0.0001 {
                    last_progress = current_progress;
                    ui::render_progress(
                        &mut self.renderer,
                        self.settings.color_scheme.normals(),
                        self.settings.color_scheme.texts(),
                        &format!(
                            " Generating maze ({}x{}x{})... {:.2} % ",
                            msize.0,
                            msize.1,
                            msize.2,
                            current_progress * 100.0
                        ),
                        current_progress,
                    )?;
                }
            }

            match handle.join().unwrap() {
                Ok(game) => game,
                Err(GenerationErrorThreaded::GenerationError(
                    GenerationErrorInstant::InvalidSize(dims),
                )) => {
                    ui::popup(
                        &mut self.renderer,
                        self.settings.color_scheme.normals(),
                        self.settings.color_scheme.texts(),
                        "Error",
                        &[
                            "Invalid maze size",
                            &format!(" {}x{}x{}", dims.0, dims.1, dims.2),
                        ],
                    )?;
                    return Err(GameError::EmptyMaze);
                }
                Err(GenerationErrorThreaded::AbortGeneration) => return Err(GameError::Back),
                Err(GenerationErrorThreaded::UnknownError(err)) => panic!("{:?}", err),
            }
        };

        let mut camera_offset = Dims3D(0, 0, 0);
        let mut spectator = false;

        self.render_game(
            game.get_maze(),
            game.get_player_pos(),
            camera_offset,
            self.settings.camera_mode,
            game.get_goal_pos(),
            is_tower,
            (
                &format!(
                    "{}x{}x{}",
                    game.get_player_pos().0 + 1,
                    game.get_player_pos().1 + 1,
                    game.get_player_pos().2 + 1
                ),
                if spectator { "Spectator" } else { "Adventure" },
                &format!("{} moves", game.get_move_count()),
                "",
            ),
            1,
            game.get_moves(),
        )?;

        game.start().unwrap();

        loop {
            if let Ok(true) = poll(Duration::from_millis(90)) {
                let event = read();

                let mut apply_move = |wall: CellWall| {
                    if spectator {
                        let cam_off = wall.reverse_wall().to_coord() + camera_offset;

                        camera_offset = Dims3D(
                            cam_off.0,
                            cam_off.1,
                            (-game.get_player_pos().2).max(
                                (game.get_maze().size().2 - game.get_player_pos().2 - 1)
                                    .min(cam_off.2),
                            ),
                        )
                    } else {
                        game.move_player(
                            wall,
                            self.settings.slow,
                            !self.settings.disable_tower_auto_up,
                        )
                        .unwrap();
                    }
                };

                match event {
                    Ok(Event::Key(KeyEvent { code, modifiers: _ })) => match code {
                        KeyCode::Up | KeyCode::Char('w' | 'W') => {
                            apply_move(CellWall::Top);
                        }
                        KeyCode::Down | KeyCode::Char('s' | 'S') => {
                            apply_move(CellWall::Bottom);
                        }
                        KeyCode::Left | KeyCode::Char('a' | 'A') => {
                            apply_move(CellWall::Left);
                        }
                        KeyCode::Right | KeyCode::Char('d' | 'D') => {
                            apply_move(CellWall::Right);
                        }
                        KeyCode::Char('f' | 'F' | 'q' | 'Q' | 'l' | 'L') => {
                            apply_move(CellWall::Down);
                        }
                        KeyCode::Char('r' | 'R' | 'e' | 'E' | 'p' | 'P') => {
                            apply_move(CellWall::Up);
                        }
                        KeyCode::Char(' ') => {
                            if spectator {
                                camera_offset = Dims3D(0, 0, 0);
                                spectator = false
                            } else {
                                spectator = true
                            }
                        }
                        KeyCode::Esc => {
                            game.pause().unwrap();
                            match ui::menu(
                                &mut self.renderer,
                                self.settings.color_scheme.normals(),
                                self.settings.color_scheme.texts(),
                                "Paused",
                                &["Resume", "Main Menu", "Quit"],
                                None,
                                false,
                            )? {
                                1 => break Err(GameError::Back),
                                2 => break Err(GameError::FullQuit),
                                _ => {}
                            }
                            game.resume().unwrap();
                        }
                        _ => {}
                    },
                    Err(err) => {
                        break Err(CrosstermError(err).into());
                    }
                    _ => {}
                }

                self.renderer.event(&event.unwrap());
            }

            let from_start = game.get_elapsed().unwrap();
            self.render_game(
                game.get_maze(),
                game.get_player_pos(),
                camera_offset,
                self.settings.camera_mode,
                game.get_goal_pos(),
                is_tower,
                (
                    &format!(
                        "{}x{}x{}",
                        game.get_player_pos().0 + 1,
                        game.get_player_pos().1 + 1,
                        game.get_player_pos().2 + 1
                    ),
                    if spectator { "Spectator" } else { "Adventure" },
                    &format!("{} moves", game.get_move_count()),
                    &ui::format_duration(from_start),
                ),
                1,
                game.get_moves(),
            )?;

            // check if player won
            if game.get_state() == GameState::Finished {
                let play_time = game.get_elapsed().unwrap();

                if let KeyCode::Char('r' | 'R') = ui::popup(
                    &mut self.renderer,
                    self.settings.color_scheme.normals(),
                    self.settings.color_scheme.texts(),
                    "You won",
                    &[
                        &format!("Time: {}", ui::format_duration(play_time)),
                        &format!("Moves: {}", game.get_move_count()),
                        &format!("Size: {}x{}x{}", msize.0, msize.1, msize.2),
                        "",
                        "R for new game",
                    ],
                )? {
                    break Err(GameError::NewGame);
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
    ) -> Result<(), GameError> {
        let maze_render_size = helpers::maze_render_size(maze);
        let size = {
            let size = size()?;
            (size.0 as i32, size.1 as i32)
        };
        let is_around_player =
            maze_render_size.0 > size.0 as i32 || maze_render_size.1 + 3 > size.1 as i32;

        let pos = {
            let pos = if is_around_player {
                let player_real_maze_pos = helpers::from_maze_to_real(player_pos);

                match camera_mode {
                    CameraMode::CloseFollow => Dims(
                        size.0 / 2 - player_real_maze_pos.0,
                        size.1 / 2 - player_real_maze_pos.1,
                    ),
                    CameraMode::EdgeFollow(margin_x, margin_y) => {
                        let current_player_real_pos =
                            self.last_edge_follow_offset + player_real_maze_pos;

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
                ui::box_center_screen(Dims(maze_render_size.0 as i32, maze_render_size.1 as i32))?
            };

            (pos.0 + camera_offset.0 * 2, pos.1 + camera_offset.1 * 2)
        };

        let floor = player_pos.2 + camera_offset.2;

        self.renderer.begin()?;

        let draw_line_double_duo = |self_: &mut App, pos: (i32, i32), l1: LineDir, l2: LineDir| {
            ui::draw_str(
                &mut self_.renderer,
                pos.0,
                pos.1,
                &format!("{}{}", l1.double_line(), l2.double_line(),),
                self_.settings.color_scheme.normals(),
            )
        };

        let draw_line_double = |self_: &mut App, pos: (i32, i32), l: LineDir| {
            ui::draw_str(
                &mut self_.renderer,
                pos.0,
                pos.1,
                &format!("{}", l.double_line(),),
                self_.settings.color_scheme.normals(),
            )
        };

        // corners
        if pos.1 > 0 {
            draw_line_double_duo(self, pos, LineDir::BottomRight, LineDir::Horizontal);
            draw_line_double_duo(
                self,
                (pos.0 + maze_render_size.0 - 2, pos.1),
                LineDir::Horizontal,
                LineDir::BottomLeft,
            );
        }

        if pos.1 + maze_render_size.1 - 2 < size.1 - 3 {
            draw_line_double(
                self,
                (pos.0, pos.1 + maze_render_size.1 - 2),
                LineDir::Vertical,
            );
            draw_line_double(
                self,
                (
                    pos.0 + maze_render_size.0 - 1,
                    pos.1 + maze_render_size.1 - 2,
                ),
                LineDir::Vertical,
            );
        }
        if pos.1 + maze_render_size.1 - 1 < size.1 - 2 {
            draw_line_double(
                self,
                (pos.0, pos.1 + maze_render_size.1 - 1),
                LineDir::TopRight,
            );
            draw_line_double_duo(
                self,
                (
                    pos.0 + maze_render_size.0 - 2,
                    pos.1 + maze_render_size.1 - 1,
                ),
                LineDir::Horizontal,
                LineDir::TopLeft,
            );
        }
        // horizontal edge lines
        for x in 0..maze.size().0 - 1 {
            if pos.1 > 0 {
                draw_line_double_duo(
                    self,
                    (x as i32 * 2 + pos.0 + 1, pos.1),
                    LineDir::Horizontal,
                    if maze
                        .get_cell(Dims3D(x, 0, floor))
                        .unwrap()
                        .get_wall(CellWall::Right)
                    {
                        LineDir::ClosedTop
                    } else {
                        LineDir::Horizontal
                    },
                );
            }

            if pos.1 + maze_render_size.1 - 1 < size.1 - 2 {
                draw_line_double_duo(
                    self,
                    (x as i32 * 2 + pos.0 + 1, pos.1 + maze_render_size.1 - 1),
                    LineDir::Horizontal,
                    if maze
                        .get_cell(Dims3D(x, maze.size().1 - 1, floor))
                        .unwrap()
                        .get_wall(CellWall::Right)
                    {
                        LineDir::ClosedBottom
                    } else {
                        LineDir::Horizontal
                    },
                );
            }
        }

        // vertical edge lines
        for y in 0..maze.size().1 - 1 {
            let ypos = y as i32 * 2 + pos.1 + 1;
            if ypos >= size.1 - 2 {
                break;
            }

            if ypos == -1 {
                continue;
            }

            if ypos + 1 < size.1 {
                draw_line_double(
                    self,
                    (pos.0, ypos + 1),
                    if maze
                        .get_cell(Dims3D(0, y, floor))
                        .unwrap()
                        .get_wall(CellWall::Bottom)
                    {
                        LineDir::ClosedLeft
                    } else {
                        LineDir::Vertical
                    },
                );

                draw_line_double(
                    self,
                    (pos.0 + maze_render_size.0 - 1, ypos + 1),
                    if maze
                        .get_cell(Dims3D(maze.size().0 - 1, y, floor))
                        .unwrap()
                        .get_wall(CellWall::Bottom)
                    {
                        LineDir::ClosedLeft
                    } else {
                        LineDir::Vertical
                    },
                );
            }

            draw_line_double(self, (pos.0, ypos), LineDir::Vertical);

            draw_line_double(
                self,
                (pos.0 + maze_render_size.0 - 1, y as i32 * 2 + pos.1 + 1),
                LineDir::Vertical,
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

        // helper for drawing the stairs
        let draw_stairs = |self_: &mut Self, cell: &Cell, stairs_pos: (i32, i32)| {
            if !cell.get_wall(CellWall::Up) && !cell.get_wall(CellWall::Down) {
                ui::draw_char(
                    &mut self_.renderer,
                    stairs_pos.0,
                    stairs_pos.1,
                    '⥮',
                    if player_pos.2 == floor
                        && player_pos.0 * 2 + 1 + pos.0 == stairs_pos.0
                        && player_pos.1 * 2 + 1 + pos.1 == stairs_pos.1
                    {
                        self_.settings.color_scheme.players()
                    } else {
                        self_.settings.color_scheme.normals()
                    },
                );
            } else if !cell.get_wall(CellWall::Up) {
                ui::draw_char(
                    &mut self_.renderer,
                    stairs_pos.0,
                    stairs_pos.1,
                    '↑',
                    if player_pos.2 == floor
                        && player_pos.0 * 2 + 1 + pos.0 == stairs_pos.0
                        && player_pos.1 * 2 + 1 + pos.1 == stairs_pos.1
                    {
                        self_.settings.color_scheme.players()
                    } else {
                        if ups_as_goal {
                            self_.settings.color_scheme.goals()
                        } else {
                            self_.settings.color_scheme.normals()
                        }
                    },
                );
            } else if !cell.get_wall(CellWall::Down) {
                ui::draw_char(
                    &mut self_.renderer,
                    stairs_pos.0,
                    stairs_pos.1,
                    '↓',
                    if player_pos.2 == floor
                        && player_pos.0 * 2 + 1 + pos.0 == stairs_pos.0
                        && player_pos.1 * 2 + 1 + pos.1 == stairs_pos.1
                    {
                        self_.settings.color_scheme.players()
                    } else {
                        self_.settings.color_scheme.normals()
                    },
                );
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
                    draw_line_double(self, (xpos + 1, ypos), LineDir::Vertical);
                }
                if ypos + 1 < size.1 as i32 - 2
                    && cell.get_wall(CellWall::Bottom)
                    && iy != maze.size().1 as usize - 1
                {
                    draw_line_double(self, (xpos, ypos + 1), LineDir::Horizontal);
                }

                draw_stairs(self, cell, (xpos, ypos));

                if iy == maze.size().1 as usize - 1 || ix == maze.size().0 as usize - 1 {
                    continue;
                }

                let cell2 = &maze.get_cells()[floor as usize][iy + 1][ix + 1];

                if ypos < size.1 as i32 - 3 {
                    ui::draw_str(
                        &mut self.renderer,
                        ix as i32 * 2 + 2 + pos.0,
                        iy as i32 * 2 + 2 + pos.1,
                        LineDir::double_line_bools(
                            cell.get_wall(CellWall::Bottom),
                            cell.get_wall(CellWall::Right),
                            cell2.get_wall(CellWall::Top),
                            cell2.get_wall(CellWall::Left),
                        )
                        .double_line(),
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
                self,
                &maze.get_cells()[floor as usize][player_pos.1 as usize][player_pos.0 as usize],
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
            self.settings.color_scheme.texts(),
        );
        ui::draw_str(
            &mut self.renderer,
            str_pos_tr.0,
            str_pos_tr.1,
            texts.1,
            self.settings.color_scheme.texts(),
        );
        ui::draw_str(
            &mut self.renderer,
            str_pos_bl.0,
            str_pos_bl.1,
            texts.2,
            self.settings.color_scheme.texts(),
        );
        ui::draw_str(
            &mut self.renderer,
            str_pos_br.0,
            str_pos_br.1,
            texts.3,
            self.settings.color_scheme.texts(),
        );

        self.renderer.end(&mut self.stdout)?;

        Ok(())
    }

    fn get_game_properities(
        &mut self,
    ) -> Result<
        (
            GameMode,
            fn(Dims3D, bool) -> Result<MazeGeneratorComunication, GenerationErrorInstant>,
        ),
        GameError,
    > {
        Ok((
            *ui::choice_menu(
                &mut self.renderer,
                self.settings.color_scheme.normals(),
                self.settings.color_scheme.texts(),
                "Maze size",
                &self
                    .settings
                    .mazes
                    .iter()
                    .map(|maze| {
                        (
                            GameMode {
                                size: Dims3D(
                                    maze.width as i32,
                                    maze.height as i32,
                                    maze.depth as i32,
                                ),
                                is_tower: maze.tower,
                            },
                            maze.title.as_str(),
                        )
                    })
                    .collect::<Vec<_>>(),
                self.settings.mazes.iter().position(|maze| maze.default),
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
                    self.settings.color_scheme.texts(),
                    "Maze generation algorithm",
                    &["Randomized Kruskal's", "Depth-first search"],
                    match self.settings.default_maze_gen_algo {
                        MazeGenAlgo::RandomKruskals => Some(0),
                        MazeGenAlgo::DepthFirstSearch => Some(1),
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
