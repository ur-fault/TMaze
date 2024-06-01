use cmaze::{
    core::{Dims, Dims3D, GameMode},
    game::{GameProperities, GeneratorFn, ProgressComm, RunningGame, RunningGameState},
    gameboard::{
        algorithms::{
            DepthFirstSearch, GenErrorInstant, GenErrorThreaded, MazeAlgorithm, Progress,
            RndKruskals,
        },
        Cell, CellWall,
    },
};

use crate::{
    app::{game_state::GameData, GameViewMode},
    helpers::{constants, is_release, maze2screen, maze2screen_3d, maze_render_size, LineDir},
    lerp, menu_actions,
    renderer::Frame,
    settings::{self, CameraMode, ColorScheme, Offset, Settings, SettingsActivity},
    ui::{
        self, draw_box, multisize_string, split_menu_actions, Menu, MenuAction, MenuConfig, Popup,
        ProgressBar, Screen,
    },
};

#[cfg(feature = "sound")]
#[allow(unused_imports)]
use crate::sound::{track::MusicTrack, SoundPlayer};

#[cfg(feature = "updates")]
#[allow(unused_imports)]
use crate::updates::UpdateCheckerActivity;

use crossterm::event::{Event as TermEvent, KeyCode, KeyEvent};

#[cfg(feature = "sound")]
#[allow(unused_imports)]
use rodio::Source;

use super::{
    app::{AppData, AppStateData},
    Activity, ActivityHandler, Change, Event,
};

pub fn create_controls_popup(settings: &Settings) -> Activity {
    let popup = Popup::new(
        "Controls".to_string(),
        [
            "~ In game",
            " WASD and arrows: move",
            " Space: switch adventure/spectaror mode",
            " Q, F or L: move down",
            " E, R or P: move up",
            " With SHIFT move at the end in single dir",
            " Escape: pause menu",
            "",
            "~ In end game popup",
            " Enter or space: main menu",
            " Q: quit TMaze",
            " R: restart game",
        ]
        .into_iter()
        .map(String::from)
        .collect::<Vec<_>>(),
    )
    .styles_from_settings(settings);

    Activity::new_base("controls".to_string(), Box::new(popup))
}

pub struct MainMenu {
    menu: Menu,
    actions: Vec<MenuAction<Change>>,

    #[cfg(feature = "updates")]
    update_checked: bool,
}

impl MainMenu {
    pub fn new(settings: &Settings) -> Self {
        let options = menu_actions!(
            "New Game" -> data => Self::start_new_game(&data.settings, &data.use_data),
            "Settings" -> data => Self::show_settings_screen(&data.settings),
            "Controls" -> data => Self::show_controls_popup(&data.settings),
            "About" -> data => Self::show_about_popup(&data.settings),
            "Quit" -> _ => Change::pop_top(),
        );

        let (options, actions) = split_menu_actions(options);

        Self {
            menu: Menu::new(
                MenuConfig::new_from_strings("TMaze", options)
                    .counted()
                    .styles_from_settings(settings),
            ),
            actions,

            #[cfg(feature = "updates")]
            update_checked: false,
        }
    }

    fn show_settings_screen(settings: &Settings) -> Change {
        Change::push(Activity::new_base(
            "settings".to_string(),
            Box::new(settings::SettingsActivity::new(settings)),
        ))
    }

    fn show_controls_popup(settings: &Settings) -> Change {
        Change::push(create_controls_popup(settings))
    }

    fn show_about_popup(settings: &Settings) -> Change {
        let popup = Popup::new(
            "About".to_string(),
            vec![
                "This is simple maze solving game".to_string(),
                "Supported algorithms:".to_string(),
                "    - Depth-first search".to_string(),
                "    - Kruskal's algorithm".to_string(),
                "Supports 3D mazes".to_string(),
                "".to_string(),
                "Created by:".to_string(),
                format!("    - {}", env!("CARGO_PKG_AUTHORS")),
                "".to_string(),
                "Version:".to_string(),
                format!("    {}", env!("CARGO_PKG_VERSION")),
            ],
        )
        .styles_from_settings(settings);

        Change::push(Activity::new_base("about".to_string(), Box::new(popup)))
    }

