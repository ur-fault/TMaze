use std::cell::RefCell;
use std::time::Duration;

use cmaze::core::*;
use cmaze::game::{Game, GameProperities, GameState as GameStatus};

use crossterm::event::{poll, read, Event, KeyCode, KeyEvent};
#[cfg(feature = "sound")]
use rodio::Source;

use crate::data::SaveData;
use crate::gameboard::CellWall;
use crate::gameboard::{algorithms::*, Cell};
use crate::helpers::{constants, value_if_else, LineDir};
use crate::renderer::helpers::term_size;
use crate::renderer::Renderer;
use crate::settings::{editable::EditableField, CameraMode, MazeGenAlgo, Settings};
use crate::ui::{DrawContext, Frame, MenuError};
use crate::{helpers, ui};

#[cfg(feature = "updates")]
use crate::updates;

#[cfg(feature = "sound")]
use crate::sound::{track::MusicTracks, SoundPlayer};

use super::{GameError, GameState, GameViewMode};

pub struct App {
    renderer: Renderer,
    #[cfg(feature = "sound")]
    sound_player: SoundPlayer,
    settings: Settings,
    save_data: SaveData,
    last_edge_follow_offset: Dims,
    last_selected_preset: Option<usize>,
    #[cfg(feature = "sound")]
    bgm_track: Option<MusicTracks>,
}

struct GameDrawContexts<'a> {
    normal: DrawContext<'a>,
    player: DrawContext<'a>,
    goal: DrawContext<'a>,
}

impl App {
    pub fn new() -> Self {
        let settings_path = Settings::default_path();
        App {
            renderer: Renderer::new().expect("Failed to initialize renderer"),
            #[cfg(feature = "sound")]
            sound_player: SoundPlayer::new(),
            settings: Settings::load(settings_path),
            save_data: SaveData::load_or(),
            last_edge_follow_offset: Dims(0, 0),
            last_selected_preset: None,
            #[cfg(feature = "sound")]
            bgm_track: None,
        }
    }

    #[cfg(feature = "sound")]
    fn play_bgm(&mut self, track: MusicTracks) {
        if let Some(prev_track) = self.bgm_track {
            if prev_track == track {
                return;
            }
        }

        if !self.settings.get_enable_audio() || !self.settings.get_enable_music() {
            return;
        }

        let volume = self.settings.get_audio_volume() * self.settings.get_music_volume();
        self.sound_player.sink().set_volume(volume);

        self.bgm_track = Some(track);
        self.sound_player
            .play_track(Box::new(track.get_track().repeat_infinite()));
    }

    #[cfg(feature = "updates")]
    fn check_for_updates(&mut self) -> Result<(), GameError> {
        use chrono::Local;
        use crossterm::event::{self, KeyEventKind};

        use crate::helpers::ToDebug;

        if !self.save_data.is_update_checked(&self.settings) {
            let last_check_before = self
                .save_data
                .last_update_check
                .map(|l| Local::now().signed_duration_since(l))
                .map(|d| d.to_std().expect("Failed to convert to std duration"))
                .map(|d| d - Duration::from_nanos(d.subsec_nanos() as u64)) // Remove subsec time
                .map(humantime::format_duration);

            let update_interval = format!(
                "Currently checkes {} for updates",
                self.settings.get_check_interval().to_debug().to_lowercase()
            );

            ui::popup::render_popup(
                &mut self.renderer,
                Default::default(),
                Default::default(),
                "Checking for newer version",
                &[
                    "Please wait...",
                    &update_interval,
                    &last_check_before
                        .map(|lc| format!("Last check before: {}", lc))
                        .unwrap_or("Never checked for updates".to_owned()),
                    "Press 'q' to cancel or Esc to skip",
                ],
            )?;

            let rt = tokio::runtime::Runtime::new().unwrap();

            let handle = rt.spawn(updates::get_newer_async());
            while !handle.is_finished() {
                if let Ok(true) = event::poll(Duration::from_millis(15)) {
                    match event::read() {
                        Ok(Event::Key(KeyEvent {
                            code: KeyCode::Char('q'),
                            kind: KeyEventKind::Press | KeyEventKind::Repeat,
                            ..
                        })) => {
                            handle.abort();
                            return Ok(());
                        }
                        Ok(Event::Key(KeyEvent {
                            code: KeyCode::Esc,
                            kind: KeyEventKind::Press | KeyEventKind::Repeat,
                            ..
                        })) => handle.abort(),
                        _ => (),
                    }
                }
            }

            match rt.block_on(handle).unwrap() {
                Ok(Some(version)) => {
                    ui::popup(
                        &mut self.renderer,
                        Default::default(),
                        Default::default(),
                        "New version available",
                        &[
                            format!("New version {} is available", version).as_str(),
                            format!("Your version is {}", env!("CARGO_PKG_VERSION")).as_str(),
                        ],
                    )?;
                }
                Err(err) if self.settings.get_display_update_check_errors() => {
                    ui::popup(
                        &mut self.renderer,
                        Default::default(),
                        Default::default(),
                        "Error while checking for updates",
                        &[
                            "There was an error while checking for updates",
                            &format!("Error: {}", err),
                        ],
                    )?;
                }
                _ => {}
            }

            self.save_data
                .update_last_check()
                .expect("Failed to save data");
        }

        Ok(())
    }

