use std::cell::RefCell;
use std::io::{stdout, Stdout};
use std::path::PathBuf;
use std::time::Duration;

use cmaze::game::{Game, GameProperities, GameState as GameStatus};
use crossterm::{
    event::{poll, read, Event, KeyCode, KeyEvent},
    terminal::size,
};
use masof::Renderer;

use crate::helpers::{constants, LineDir};
use crate::maze::CellWall;
use crate::maze::{algorithms::*, Cell};
use crate::settings::{CameraMode, MazeGenAlgo, Settings};
use crate::ui::{DrawContext, MenuError};
use crate::{helpers, ui, ui::CrosstermError};
use cmaze::core::*;
use dirs::preference_dir;

use super::{GameError, GameState, GameViewMode};

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
                self.settings.get_color_scheme().normals(),
                self.settings.get_color_scheme().texts(),
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
                        self.show_settings_popup()?;
                    }
                    2 => {
                        self.show_controls_popup()?;
                    }
                    3 => {
                        self.show_about_popup()?;
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

    fn show_settings_popup(&mut self) -> Result<(), GameError> {
        ui::popup(
            &mut self.renderer,
            self.settings.get_color_scheme().normals(),
            self.settings.get_color_scheme().texts(),
            "Settings",
            &[
                "Settings file is located at:",
                &format!(" {}", self.settings_file_path.to_str().unwrap()),
            ],
        )?;

        Ok(())
    }

    fn show_controls_popup(&mut self) -> Result<(), GameError> {
        ui::popup(
            &mut self.renderer,
            self.settings.get_color_scheme().normals(),
            self.settings.get_color_scheme().texts(),
            "Controls",
            &[
                "WASD and arrows: move",
                "Space: switch adventure/spectaror mode",
                "Q, F or L: move down",
                "E, R or P: move up",
                "With SHIFT move at the end in single dir",
                "Escape: pause menu",
            ],
        )?;

        Ok(())
    }

    fn show_about_popup(&mut self) -> Result<(), GameError> {
        ui::popup(
            &mut self.renderer,
            self.settings.get_color_scheme().normals(),
            self.settings.get_color_scheme().texts(),
            "About",
            &[
                "This is simple maze solving game",
                "Supported algorithms:",
                "    - Depth-first search",
                "    - Kruskal's algorithm",
                "Supports 3D mazes",
                "",
                "Created by:",
                &format!("    - {}", env!("CARGO_PKG_AUTHORS")),
                "",
                "Version:",
                &format!("    {}", env!("CARGO_PKG_VERSION")),
            ],
        )?;

        Ok(())
    }

    fn run_game(&mut self) -> Result<(), GameError> {
        let props = self.get_game_properities()?;
        self.run_game_with_props(props)
    }

    fn run_game_with_props(&mut self, game_props: GameProperities) -> Result<(), GameError> {
        let GameProperities {
            game_mode:
                GameMode {
                    size: msize,
                    is_tower,
                },
            ..
        } = game_props;

        let game = self.generate_maze(game_props)?;

        let mut game_state = GameState {
            game,
            camera_offset: Dims3D(0, 0, 0),
            is_tower,
            player_char: constants::get_random_player_char(),
            view_mode: GameViewMode::Adventure,
            settings: self.settings.clone(),
        };

        game_state.game.start().unwrap();

        loop {
            if let Ok(true) = poll(Duration::from_millis(90)) {
                let event = read();

                match event {
                    Ok(Event::Key(key_event)) => {
                        if let Err(_) = game_state.handle_event(key_event) {
                            game_state.game.pause().unwrap();
                            match ui::menu(
                                &mut self.renderer,
                                self.settings.get_color_scheme().normals(),
                                self.settings.get_color_scheme().texts(),
                                "Paused",
                                &["Resume", "Main Menu", "Quit"],
                                None,
                                false,
                            )? {
                                1 => return Err(GameError::Back),
                                2 => return Err(GameError::FullQuit),
                                _ => {}
                            }
                            game_state.game.resume().unwrap();
                        }
                    }
                    Err(err) => {
                        break Err(CrosstermError(err).into());
                    }
                    _ => {}
                }

                self.renderer.event(&event.unwrap());
            }

            self.render_game(&game_state, self.settings.get_camera_mode(), is_tower, 1)?;

            // Check if player won
            if game_state.game.get_state() == GameStatus::Finished {
                let play_time = game_state.game.get_elapsed().unwrap();

                if let KeyCode::Char('r' | 'R') = ui::popup(
                    &mut self.renderer,
                    self.settings.get_color_scheme().normals(),
                    self.settings.get_color_scheme().texts(),
                    "You won",
                    &[
                        &format!("Time:  {}", ui::format_duration(play_time)),
                        &format!("Moves: {}", game_state.game.get_move_count()),
                        &format!("Size:  {}x{}x{}", msize.0, msize.1, msize.2),
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
        game_state: &GameState,
        camera_mode: CameraMode,
        ups_as_goal: bool,
        text_horizontal_margin: i32,
    ) -> Result<(), GameError> {
        let GameState {
            game,
            camera_offset,
            player_char,
            ..
        } = game_state;

        let player_pos = game.get_player_pos();

        let maze = game.get_maze();

        let maze_render_size = helpers::maze_render_size(maze);
        let size = {
            let size = size()?;
            (size.0 as i32, size.1 as i32)
        };

        let is_around_player =
            maze_render_size.0 > size.0 as i32 || maze_render_size.1 + 3 > size.1 as i32;

        let maze_pos = {
            let pos = if is_around_player {
                let last_player_real_pos = helpers::from_maze_to_real(player_pos);

                match camera_mode {
                    CameraMode::CloseFollow => Dims(
                        size.0 / 2 - last_player_real_pos.0,
                        size.1 / 2 - last_player_real_pos.1,
                    ),
                    CameraMode::EdgeFollow(margin_x, margin_y) => {
                        let player_real_pos =
                            self.last_edge_follow_offset + last_player_real_pos;

                        if player_real_pos.0 < margin_x
                            || player_real_pos.0 > size.0 - margin_x
                        {
                            self.last_edge_follow_offset.0 = size.0 / 2 - last_player_real_pos.0;
                        }

                        if player_real_pos.1 < margin_y
                            || player_real_pos.1 > size.1 - margin_y
                        {
                            self.last_edge_follow_offset.1 = size.1 / 2 - last_player_real_pos.1;
                        }
                        self.last_edge_follow_offset
                    }
                }
            } else {
                ui::box_center_screen(Dims(maze_render_size.0 as i32, maze_render_size.1 as i32))?
            };

            (pos.0 + camera_offset.0 * 2, pos.1 + camera_offset.1 * 2)
        };

        let normal_style = self.settings.get_color_scheme().normals();
        let text_style = self.settings.get_color_scheme().texts();
        let player_style = self.settings.get_color_scheme().players();
        let goal_style = self.settings.get_color_scheme().goals();

        self.renderer.begin()?;
        let renderer_cell = RefCell::new(&mut self.renderer);

        let mut normal_context = DrawContext {
            renderer: &renderer_cell,
            style: normal_style,
        };
        let mut text_context = DrawContext {
            renderer: &renderer_cell,
            style: text_style,
        };
        let mut player_context = DrawContext {
            renderer: &renderer_cell,
            style: player_style,
        };
        let mut goal_context = DrawContext {
            renderer: &renderer_cell,
            style: goal_style,
        };

        let floor = player_pos.2 + camera_offset.2;

        let draw_line_double_duo =
            |context: &mut DrawContext, pos: (i32, i32), l1: LineDir, l2: LineDir| {
                context.draw_str(
                    pos.into(),
                    &format!("{}{}", l1.double_line(), l2.double_line(),),
                )
            };

        let draw_line_double = |context: &mut DrawContext, pos: (i32, i32), l: LineDir| {
            context.draw_str(pos.into(), l.double_line())
        };

        // corners
        if maze_pos.1 > 0 {
            draw_line_double_duo(
                &mut normal_context,
                maze_pos,
                LineDir::BottomRight,
                LineDir::Horizontal,
            );
            draw_line_double_duo(
                &mut normal_context,
                (maze_pos.0 + maze_render_size.0 - 2, maze_pos.1),
                LineDir::Horizontal,
                LineDir::BottomLeft,
            );
        }

        if maze_pos.1 + maze_render_size.1 - 2 < size.1 - 3 {
            draw_line_double(
                &mut normal_context,
                (maze_pos.0, maze_pos.1 + maze_render_size.1 - 2),
                LineDir::Vertical,
            );
            draw_line_double(
                &mut normal_context,
                (
                    maze_pos.0 + maze_render_size.0 - 1,
                    maze_pos.1 + maze_render_size.1 - 2,
                ),
                LineDir::Vertical,
            );
        }

        if maze_pos.1 + maze_render_size.1 - 1 < size.1 - 2 {
            draw_line_double(
                &mut normal_context,
                (maze_pos.0, maze_pos.1 + maze_render_size.1 - 1),
                LineDir::TopRight,
            );
            draw_line_double_duo(
                &mut normal_context,
                (
                    maze_pos.0 + maze_render_size.0 - 2,
                    maze_pos.1 + maze_render_size.1 - 1,
                ),
                LineDir::Horizontal,
                LineDir::TopLeft,
            );
        }

        // Horizontal edge lines
        for x in 0..maze.size().0 - 1 {
            if maze_pos.1 > 0 {
                draw_line_double_duo(
                    &mut normal_context,
                    (x as i32 * 2 + maze_pos.0 + 1, maze_pos.1),
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

            if maze_pos.1 + maze_render_size.1 - 1 < size.1 - 2 {
                draw_line_double_duo(
                    &mut normal_context,
                    (
                        x as i32 * 2 + maze_pos.0 + 1,
                        maze_pos.1 + maze_render_size.1 - 1,
                    ),
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

        // Vertical edge lines
        for y in 0..maze.size().1 - 1 {
            let ypos = y as i32 * 2 + maze_pos.1 + 1;
            if ypos >= size.1 - 2 {
                break;
            }

            if ypos == -1 {
                continue;
            }

            if ypos + 3 < size.1 {
                draw_line_double(
                    &mut normal_context,
                    (maze_pos.0, ypos + 1),
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
                    &mut normal_context,
                    (maze_pos.0 + maze_render_size.0 - 1, ypos + 1),
                    if maze
                        .get_cell(Dims3D(maze.size().0 - 1, y, floor))
                        .unwrap()
                        .get_wall(CellWall::Bottom)
                    {
                        LineDir::ClosedRight
                    } else {
                        LineDir::Vertical
                    },
                );
            }

            if ypos + 1 < size.1 {
                draw_line_double(&mut normal_context, (maze_pos.0, ypos), LineDir::Vertical);

                draw_line_double(
                    &mut normal_context,
                    (
                        maze_pos.0 + maze_render_size.0 - 1,
                        y as i32 * 2 + maze_pos.1 + 1,
                    ),
                    LineDir::Vertical,
                );
            }
        }

        let moves = game.get_moves();
        // Drawing visited places (moves)
        for (move_pos, _) in moves {
            if move_pos.2 == floor {
                let real_pos = helpers::from_maze_to_real(*move_pos);
                normal_context.draw_char(Dims::from(maze_pos) + real_pos, '.');
            }
        }

        // Drawing maze itself
        for (iy, row) in maze.get_cells()[floor as usize].iter().enumerate() {
            let ypos = iy as i32 * 2 + 1 + maze_pos.1;
            if ypos >= size.1 - 2 {
                break;
            }

            for (ix, cell) in row.iter().enumerate() {
                let xpos = ix as i32 * 2 + 1 + maze_pos.0;
                if cell.get_wall(CellWall::Right) && ix != maze.size().0 as usize - 1 {
                    draw_line_double(&mut normal_context, (xpos + 1, ypos), LineDir::Vertical);
                }
                if ypos + 1 < size.1 as i32 - 2
                    && ypos > 0
                    && cell.get_wall(CellWall::Bottom)
                    && iy != maze.size().1 as usize - 1
                {
                    draw_line_double(&mut normal_context, (xpos, ypos + 1), LineDir::Horizontal);
                }

                Self::draw_stairs(
                    &mut normal_context,
                    &mut player_context,
                    &mut goal_context,
                    cell,
                    (ix as i32, iy as i32),
                    maze_pos,
                    floor,
                    player_pos,
                    ups_as_goal,
                );

                if iy == maze.size().1 as usize - 1 || ix == maze.size().0 as usize - 1 {
                    continue;
                }

                let cell2 = &maze.get_cells()[floor as usize][iy + 1][ix + 1];

                if ypos < size.1 as i32 - 3 && ypos > 0 {
                    draw_line_double(
                        &mut normal_context,
                        (xpos + 1, ypos + 1),
                        LineDir::double_line_bools(
                            cell.get_wall(CellWall::Bottom),
                            cell.get_wall(CellWall::Right),
                            cell2.get_wall(CellWall::Top),
                            cell2.get_wall(CellWall::Left),
                        ),
                    );
                }
            }
        }

        let goal_pos = game.get_goal_pos();
        if floor == goal_pos.2 {
            goal_context.draw_char(
                Dims::from(goal_pos) * 2 + maze_pos.into() + Dims(1, 1),
                constants::GOAL_CHAR,
            );
        }

        if floor == player_pos.2 {
            player_context.draw_char(
                Dims::from(player_pos) * 2 + maze_pos.into() + Dims(1, 1),
                *player_char,
            );

            Self::draw_stairs(
                &mut normal_context,
                &mut player_context,
                &mut goal_context,
                &maze.get_cells()[floor as usize][player_pos.1 as usize][player_pos.0 as usize],
                (player_pos.0, player_pos.1),
                maze_pos,
                floor,
                player_pos,
                ups_as_goal,
            );
        }

        let from_start = game.get_elapsed().unwrap();
        let texts = (
            &format!(
                "{}x{}x{}",
                game_state.game.get_player_pos().0 + 1,
                game_state.game.get_player_pos().1 + 1,
                game_state.game.get_player_pos().2 + 1
            ),
            match game_state.view_mode {
                GameViewMode::Adventure => "Adventure",
                GameViewMode::Spectator => "Spectator",
            },
            &format!("{} moves", game_state.game.get_move_count()),
            &ui::format_duration(from_start),
        );

        // Print texts
        let str_pos_tl = Dims(text_horizontal_margin, 0);
        let str_pos_tr = Dims(size.0 - text_horizontal_margin - texts.1.len() as i32, 0);
        let str_pos_bl = Dims(text_horizontal_margin, size.1 - 2);
        let str_pos_br = Dims::from(size) - Dims(text_horizontal_margin + texts.3.len() as i32, 2);

        text_context.draw_str(str_pos_tl, texts.0);
        text_context.draw_str(str_pos_tr, texts.1);
        text_context.draw_str(str_pos_bl, texts.2);
        text_context.draw_str(str_pos_br, texts.3);

        self.renderer.end(&mut self.stdout)?;

        Ok(())
    }

    fn generate_maze(&mut self, game_props: GameProperities) -> Result<Game, GameError> {
        let mut last_progress = f64::MIN;

        let msize = game_props.game_mode.size;
        let res = Game::new_threaded(game_props);

        let (handle, stop_flag, progress) = match res {
            Ok(com) => com,
            Err(GenerationErrorInstant::InvalidSize(dims)) => {
                ui::popup(
                    &mut self.renderer,
                    self.settings.get_color_scheme().normals(),
                    self.settings.get_color_scheme().texts(),
                    "Error",
                    &[
                        "Invalid maze size",
                        &format!(" {}x{}x{}", dims.0, dims.1, dims.2),
                    ],
                )?;
                return Err(GameError::EmptyMaze);
            }
        };

        for Progress { done, from } in progress.iter() {
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
                    self.settings.get_color_scheme().normals(),
                    self.settings.get_color_scheme().texts(),
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
            Ok(game) => Ok(game),
            Err(GenerationErrorThreaded::GenerationError(GenerationErrorInstant::InvalidSize(
                dims,
            ))) => {
                ui::popup(
                    &mut self.renderer,
                    self.settings.get_color_scheme().normals(),
                    self.settings.get_color_scheme().texts(),
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
    }

    fn draw_stairs<'a>(
        normal_context: &'a mut DrawContext,
        player_context: &'a mut DrawContext,
        goal_context: &'a mut DrawContext,
        cell: &Cell,
        stairs_pos: (i32, i32),
        maze_pos: (i32, i32),
        floor: i32,
        player_pos: Dims3D,
        ups_as_goal: bool,
    ) {
        let real_pos = helpers::from_maze_to_real(Dims3D(stairs_pos.0, stairs_pos.1, floor))
            + Dims::from(maze_pos);

        if !cell.get_wall(CellWall::Up) && !cell.get_wall(CellWall::Down) {
            if player_pos.2 == floor && Dims::from(player_pos) == stairs_pos.into() {
                player_context.draw_char(real_pos, '⥮');
            } else {
                normal_context.draw_char(real_pos, '⥮');
            };
        } else if !cell.get_wall(CellWall::Up) {
            if player_pos.2 == floor && Dims::from(player_pos) == stairs_pos.into() {
                player_context.draw_char(real_pos, '↑');
            } else if ups_as_goal {
                goal_context.draw_char(real_pos, '↑');
            } else {
                normal_context.draw_char(real_pos, '↑');
            }
        } else if !cell.get_wall(CellWall::Down) {
            if player_pos.2 == floor && Dims::from(player_pos) == stairs_pos.into() {
                player_context.draw_char(real_pos, '↓');
            } else {
                normal_context.draw_char(real_pos, '↓');
            }
        }
    }

    fn get_game_properities(&mut self) -> Result<GameProperities, GameError> {
        let mode = *ui::choice_menu(
            &mut self.renderer,
            self.settings.get_color_scheme().normals(),
            self.settings.get_color_scheme().texts(),
            "Maze size",
            &self
                .settings
                .get_mazes()
                .iter()
                .map(|maze| {
                    (
                        GameMode {
                            size: Dims3D(maze.width as i32, maze.height as i32, maze.depth as i32),
                            is_tower: maze.tower,
                        },
                        maze.title.as_str(),
                    )
                })
                .collect::<Vec<_>>(),
            self.settings
                .get_mazes()
                .iter()
                .position(|maze| maze.default),
            false,
        )?;

        let gen = if self.settings.get_dont_ask_for_maze_algo() {
            match self.settings.get_default_maze_gen_algo() {
                MazeGenAlgo::RandomKruskals => RndKruskals::generate,
                MazeGenAlgo::DepthFirstSearch => DepthFirstSearch::generate,
            }
        } else {
            match ui::menu(
                &mut self.renderer,
                self.settings.get_color_scheme().normals(),
                self.settings.get_color_scheme().texts(),
                "Maze generation algorithm",
                &["Randomized Kruskal's", "Depth-first search"],
                match self.settings.get_default_maze_gen_algo() {
                    MazeGenAlgo::RandomKruskals => Some(0),
                    MazeGenAlgo::DepthFirstSearch => Some(1),
                },
                true,
            )? {
                0 => RndKruskals::generate,
                1 => DepthFirstSearch::generate,
                _ => panic!(),
            }
        };

        Ok(GameProperities {
            game_mode: mode,
            generator: gen,
        })
    }
}

impl Default for App {
    fn default() -> Self {
        Self::new()
    }
}