    fn start_new_game(settings: &Settings, use_data: &AppStateData) -> Change {
        Change::push(Activity::new_base(
            "maze size",
            Box::new(MazeSizeMenu::new(settings, use_data)),
        ))
    }

    #[cfg(feature = "sound")]
    fn play_menu_bgm(data: &mut AppData) {
        data.play_bgm(MusicTrack::Menu);
    }
}

impl ActivityHandler for MainMenu {
    fn update(&mut self, events: Vec<super::Event>, data: &mut AppData) -> Option<Change> {
        #[cfg(feature = "sound")]
        Self::play_menu_bgm(data);

        #[cfg(feature = "updates")]
        if !self.update_checked {
            self.update_checked = true;
            return Some(Change::push(Activity::new_base(
                "update check",
                Box::new(UpdateCheckerActivity::new(&data.settings, &data.save)),
            )));
        }

        match self.menu.update(events, data)? {
            Change::Pop {
                res: Some(sub_activity),
                ..
            } => {
                let index = *sub_activity
                    .downcast::<usize>()
                    .expect("menu should return index");
                Some(self.actions[index](data))
            }
            res => Some(res),
        }
    }

    fn screen(&self) -> &dyn ui::Screen {
        &self.menu
    }
}

pub struct MazeSizeMenu {
    menu: Menu,
    presets: Vec<GameMode>,
}

impl MazeSizeMenu {
    pub fn new(settings: &Settings, app_state_data: &AppStateData) -> Self {
        let color_scheme = settings.get_color_scheme();
        let mut menu_config = MenuConfig::new_from_strings(
            "Maze size".to_string(),
            settings
                .get_mazes()
                .iter()
                .map(|maze| maze.title.clone())
                .collect::<Vec<_>>(),
        )
        .box_style(color_scheme.normals())
        .text_style(color_scheme.texts());

        let default = app_state_data
            .last_selected_preset
            .or_else(|| settings.get_mazes().iter().position(|maze| maze.default));

        if let Some(i) = default {
            menu_config = menu_config.default(i);
        }

        let menu = Menu::new(menu_config);

        let presets = settings
            .get_mazes()
            .iter()
            .map(|maze| GameMode {
                size: Dims3D(maze.width as i32, maze.height as i32, maze.depth as i32),
                is_tower: maze.tower,
            })
            .collect::<Vec<_>>();

        Self { menu, presets }
    }

    // TODO: custom maze size config
    // just one-time, since it's already in settings
}

impl ActivityHandler for MazeSizeMenu {
    fn update(&mut self, events: Vec<super::Event>, data: &mut AppData) -> Option<Change> {
        match self.menu.update(events, data) {
            Some(change) => match change {
                Change::Pop {
                    res: Some(size), ..
                } => {
                    let index = *size.downcast::<usize>().expect("menu should return index");
                    data.use_data.last_selected_preset = Some(index);

                    let preset = self.presets[index];

                    Some(Change::push(Activity::new_base(
                        "maze_gen".to_string(),
                        Box::new(MazeAlgorithmMenu::new(preset, &data.settings)),
                    )))
                }
                res => Some(res),
            },
            None => None,
        }
    }

    fn screen(&self) -> &dyn ui::Screen {
        &self.menu
    }
}

pub struct MazeAlgorithmMenu {
    preset: GameMode,
    menu: Menu,
    functions: Vec<MenuAction<GeneratorFn>>,
}

impl MazeAlgorithmMenu {
    pub fn new(preset: GameMode, settings: &Settings) -> Self {
        let options = menu_actions!(
            "Randomized Kruskal's" -> _ => RndKruskals::generate as GeneratorFn,
            "Depth-first search" -> _ => DepthFirstSearch::generate,
        );

        let (options, functions) = split_menu_actions(options);

        let menu_config =
            MenuConfig::new_from_strings("Maze generation algorithm".to_string(), options)
                .counted()
                .styles_from_settings(settings)
                .maybe_default(settings.read().default_maze_gen_algo.map(|a| a as usize));

        let menu = Menu::new(menu_config);

        Self {
            menu,
            preset,
            functions,
        }
    }
}