    pub fn run(mut self) -> Result<(), GameError> {
        #[cfg(feature = "updates")]
        self.check_for_updates()?;

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

            #[cfg(feature = "sound")]
            self.play_bgm(MusicTracks::Menu);

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
                        self.show_settings_screen()?;
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

        Ok(())
    }

    fn show_settings_screen(&mut self) -> Result<(), GameError> {
        let mut settings = self.settings.clone();
        settings.edit(
            &mut self.renderer,
            self.settings.read().color_scheme.clone().unwrap(),
        )?;
        self.settings = settings;
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

        #[cfg(feature = "sound")]
        self.play_bgm(MusicTracks::choose_for_maze(&game.get_maze()));

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
                        if game_state.handle_event(key_event).is_err() {
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
                        break Err(err.into());
                    }
                    _ => {}
                }

                self.renderer.on_event(&event.unwrap())?;
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
            let size = term_size();
            Dims(size.0 as i32, size.1 as i32)
        };

        let maze_margin = Dims(10, 3);

        let fits_on_screen = maze_render_size.0 + maze_margin.0 + 2 <= size.0
            && maze_render_size.1 + 3 + maze_margin.1 + 4 <= size.1;

        let maze_pos = {
            let pos = if fits_on_screen {
                ui::box_center_screen(maze_render_size)?
            } else {
                let last_player_real_pos = helpers::maze_pos_to_real(player_pos);

                match camera_mode {
                    CameraMode::CloseFollow => size / 2 - last_player_real_pos,
                    CameraMode::EdgeFollow(margin_x, margin_y) => {
                        let player_real_pos = self.last_edge_follow_offset + last_player_real_pos;

                        if player_real_pos.0 < margin_x + maze_margin.0 + 1
                            || player_real_pos.0 > size.0 - margin_x - maze_margin.1 - 1
                        {
                            self.last_edge_follow_offset.0 = size.0 / 2 - last_player_real_pos.0;
                        }

                        if player_real_pos.1 < margin_y + maze_margin.1 + 1
                            || player_real_pos.1 > size.1 - margin_y - maze_margin.1 - 1
                        {
                            self.last_edge_follow_offset.1 = size.1 / 2 - last_player_real_pos.1;
                        }
                        self.last_edge_follow_offset
                    }
                }
            };

            pos + Dims::from(*camera_offset) * 2
        };

        let normal_style = self.settings.get_color_scheme().normals();
        let text_style = self.settings.get_color_scheme().texts();
        let player_style = self.settings.get_color_scheme().players();
        let goal_style = self.settings.get_color_scheme().goals();

        // self.renderer.begin()?;
        let renderer_cell = RefCell::new(&mut self.renderer);

        let text_frame = if fits_on_screen {
            Frame::new_sized(maze_pos, maze_render_size - Dims(1, 1)).with_margin(Dims(-1, -2))
        } else {
            Frame::new_sized(Dims(0, 0), size).with_margin(maze_margin)
        };
        let frame = text_frame.with_margin(Dims(1, 2));

        let mut normal_context = DrawContext {
            renderer: &renderer_cell,
            style: normal_style,
            frame: frame.into(),
        };
        let mut text_context = DrawContext {
            renderer: &renderer_cell,
            style: text_style,
            frame: text_frame.into(),
        };
        let mut player_context = DrawContext {
            renderer: &renderer_cell,
            style: player_style,
            frame: frame.into(),
        };
        let mut goal_context = DrawContext {
            renderer: &renderer_cell,
            style: goal_style,
            frame: frame.into(),
        };

        let box_frame = text_frame.with_margin(Dims(0, 1));
        normal_context.draw_box(box_frame.start, box_frame.size());

        let floor = player_pos.2 + camera_offset.2;

        let draw_line_double_duo =
            |mut context: DrawContext, pos: (i32, i32), l1: LineDir, l2: LineDir| {
                context.draw_str(
                    pos.into(),
                    &format!("{}{}", l1.double_line(), l2.double_line(),),
                )
            };

        let draw_line_double = |mut context: DrawContext, pos: (i32, i32), l: LineDir| {
            context.draw_str(pos.into(), l.double_line())
        };

        draw_line_double_duo(
            normal_context,
            maze_pos.into(),
            LineDir::BottomRight,
            LineDir::Horizontal,
        );
        draw_line_double_duo(
            normal_context,
            (maze_pos.0 + maze_render_size.0 - 2, maze_pos.1),
            LineDir::Horizontal,
            LineDir::BottomLeft,
        );

        draw_line_double(
            normal_context,
            (maze_pos.0, maze_pos.1 + maze_render_size.1 - 2),
            LineDir::Vertical,
        );
        draw_line_double(
            normal_context,
            (
                maze_pos.0 + maze_render_size.0 - 1,
                maze_pos.1 + maze_render_size.1 - 2,
            ),
            LineDir::Vertical,
        );

        draw_line_double(
            normal_context,
            (maze_pos.0, maze_pos.1 + maze_render_size.1 - 1),
            LineDir::TopRight,
        );
        draw_line_double_duo(
            normal_context,
            (
                maze_pos.0 + maze_render_size.0 - 2,
                maze_pos.1 + maze_render_size.1 - 1,
            ),
            LineDir::Horizontal,
            LineDir::TopLeft,
        );

        for x in 0..maze.size().0 - 1 {
            draw_line_double_duo(
                normal_context,
                (x * 2 + maze_pos.0 + 1, maze_pos.1),
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

            draw_line_double_duo(
                normal_context,
                (x * 2 + maze_pos.0 + 1, maze_pos.1 + maze_render_size.1 - 1),
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

        // Vertical edge lines
        for y in 0..maze.size().1 - 1 {
            let ypos = y * 2 + maze_pos.1 + 1;
            if ypos >= size.1 - 2 {
                break;
            }

            if ypos == -1 {
                continue;
            }

            draw_line_double(
                normal_context,
                (maze_pos.0, ypos + 1),
                value_if_else(
                    maze.get_cell(Dims3D(0, y, floor))
                        .unwrap()
                        .get_wall(CellWall::Bottom),
                    || LineDir::ClosedLeft,
                    || LineDir::Vertical,
                ),
            );

            draw_line_double(
                normal_context,
                (maze_pos.0 + maze_render_size.0 - 1, ypos + 1),
                value_if_else(
                    maze.get_cell(Dims3D(maze.size().0 - 1, y, floor))
                        .unwrap()
                        .get_wall(CellWall::Bottom),
                    || LineDir::ClosedRight,
                    || LineDir::Vertical,
                ),
            );

            draw_line_double(normal_context, (maze_pos.0, ypos), LineDir::Vertical);

            draw_line_double(
                normal_context,
                (maze_pos.0 + maze_render_size.0 - 1, y * 2 + maze_pos.1 + 1),
                LineDir::Vertical,
            );
        }

        // Drawing visited places (moves)
        let moves = game.get_moves();
        for (move_pos, _) in moves {
            if move_pos.2 == floor {
                let real_pos = helpers::maze_pos_to_real(*move_pos);
                normal_context.draw_char(maze_pos + real_pos, '.');
            }
        }

        // Drawing insides of the maze itself
        for (iy, row) in maze.get_cells()[floor as usize].iter().enumerate() {
            let ypos = iy as i32 * 2 + 1 + maze_pos.1;

            for (ix, cell) in row.iter().enumerate() {
                let xpos = ix as i32 * 2 + 1 + maze_pos.0;
                if cell.get_wall(CellWall::Right) && ix != maze.size().0 as usize - 1 {
                    draw_line_double(normal_context, (xpos + 1, ypos), LineDir::Vertical);
                }

                if ypos + 1 < size.1 - 2
                    && ypos > 0
                    && cell.get_wall(CellWall::Bottom)
                    && iy != maze.size().1 as usize - 1
                {
                    draw_line_double(normal_context, (xpos, ypos + 1), LineDir::Horizontal);
                }

                let contexts = GameDrawContexts {
                    normal: normal_context,
                    player: player_context,
                    goal: goal_context,
                };

                Self::draw_stairs(
                    contexts,
                    cell,
                    (ix as i32, iy as i32),
                    maze_pos.into(),
                    floor,
                    player_pos,
                    ups_as_goal,
                );

                let cell2 = match maze.get_cell(Dims3D(ix as i32 + 1, iy as i32 + 1, floor)) {
                    Some(cell) => cell,
                    None => continue,
                };

                draw_line_double(
                    normal_context,
                    (xpos + 1, ypos + 1),
                    LineDir::from_bools(
                        cell.get_wall(CellWall::Bottom),
                        cell.get_wall(CellWall::Right),
                        cell2.get_wall(CellWall::Top),
                        cell2.get_wall(CellWall::Left),
                    ),
                );
            }
        }

        let goal_pos = game.get_goal_pos();
        if floor == goal_pos.2 {
            goal_context.draw_char(
                Dims::from(goal_pos) * 2 + maze_pos + Dims(1, 1),
                constants::GOAL_CHAR,
            );
        }

        if floor == player_pos.2 {
            player_context.draw_char(
                Dims::from(player_pos) * 2 + maze_pos + Dims(1, 1),
                *player_char,
            );

            let contexts = GameDrawContexts {
                normal: normal_context,
                player: player_context,
                goal: goal_context,
            };

            Self::draw_stairs(
                contexts,
                maze.get_cell(player_pos).unwrap(),
                (player_pos.0, player_pos.1),
                maze_pos.into(),
                floor,
                player_pos,
                ups_as_goal,
            );
        }

        let pos_text = if maze.size().2 > 1 {
            format!(
                "x:{} y:{} floor:{}",
                player_pos.0 + 1,
                player_pos.1 + 1,
                player_pos.2 + 1
            )
        } else {
            format!("x:{} y:{}", player_pos.0 + 1, player_pos.1 + 1)
        };

        let from_start = game.get_elapsed().unwrap();
        let view_mode = game_state.view_mode.to_string();
        let (view_mode, pos_text) =
            if view_mode.len() as i32 + text_horizontal_margin * 2 + pos_text.len() as i32 + 1
                > text_frame.size().0
            {
                (
                    format!("{}", view_mode.chars().next().unwrap()),
                    format!(
                        "x:{} y:{} f:{}",
                        player_pos.0 + 1,
                        player_pos.1 + 1,
                        player_pos.2 + 1
                    ),
                )
            } else {
                (view_mode, pos_text)
            };

        let texts = (
            &pos_text,
            view_mode.as_str(),
            &format!("{} moves", game_state.game.get_move_count()),
            &ui::format_duration(from_start),
        );

        // Print texts
        let str_pos_tl = Dims(
            text_horizontal_margin + text_frame.start.0,
            text_frame.start.1,
        );
        let str_pos_tr = Dims(
            text_frame.end.0 - text_horizontal_margin - texts.1.len() as i32 + 1,
            text_frame.start.1,
        );
        let str_pos_bl = Dims(
            text_horizontal_margin + text_frame.start.0,
            text_frame.end.1,
        );
        let str_pos_br =
            text_frame.end - Dims(text_horizontal_margin + texts.3.len() as i32 - 1, 0);

        text_context.draw_str(str_pos_tl, texts.0);
        text_context.draw_str(str_pos_tr, texts.1);
        text_context.draw_str(str_pos_bl, texts.2);
        text_context.draw_str(str_pos_br, texts.3);

        self.renderer.render()?;

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
                return Err(GameError::EmptyMenu);
            }
        };

        for Progress { done, from } in progress.iter() {
            let current_progress = done as f64 / from as f64;

            if let Ok(true) = poll(Duration::from_nanos(1)) {
                if let Ok(Event::Key(KeyEvent { code, .. })) = read() {
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
                Err(GameError::EmptyMenu)
            }
            Err(GenerationErrorThreaded::AbortGeneration) => Err(GameError::Back),
            Err(GenerationErrorThreaded::UnknownError(err)) => panic!("{:?}", err),
        }
    }

    fn draw_stairs(
        contexts: GameDrawContexts,
        cell: &Cell,
        stairs_pos: (i32, i32),
        maze_pos: (i32, i32),
        floor: i32,
        player_pos: Dims3D,
        ups_as_goal: bool,
    ) {
        let real_pos = helpers::maze_pos_to_real(Dims3D(stairs_pos.0, stairs_pos.1, floor))
            + Dims::from(maze_pos);

        let GameDrawContexts {
            normal: mut normal_context,
            player: mut player_context,
            goal: mut goal_context,
        } = contexts;

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
        let (i, &mode) = ui::choice_menu(
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
            self.last_selected_preset.or_else(|| {
                self.settings
                    .get_mazes()
                    .iter()
                    .position(|maze| maze.default)
            }),
            false,
        )?;

        self.last_selected_preset = Some(i);

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