impl ActivityHandler for MazeAlgorithmMenu {
    fn update(&mut self, events: Vec<super::Event>, data: &mut AppData) -> Option<Change> {
        if data.settings.get_dont_ask_for_maze_algo() {
            return Some(Change::push(Activity::new_base(
                "maze_gen".to_string(),
                Box::new(MazeGenerationActivity::new(
                    self.preset,
                    data.settings.get_default_maze_gen_algo().to_fn(),
                    &data.settings,
                )),
            )));
        }

        match self.menu.update(events, data) {
            Some(change) => match change {
                Change::Pop {
                    res: Some(algo), ..
                } => {
                    let index = *algo.downcast::<usize>().expect("menu should return index");

                    let gen = self.functions[index](data);

                    Some(Change::push(Activity::new_base(
                        "maze_gen".to_string(),
                        Box::new(MazeGenerationActivity::new(
                            self.preset,
                            gen,
                            &data.settings,
                        )),
                    )))
                }
                res => Some(res),
            },
            None => None,
        }
    }

    fn screen(&self) -> &dyn ui::Screen {
        &self.menu
    }
}

pub struct MazeGenerationActivity {
    comm: Option<ProgressComm<Result<RunningGame, GenErrorThreaded>>>,
    game_props: GameProperities,
    progress_bar: ProgressBar,
}

impl MazeGenerationActivity {
    pub fn new(game_mode: GameMode, maze_gen: GeneratorFn, settings: &Settings) -> Self {
        let game_props = GameProperities {
            game_mode,
            generator: maze_gen,
        };

        let color_scheme = settings.get_color_scheme();
        let progress_bar = ProgressBar::new(format!("Generating maze: {:?}", game_mode.size))
            .box_style(color_scheme.normals())
            .text_style(color_scheme.texts());

        Self {
            comm: None,
            game_props,
            progress_bar,
        }
    }
}

impl ActivityHandler for MazeGenerationActivity {
    fn update(&mut self, events: Vec<super::Event>, data: &mut AppData) -> Option<Change> {
        for event in events {
            match event {
                Event::Term(TermEvent::Key(KeyEvent { code, kind, .. })) if !is_release(kind) => {
                    match code {
                        KeyCode::Esc | KeyCode::Char('q') => {
                            if let Some(comm) = self.comm.take() {
                                comm.stop_flag.stop();
                                let _ = comm.handle.join().unwrap();
                            };
                            return Some(Change::pop(2));
                        }
                        _ => {}
                    }
                }
                _ => {}
            }
        }

        match self.comm {
            None => match RunningGame::new_threaded(self.game_props.clone()) {
                Ok(comm) => {
                    log::info!("Maze generation thread started");
                    self.comm = Some(comm);

                    None
                }
                Err(err) => match err {
                    GenErrorInstant::InvalidSize(size) => {
                        let popup = Popup::new(
                            "Invalid maze size".to_string(),
                            vec![format!("Size: {:?}", size)],
                        );

                        Some(Change::replace(Activity::new_base(
                            "invalid size".to_string(),
                            Box::new(popup),
                        )))
                    }
                },
            },

            Some(ref comm) if comm.handle.is_finished() => {
                let res = self
                    .comm
                    .take()
                    .unwrap()
                    .handle
                    .join()
                    .expect("Could not join maze generation thread");

                match res {
                    Ok(game) => {
                        let game_data = GameData {
                            camera_pos: maze2screen_3d(game.get_player_pos()),
                            game,
                            view_mode: GameViewMode::Adventure,
                            player_char: constants::get_random_player_char(),
                        };
                        Some(Change::replace(Activity::new_base(
                            "game".to_string(),
                            Box::new(GameActivity::new(game_data, data)),
                        )))
                    }
                    Err(err) => match err {
                        GenErrorThreaded::AbortGeneration => Some(Change::pop_top()),
                        GenErrorThreaded::GenerationError(_) => {
                            panic!("Instant generation error should be handled before");
                        }
                    },
                }
            }

            Some(ref comm) => {
                let Progress { done, from, .. } = comm.progress();
                self.progress_bar.update_progress(done as f64 / from as f64);
                self.progress_bar.update_title(format!(
                    "Generating maze: {}/{} - {:.2} %",
                    done,
                    from,
                    done as f64 / from as f64 * 100.0
                ));
                None
            }
        }
    }

    fn screen(&self) -> &dyn ui::Screen {
        &self.progress_bar
    }
}

pub struct PauseMenu {
    menu: Menu,
    actions: Vec<MenuAction<Change>>,
}

impl PauseMenu {
    pub fn new(settings: &Settings) -> Self {
        let options = menu_actions!(
            "Resume" -> _ => Change::pop_top(),
            "Main Menu" -> _ => Change::pop_until("main menu"),
            "Controls" -> data => Change::push(create_controls_popup(&data.settings)),
            "Settings" -> data => Change::push(SettingsActivity::new_activity(&data.settings)),
            "Quit" -> _ => Change::pop_all(),
        );

        let (options, actions) = split_menu_actions(options);

        let menu = Menu::new(
            MenuConfig::new_from_strings("Paused", options).styles_from_settings(settings),
        );

        Self { menu, actions }
    }
}

impl ActivityHandler for PauseMenu {
    fn update(&mut self, events: Vec<Event>, data: &mut AppData) -> Option<Change> {
        match self.menu.update(events, data) {
            Some(change) => match change {
                Change::Pop { res: Some(res), .. } => {
                    let index = *res.downcast::<usize>().expect("menu should return index");

                    Some((self.actions[index])(data))
                }
                res => Some(res),
            },
            None => None,
        }
    }

    fn screen(&self) -> &dyn Screen {
        &self.menu
    }
}

pub struct EndGamePopup {
    popup: Popup,
    game_mode: GameMode,
    gen_fn: GeneratorFn,
}

impl EndGamePopup {
    pub fn new(game: &RunningGame, settings: &Settings) -> Self {
        let maze_size = game.get_maze().size();
        let texts = vec![
            format!(
                "Time:  {}",
                ui::format_duration(game.get_elapsed().unwrap())
            ),
            format!("Moves: {}", game.get_move_count()),
            format!("Size:  {}x{}x{}", maze_size.0, maze_size.1, maze_size.2,),
        ];

        let popup = Popup::new("You won".to_string(), texts).styles_from_settings(settings);

        let game_mode = game.get_game_mode();
        let gen_fn = game.get_gen_fn();

        Self {
            popup,
            game_mode,
            gen_fn,
        }
    }
}

impl ActivityHandler for EndGamePopup {
    fn update(&mut self, events: Vec<Event>, data: &mut AppData) -> Option<Change> {
        match self.popup.update(events, data) {
            Some(Change::Pop {
                n: 1,
                res: Some(code),
            }) => match code.downcast::<KeyCode>() {
                Ok(b) => match *b {
                    KeyCode::Char('r') => Some(Change::replace(Activity::new_base(
                        "game",
                        Box::new(MazeGenerationActivity::new(
                            self.game_mode,
                            self.gen_fn,
                            &data.settings,
                        )),
                    ))),
                    KeyCode::Char('q') => Some(Change::pop_all()),
                    KeyCode::Enter | KeyCode::Char(' ') => Some(Change::pop_top()),
                    _ => None,
                },
                _ => panic!("expected `KeyCode` from `Popup`"),
            },
            res => res,
        }
    }

    fn screen(&self) -> &dyn Screen {
        &self.popup
    }
}

pub struct GameActivity {
    camera_mode: CameraMode,
    game: GameData,
    maze_board: MazeBoard,
    show_debug: bool,

    // smooth
    sm_camera_pos: Dims3D,
    sm_player_pos: Dims3D,
}

impl GameActivity {
    pub fn new(game: GameData, app_data: &mut AppData) -> Self {
        let settings = &app_data.settings;
        let camera_mode = settings.get_camera_mode();
        let maze_board = MazeBoard::new(&game.game, settings);

        #[cfg(feature = "sound")]
        app_data.play_bgm(MusicTrack::choose_for_maze(game.game.get_maze()));

        let sm_camera_pos = game.camera_pos;
        let sm_player_pos = maze2screen_3d(game.game.get_player_pos());

        Self {
            camera_mode,
            game,
            maze_board,
            show_debug: false,

            sm_camera_pos,
            sm_player_pos,
        }
    }

    /// Returns the size of the viewport and whether the floor fits in the viewport
    pub fn viewport_size(&self, screen_size: Dims) -> (Dims, bool) {
        let vp_size = screen_size - Dims(8, 6);

        let maze_frame = &self.maze_board.frames[self.game.game.get_player_pos().2 as usize];
        let floor_size = maze_frame.size;

        let does_fit = floor_size.0 <= vp_size.0 && floor_size.1 <= vp_size.1;
        if does_fit {
            (floor_size, does_fit)
        } else {
            (vp_size, does_fit)
        }
    }

    fn current_floor_frame(&self) -> &Frame {
        &self.maze_board.frames[self.game.camera_pos.2 as usize]
    }

    fn render_meta_texts(
        &self,
        frame: &mut Frame,
        color_scheme: &ColorScheme,
        vp_pos: Dims,
        vp_size: Dims,
    ) {
        let max_width = (vp_size.0 / 2 + 1) as usize;

        let pl_pos = self.game.game.get_player_pos() + Dims3D(1, 1, 1);

        // texts
        let from_start =
            ui::multisize_duration_format(self.game.game.get_elapsed().unwrap(), max_width);
        let move_count = ui::multisize_string(
            [
                format!("{} moves", self.game.game.get_move_count()),
                format!("{}m", self.game.game.get_move_count()),
            ],
            max_width,
        );

        let pos_text = if self.game.game.get_maze().size().2 > 1 {
            multisize_string(
                [
                    format!("x:{} y:{} floor:{}", pl_pos.0, pl_pos.1, pl_pos.2),
                    format!("x:{} y:{} f:{}", pl_pos.0, pl_pos.1, pl_pos.2),
                    format!("{}:{}:{}", pl_pos.0, pl_pos.1, pl_pos.2),
                ],
                max_width,
            )
        } else {
            multisize_string(
                [
                    format!("x:{} y:{}", pl_pos.0, pl_pos.1),
                    format!("x:{} y:{}", pl_pos.0, pl_pos.1),
                    format!("{}:{}", pl_pos.0, pl_pos.1),
                ],
                max_width,
            )
        };

        let view_mode = self.game.view_mode;
        let view_mode = multisize_string(view_mode.to_multisize_strings(), max_width);

        let tl = vp_pos - Dims(1, 2);
        let br = vp_pos + vp_size + Dims(1, 1);

        // draw them
        let mut draw = |text: &str, pos| frame.draw_styled(pos, text, color_scheme.texts());

        draw(&pos_text, tl);
        draw(&view_mode, Dims(br.0 - view_mode.len() as i32, tl.1));
        draw(&move_count, Dims(tl.0, br.1));
        draw(&from_start, Dims(br.0 - from_start.len() as i32, br.1));
    }

    pub fn render_visited_places(
        &self,
        frame: &mut Frame,
        maze_pos: Dims,
        color_scheme: &ColorScheme,
    ) {
        use CellWall::{Down, Up};

        let game = &self.game.game;
        for (move_pos, _) in game.get_moves() {
            let cell = game.get_maze().get_cell(*move_pos).unwrap();
            if move_pos.2 == game.get_player_pos().2 && cell.get_wall(Up) && cell.get_wall(Down) {
                let real_pos = maze2screen(*move_pos) + maze_pos;
                frame.draw_styled(real_pos, '.', color_scheme.normals());
            }
        }
    }
}

impl ActivityHandler for GameActivity {
    fn update(&mut self, events: Vec<Event>, data: &mut AppData) -> Option<Change> {
        match self.game.game.get_state() {
            RunningGameState::NotStarted => self.game.game.start().unwrap(),
            RunningGameState::Paused => self.game.game.resume().unwrap(),
            _ => {}
        }

        for event in events {
            if let Event::Term(TermEvent::Key(key_event)) = event {
                match self.game.handle_event(&data.settings, key_event) {
                    Err(false) => {
                        self.game.game.pause().unwrap();

                        return Some(Change::push(Activity::new_base(
                            "pause".to_string(),
                            Box::new(PauseMenu::new(&data.settings)),
                        )));
                    }
                    Err(true) => return Some(Change::pop_until("main menu")),
                    Ok(_) => {}
                }
            }
        }

        if self.game.view_mode == GameViewMode::Adventure {
            match self.camera_mode {
                CameraMode::CloseFollow => {
                    self.game.camera_pos = maze2screen_3d(self.game.game.get_player_pos());
                }
                CameraMode::EdgeFollow(xoff, yoff) => 'b: {
                    let (vp_size, does_fit) = self.viewport_size(data.screen_size);
                    if does_fit {
                        break 'b;
                    }

                    let xoff = xoff.to_abs(vp_size.0);
                    let yoff = yoff.to_abs(vp_size.1);

                    let player_pos = maze2screen(self.game.game.get_player_pos());
                    let player_pos_in_vp =
                        player_pos - self.game.camera_pos.into() + vp_size / 2 + Dims(1, 1);

                    if player_pos_in_vp.0 < xoff || player_pos_in_vp.0 > vp_size.0 - xoff {
                        self.game.camera_pos.0 = player_pos.0;
                    }

                    if player_pos_in_vp.1 < yoff || player_pos_in_vp.1 > vp_size.1 - yoff {
                        self.game.camera_pos.1 = player_pos.1;
                    }
                }
            }
        }

        self.sm_player_pos = lerp!((self.sm_player_pos) -> (maze2screen_3d(self.game.game.get_player_pos())) at data.settings.get_player_smoothing());
        self.sm_camera_pos = lerp!((self.sm_camera_pos) -> (self.game.camera_pos) at data.settings.get_camera_smoothing());

        self.show_debug = data.use_data.show_debug;

        if self.game.game.get_state() == RunningGameState::Finished {
            return Some(Change::replace_at(
                1,
                Activity::new_base(
                    "won".to_string(),
                    Box::new(EndGamePopup::new(&self.game.game, &data.settings)),
                ),
            ));
        };

        None
    }

    fn screen(&self) -> &dyn ui::Screen {
        self
    }
}

impl Screen for GameActivity {
    fn draw(&self, frame: &mut Frame, color_scheme: &ColorScheme) -> std::io::Result<()> {
        let maze_frame = self.current_floor_frame();
        let game = &self.game.game;

        let (vp_size, does_fit) = self.viewport_size(frame.size);
        let maze_pos = match does_fit {
            true => match self.game.view_mode {
                GameViewMode::Adventure => Dims(0, 0),
                GameViewMode::Spectator => maze2screen(Dims(0, 0)) - self.sm_camera_pos.into(),
            },
            false => vp_size / 2 - self.sm_camera_pos.into(),
        };

        // TODO: reuse the viewport between frames and resize it when needed
        let mut viewport = Frame::new(vp_size);

        // maze
        viewport.draw(maze_pos, maze_frame);
        self.render_visited_places(&mut viewport, maze_pos, color_scheme);

        // player
        if (self.game.game.get_player_pos().2) == self.sm_camera_pos.2 {
            let player = self.sm_player_pos;
            let player_draw_pos = maze_pos + player.into();
            let cell = game
                .get_maze()
                .get_cell(self.game.game.get_player_pos())
                .unwrap();
            if !cell.get_wall(CellWall::Up) || !cell.get_wall(CellWall::Down) {
                viewport[player_draw_pos]
                    .content_mut()
                    .unwrap()
                    .style
                    .foreground_color = Some(color_scheme.player);
            } else {
                viewport.draw_styled(
                    player_draw_pos,
                    self.game.player_char,
                    color_scheme.players(),
                );
            }
        }

        let vp_pos = (frame.size - vp_size) / 2;
        draw_box(
            frame,
            vp_pos - Dims(1, 1),
            vp_size + Dims(2, 2),
            color_scheme.normals(),
        );

        if let CameraMode::EdgeFollow(xoff, yoff) = self.camera_mode {
            if !does_fit && self.show_debug {
                render_edge_follow_rulers((xoff, yoff), frame, vp_size, vp_pos, color_scheme);
            }
        }

        self.render_meta_texts(frame, color_scheme, vp_pos, vp_size);

        frame.draw(vp_pos, &viewport);

        Ok(())
    }
}

#[inline]
fn render_edge_follow_rulers(
    rulers: (Offset, Offset),
    frame: &mut Frame,
    vps: Dims,
    vp_pos: Dims,
    color_scheme: &ColorScheme,
) {
    // for future use: ['↑', '↓', '←, '→']

    let goals = color_scheme.goals();
    let players = color_scheme.players();

    let xo = rulers.0.to_abs(vps.0);
    let yo = rulers.1.to_abs(vps.1);

    use LineDir::{Horizontal, Vertical};
    const V: char = Vertical.round();
    const H: char = Horizontal.round();

    let mut draw = |pos, dir, end| {
        frame.draw_styled(
            (vp_pos - Dims(1, 1)) + pos,
            dir,
            match end {
                false => goals,
                true => players,
            },
        )
    };

    #[rustfmt::skip]
    {
        draw(Dims(xo        , 0)         , V, false);
        draw(Dims(vps.0 - xo, 0)         , V, true);
        draw(Dims(xo        , vps.1 + 1) , V, false);
        draw(Dims(vps.0 - xo, vps.1 + 1) , V, true);

        draw(Dims(0         , yo)        , H, false);
        draw(Dims(0         , vps.1 - yo), H, true);
        draw(Dims(vps.0 + 1 , yo)        , H, false);
        draw(Dims(vps.0 + 1 , vps.1 - yo), H, true);
    };
}

pub struct MazeBoard {
    frames: Vec<Frame>,
}

impl MazeBoard {
    pub fn new(game: &RunningGame, settings: &Settings) -> Self {
        let maze = game.get_maze();
        let scheme = settings.get_color_scheme();

        let mut frames: Vec<_> = (0..maze.size().2)
            .map(|floor| Self::render_floor(game, floor, scheme.clone()))
            .collect();

        Self::render_special(&mut frames, game, scheme.clone());

        Self { frames }
    }

    fn render_floor(game: &RunningGame, floor: i32, scheme: ColorScheme) -> Frame {
        let maze = game.get_maze();
        let normals = scheme.normals();

        let size = maze_render_size(maze);

        let mut frame = Frame::new(size);

        let mut draw = |pos, l: LineDir| frame.draw_styled(Dims::from(pos), l.double(), normals);

        for y in -1..maze.size().1 {
            for x in -1..maze.size().0 {
                let cell_pos = Dims3D(x, y, floor);
                let Dims(rx, ry) = maze2screen(cell_pos);

                if maze.get_wall(cell_pos, CellWall::Right).unwrap() {
                    draw((rx + 1, ry), LineDir::Vertical);
                }

                if maze.get_wall(cell_pos, CellWall::Bottom).unwrap() {
                    draw((rx, ry + 1), LineDir::Horizontal);
                }

                let cp1 = cell_pos;
                let cp2 = cell_pos + Dims3D(1, 1, 0);

                let dir = LineDir::from_bools(
                    maze.get_wall(cp1, CellWall::Bottom).unwrap(),
                    maze.get_wall(cp1, CellWall::Right).unwrap(),
                    maze.get_wall(cp2, CellWall::Top).unwrap(),
                    maze.get_wall(cp2, CellWall::Left).unwrap(),
                );

                draw((rx + 1, ry + 1), dir);
            }
        }

        let cells = &maze.get_cells()[floor as usize];
        Self::render_stairs(&mut frame, cells, maze.is_tower(), scheme);

        frame
    }

    fn render_stairs(frame: &mut Frame, floors: &[Vec<Cell>], tower: bool, scheme: ColorScheme) {
        let style = if tower {
            scheme.goals()
        } else {
            scheme.normals()
        };

        for (y, row) in floors.iter().enumerate() {
            for (x, cell) in row.iter().enumerate() {
                let ch = match (cell.get_wall(CellWall::Up), cell.get_wall(CellWall::Down)) {
                    (false, false) => '⥮',
                    (false, true) => '↑',
                    (true, false) => '↓',
                    _ => continue,
                };

                let pos = maze2screen(Dims(x as i32, y as i32));
                frame.draw_styled(pos, ch, style);
            }
        }
    }

    fn render_special(frames: &mut [Frame], game: &RunningGame, scheme: ColorScheme) {
        let goals = scheme.goals();

        let goal_pos = game.get_goal_pos();
        frames[goal_pos.2 as usize].draw_styled(maze2screen(goal_pos), '$', goals);
    }
}
