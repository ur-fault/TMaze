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
    renderer::Frame,
    settings::{CameraMode, ColorScheme, Offset, Settings},
    ui::{self, draw_box, multisize_string, Menu, Popup, ProgressBar, Screen},
};

#[cfg(feature = "sound")]
#[allow(unused_imports)]
use crate::sound::{track::MusicTrack, SoundPlayer};

#[cfg(feature = "updates")]
#[allow(unused_imports)]
use crate::updates;

use crossterm::event::{Event as TermEvent, KeyCode, KeyEvent};

#[cfg(feature = "sound")]
#[allow(unused_imports)]
use rodio::Source;

use super::{
    app::{AppData, AppStateData},
    Activity, ActivityHandler, Change, Event,
};

//     // #[cfg(feature = "sound")]
//     // fn play_bgm(&mut self, track: MusicTrack) {
//     //     if let Some(prev_track) = self.bgm_track {
//     //         if prev_track == track {
//     //             return;
//     //         }
//     //     }
//     //
//     //     if !self.settings.get_enable_audio() || !self.settings.get_enable_music() {
//     //         return;
//     //     }
//     //
//     //     let volume = self.settings.get_audio_volume() * self.settings.get_music_volume();
//     //     self.sound_player.sink().set_volume(volume);
//     //
//     //     self.bgm_track = Some(track);
//     //     let track = track.get_track().repeat_infinite();
//     //     self.sound_player.play_track(Box::new(track));
//     // }
//
//     fn render_game(
//         &mut self,
//         game_state: &GameData,
//         camera_mode: CameraMode,
//         ups_as_goal: bool,
//         text_horizontal_margin: i32,
//     ) -> Result<(), GameError> {
//         let GameData {
//             game,
//             camera_offset,
//             player_char,
//             ..
//         } = game_state;
//
//         let player_pos = game.get_player_pos();
//
//         let maze = game.get_maze();
//
//         let maze_render_size = helpers::maze_render_size(maze);
//         let size = {
//             let size = term_size();
//             Dims(size.0 as i32, size.1 as i32)
//         };
//
//         let maze_margin = Dims(10, 3);
//
//         let fits_on_screen = maze_render_size.0 + maze_margin.0 + 2 <= size.0
//             && maze_render_size.1 + 3 + maze_margin.1 + 4 <= size.1;
//
//         let maze_pos = {
//             let pos = if fits_on_screen {
//                 ui::box_center_screen(maze_render_size)
//             } else {
//                 let last_player_real_pos = helpers::maze_pos_to_real(player_pos);
//
//                 match camera_mode {
//                     CameraMode::CloseFollow => size / 2 - last_player_real_pos,
//                     CameraMode::EdgeFollow(margin_x, margin_y) => {
//                         let player_real_pos = self.last_edge_follow_offset + last_player_real_pos;
//
//                         if player_real_pos.0 < margin_x + maze_margin.0 + 1
//                             || player_real_pos.0 > size.0 - margin_x - maze_margin.1 - 1
//                         {
//                             self.last_edge_follow_offset.0 = size.0 / 2 - last_player_real_pos.0;
//                         }
//
//                         if player_real_pos.1 < margin_y + maze_margin.1 + 1
//                             || player_real_pos.1 > size.1 - margin_y - maze_margin.1 - 1
//                         {
//                             self.last_edge_follow_offset.1 = size.1 / 2 - last_player_real_pos.1;
//                         }
//                         self.last_edge_follow_offset
//                     }
//                 }
//             };
//
//             pos + Dims::from(*camera_offset) * 2
//         };
//
//         let normal_style = self.settings.get_color_scheme().normals();
//         let text_style = self.settings.get_color_scheme().texts();
//         let player_style = self.settings.get_color_scheme().players();
//         let goal_style = self.settings.get_color_scheme().goals();
//
//         let renderer_cell = RefCell::new(self.renderer.frame());
//
//         let text_frame = if fits_on_screen {
//             Rect::new_sized(maze_pos, maze_render_size - Dims(1, 1)).with_margin(Dims(-1, -2))
//         } else {
//             Rect::new_sized(Dims(0, 0), size).with_margin(maze_margin)
//         };
//         let frame = text_frame.with_margin(Dims(1, 2));
//
//         let mut normal_context = DrawContext {
//             frame: &renderer_cell,
//             style: normal_style,
//             rect: frame.into(),
//         };
//         let mut text_context = DrawContext {
//             frame: &renderer_cell,
//             style: text_style,
//             rect: text_frame.into(),
//         };
//         let mut player_context = DrawContext {
//             frame: &renderer_cell,
//             style: player_style,
//             rect: frame.into(),
//         };
//         let mut goal_context = DrawContext {
//             frame: &renderer_cell,
//             style: goal_style,
//             rect: frame.into(),
//         };
//
//         let box_frame = text_frame.with_margin(Dims(0, 1));
//         normal_context.draw_box(box_frame.start, box_frame.size());
//
//         let floor = player_pos.2 + camera_offset.2;
//
//         let draw_line_double_duo =
//             |mut context: DrawContext, pos: (i32, i32), l1: LineDir, l2: LineDir| {
//                 context.draw_str(
//                     pos.into(),
//                     &format!("{}{}", l1.double_line(), l2.double_line(),),
//                 )
//             };
//
//         let draw_line_double = |mut context: DrawContext, pos: (i32, i32), l: LineDir| {
//             context.draw_str(pos.into(), l.double_line())
//         };
//
//         draw_line_double_duo(
//             normal_context,
//             maze_pos.into(),
//             LineDir::BottomRight,
//             LineDir::Horizontal,
//         );
//         draw_line_double_duo(
//             normal_context,
//             (maze_pos.0 + maze_render_size.0 - 2, maze_pos.1),
//             LineDir::Horizontal,
//             LineDir::BottomLeft,
//         );
//
//         draw_line_double(
//             normal_context,
//             (maze_pos.0, maze_pos.1 + maze_render_size.1 - 2),
//             LineDir::Vertical,
//         );
//         draw_line_double(
//             normal_context,
//             (
//                 maze_pos.0 + maze_render_size.0 - 1,
//                 maze_pos.1 + maze_render_size.1 - 2,
//             ),
//             LineDir::Vertical,
//         );
//
//         draw_line_double(
//             normal_context,
//             (maze_pos.0, maze_pos.1 + maze_render_size.1 - 1),
//             LineDir::TopRight,
//         );
//         draw_line_double_duo(
//             normal_context,
//             (
//                 maze_pos.0 + maze_render_size.0 - 2,
//                 maze_pos.1 + maze_render_size.1 - 1,
//             ),
//             LineDir::Horizontal,
//             LineDir::TopLeft,
//         );
//
//         for x in 0..maze.size().0 - 1 {
//             draw_line_double_duo(
//                 normal_context,
//                 (x * 2 + maze_pos.0 + 1, maze_pos.1),
//                 LineDir::Horizontal,
//                 if maze
//                     .get_cell(Dims3D(x, 0, floor))
//                     .unwrap()
//                     .get_wall(CellWall::Right)
//                 {
//                     LineDir::ClosedTop
//                 } else {
//                     LineDir::Horizontal
//                 },
//             );
//
//             draw_line_double_duo(
//                 normal_context,
//                 (x * 2 + maze_pos.0 + 1, maze_pos.1 + maze_render_size.1 - 1),
//                 LineDir::Horizontal,
//                 if maze
//                     .get_cell(Dims3D(x, maze.size().1 - 1, floor))
//                     .unwrap()
//                     .get_wall(CellWall::Right)
//                 {
//                     LineDir::ClosedBottom
//                 } else {
//                     LineDir::Horizontal
//                 },
//             );
//         }
//
//         // Vertical edge lines
//         for y in 0..maze.size().1 - 1 {
//             let ypos = y * 2 + maze_pos.1 + 1;
//             if ypos >= size.1 - 2 {
//                 break;
//             }
//
//             if ypos == -1 {
//                 continue;
//             }
//
//             draw_line_double(
//                 normal_context,
//                 (maze_pos.0, ypos + 1),
//                 value_if_else(
//                     maze.get_cell(Dims3D(0, y, floor))
//                         .unwrap()
//                         .get_wall(CellWall::Bottom),
//                     || LineDir::ClosedLeft,
//                     || LineDir::Vertical,
//                 ),
//             );
//
//             draw_line_double(
//                 normal_context,
//                 (maze_pos.0 + maze_render_size.0 - 1, ypos + 1),
//                 value_if_else(
//                     maze.get_cell(Dims3D(maze.size().0 - 1, y, floor))
//                         .unwrap()
//                         .get_wall(CellWall::Bottom),
//                     || LineDir::ClosedRight,
//                     || LineDir::Vertical,
//                 ),
//             );
//
//             draw_line_double(normal_context, (maze_pos.0, ypos), LineDir::Vertical);
//
//             draw_line_double(
//                 normal_context,
//                 (maze_pos.0 + maze_render_size.0 - 1, y * 2 + maze_pos.1 + 1),
//                 LineDir::Vertical,
//             );
//         }
//
//         // Drawing visited places (moves)
//         let moves = game.get_moves();
//         for (move_pos, _) in moves {
//             if move_pos.2 == floor {
//                 let real_pos = helpers::maze_pos_to_real(*move_pos);
//                 normal_context.draw_char(maze_pos + real_pos, '.');
//             }
//         }
//
//         // Drawing insides of the maze itself
//         for (iy, row) in maze.get_cells()[floor as usize].iter().enumerate() {
//             let ypos = iy as i32 * 2 + 1 + maze_pos.1;
//
//             for (ix, cell) in row.iter().enumerate() {
//                 let xpos = ix as i32 * 2 + 1 + maze_pos.0;
//                 if cell.get_wall(CellWall::Right) && ix != maze.size().0 as usize - 1 {
//                     draw_line_double(normal_context, (xpos + 1, ypos), LineDir::Vertical);
//                 }
//
//                 if ypos + 1 < size.1 - 2
//                     && ypos > 0
//                     && cell.get_wall(CellWall::Bottom)
//                     && iy != maze.size().1 as usize - 1
//                 {
//                     draw_line_double(normal_context, (xpos, ypos + 1), LineDir::Horizontal);
//                 }
//
//                 let contexts = GameDrawContexts {
//                     normal: normal_context,
//                     player: player_context,
//                     goal: goal_context,
//                 };
//
//                 Self::draw_stairs(
//                     contexts,
//                     cell,
//                     (ix as i32, iy as i32),
//                     maze_pos.into(),
//                     floor,
//                     player_pos,
//                     ups_as_goal,
//                 );
//
//                 let cell2 = match maze.get_cell(Dims3D(ix as i32 + 1, iy as i32 + 1, floor)) {
//                     Some(cell) => cell,
//                     None => continue,
//                 };
//
//                 draw_line_double(
//                     normal_context,
//                     (xpos + 1, ypos + 1),
//                     LineDir::from_bools(
//                         cell.get_wall(CellWall::Bottom),
//                         cell.get_wall(CellWall::Right),
//                         cell2.get_wall(CellWall::Top),
//                         cell2.get_wall(CellWall::Left),
//                     ),
//                 );
//             }
//         }
//
//         let goal_pos = game.get_goal_pos();
//         if floor == goal_pos.2 {
//             goal_context.draw_char(
//                 Dims::from(goal_pos) * 2 + maze_pos + Dims(1, 1),
//                 constants::GOAL_CHAR,
//             );
//         }
//
//         if floor == player_pos.2 {
//             player_context.draw_char(
//                 Dims::from(player_pos) * 2 + maze_pos + Dims(1, 1),
//                 *player_char,
//             );
//
//             let contexts = GameDrawContexts {
//                 normal: normal_context,
//                 player: player_context,
//                 goal: goal_context,
//             };
//
//             Self::draw_stairs(
//                 contexts,
//                 maze.get_cell(player_pos).unwrap(),
//                 (player_pos.0, player_pos.1),
//                 maze_pos.into(),
//                 floor,
//                 player_pos,
//                 ups_as_goal,
//             );
//         }
//
//         let pos_text = if maze.size().2 > 1 {
//             format!(
//                 "x:{} y:{} floor:{}",
//                 player_pos.0 + 1,
//                 player_pos.1 + 1,
//                 player_pos.2 + 1
//             )
//         } else {
//             format!("x:{} y:{}", player_pos.0 + 1, player_pos.1 + 1)
//         };
//
//         let from_start = game.get_elapsed().unwrap();
//         let view_mode = game_state.view_mode.to_string();
//         let (view_mode, pos_text) =
//             if view_mode.len() as i32 + text_horizontal_margin * 2 + pos_text.len() as i32 + 1
//                 > text_frame.size().0
//             {
//                 (
//                     format!("{}", view_mode.chars().next().unwrap()),
//                     format!(
//                         "x:{} y:{} f:{}",
//                         player_pos.0 + 1,
//                         player_pos.1 + 1,
//                         player_pos.2 + 1
//                     ),
//                 )
//             } else {
//                 (view_mode, pos_text)
//             };
//
//         let texts = (
//             &pos_text,
//             view_mode.as_str(),
//             &format!("{} moves", game_state.game.get_move_count()),
//             &ui::format_duration(from_start),
//         );
//
//         // Print texts
//         let str_pos_tl = Dims(
//             text_horizontal_margin + text_frame.start.0,
//             text_frame.start.1,
//         );
//         let str_pos_tr = Dims(
//             text_frame.end.0 - text_horizontal_margin - texts.1.len() as i32 + 1,
//             text_frame.start.1,
//         );
//         let str_pos_bl = Dims(
//             text_horizontal_margin + text_frame.start.0,
//             text_frame.end.1,
//         );
//         let str_pos_br =
//             text_frame.end - Dims(text_horizontal_margin + texts.3.len() as i32 - 1, 0);
//
//         text_context.draw_str(str_pos_tl, texts.0);
//         text_context.draw_str(str_pos_tr, texts.1);
//         text_context.draw_str(str_pos_bl, texts.2);
//         text_context.draw_str(str_pos_br, texts.3);
//
//         self.renderer.show()?;
//
//         Ok(())
//     }
// }

pub struct MainMenu(Menu);

impl MainMenu {
    pub fn new(settings: &Settings) -> Self {
        let color_scheme = settings.get_color_scheme();

        Self(Menu::new(
            ui::MenuConfig::new(
                "TMaze".to_string(),
                vec![
                    "New Game".to_string(),
                    "Settings".to_string(),
                    "Controls".to_string(),
                    "About".to_string(),
                    "Quit".to_string(),
                ],
            )
            .counted()
            .box_style(color_scheme.normals())
            .text_style(color_scheme.texts()),
        ))
    }

    fn show_settings_screen(&mut self, settings: &Settings) -> Change {
        let popup = Popup::new(
            "Settings".to_string(),
            vec![
                "Path to the current settings:".to_string(),
                format!(" {}", settings.path().to_string_lossy().to_string()),
            ],
        );

        Change::push(Activity::new_base("controls".to_string(), Box::new(popup)))
    }

    fn show_controls_popup(&mut self) -> Change {
        let popup = Popup::new(
            "Controls".to_string(),
            vec![
                "WASD and arrows: move".to_string(),
                "Space: switch adventure/spectaror mode".to_string(),
                "Q, F or L: move down".to_string(),
                "E, R or P: move up".to_string(),
                "With SHIFT move at the end in single dir".to_string(),
                "Escape: pause menu".to_string(),
            ],
        );

        Change::push(Activity::new_base("controls".to_string(), Box::new(popup)))
    }

    fn show_about_popup(&mut self) -> Change {
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
        );

        Change::push(Activity::new_base("about".to_string(), Box::new(popup)))
    }

    fn start_new_game(&mut self, settings: &Settings, use_data: &AppStateData) -> Change {
        Change::push(Activity::new_base(
            "maze size",
            Box::new(MazeSizeMenu::new(settings, use_data)),
        ))
    }
}

impl ActivityHandler for MainMenu {
    fn update(&mut self, events: Vec<super::Event>, data: &mut AppData) -> Option<Change> {
        match self.0.update(events, data)? {
            Change::Pop {
                res: Some(sub_activity),
                ..
            } => {
                let index = *sub_activity
                    .downcast::<usize>()
                    .expect("menu should return index");
                match index {
                    0 /* new game */ => Some(self.start_new_game(&data.settings, &data.use_data)),
                    1 /* settings */ => Some(self.show_settings_screen(&data.settings)),
                    2 /* controls */ => Some(self.show_controls_popup()),
                    3 /* about    */ => Some(self.show_about_popup()),
                    4 /* quit     */ => Some(Change::pop_top()),
                    _ => panic!("main menu should only return valid index between 0 and 4"),
                }
            }
            res => Some(res),
        }
    }

    fn screen(&self) -> &dyn ui::Screen {
        &self.0
    }
}

pub struct MazeSizeMenu {
    menu: Menu,
    presets: Vec<GameMode>,
}

impl MazeSizeMenu {
    pub fn new(settings: &Settings, app_state_data: &AppStateData) -> Self {
        let color_scheme = settings.get_color_scheme();
        let mut menu_config = ui::MenuConfig::new(
            "Maze size".to_string(),
            settings
                .get_mazes()
                .iter()
                .map(|maze| maze.title.clone())
                .collect::<Vec<_>>(),
            // vec!["100x100".to_string()],
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
        // let presets = vec![GameMode {
        //     size: Dims3D(100, 100, 1),
        //     is_tower: false,
        // }];

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

                    return Some(Change::push(Activity::new_base(
                        "maze_gen".to_string(),
                        Box::new(MazeAlgorithmMenu::new(preset, &data.settings)),
                    )));
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
}

impl MazeAlgorithmMenu {
    pub fn new(preset: GameMode, settings: &Settings) -> Self {
        let color_scheme = settings.get_color_scheme();
        let menu = Menu::new(
            ui::MenuConfig::new(
                "Maze generation algorithm".to_string(),
                vec![
                    "Randomized Kruskal's".to_string(),
                    "Depth-first search".to_string(),
                ],
            )
            .counted()
            .box_style(color_scheme.normals())
            .text_style(color_scheme.texts()),
        );

        Self { menu, preset }
    }
}

impl ActivityHandler for MazeAlgorithmMenu {
    fn update(&mut self, events: Vec<super::Event>, data: &mut AppData) -> Option<Change> {
        match self.menu.update(events, data) {
            Some(change) => match change {
                Change::Pop {
                    res: Some(algo), ..
                } => {
                    let index = *algo.downcast::<usize>().expect("menu should return index");

                    let gen = match index {
                        0 => RndKruskals::generate,
                        1 => DepthFirstSearch::generate,
                        _ => panic!(),
                    };

                    return Some(Change::push(Activity::new_base(
                        "maze_gen".to_string(),
                        Box::new(MazeGenerationActivity::new(
                            self.preset,
                            gen,
                            &data.settings,
                        )),
                    )));
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
                            match self.comm.take() {
                                Some(comm) => {
                                    comm.stop_flag.stop();
                                    let _ = comm.handle.join().unwrap();
                                }
                                None => {}
                            };
                            return Some(Change::pop_top());
                        }
                        _ => {}
                    }
                }
                _ => {}
            }
        }

        if self.comm.is_none() {
            return match RunningGame::new_threaded(self.game_props.clone()) {
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
            };
        }

        if self.comm.as_ref().unwrap().handle.is_finished() {
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
                        Box::new(GameActivity::new(game_data, &data.settings)),
                    )))
                }
                Err(err) => match err {
                    GenErrorThreaded::AbortGeneration => Some(Change::pop_top()),
                    GenErrorThreaded::GenerationError(_) => {
                        panic!("Instant generation error should be handled before");
                    }
                },
            }
        } else {
            let Progress { done, from, .. } = self.comm.as_ref().unwrap().progress();
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

    fn screen(&self) -> &dyn ui::Screen {
        &self.progress_bar
    }
}

pub struct PauseMenu {
    menu: Menu,
}

impl PauseMenu {
    pub fn new(settings: &Settings) -> Self {
        let color_scheme = settings.get_color_scheme();
        let menu = Menu::new(
            ui::MenuConfig::new(
                "Paused".to_string(),
                vec![
                    "Resume".to_string(),
                    "Main Menu".to_string(),
                    "Quit".to_string(),
                ],
            )
            .box_style(color_scheme.normals())
            .text_style(color_scheme.texts()),
        );

        Self { menu }
    }
}

impl ActivityHandler for PauseMenu {
    fn update(&mut self, events: Vec<Event>, data: &mut AppData) -> Option<Change> {
        match self.menu.update(events, data) {
            Some(change) => match change {
                Change::Pop { res: Some(res), .. } => {
                    let index = *res.downcast::<usize>().expect("menu should return index");

                    match index {
                        0 => Some(Change::pop_top()),
                        1 => Some(Change::pop_until("main menu")),
                        2 => Some(Change::pop_all()),
                        _ => panic!(),
                    }
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

pub struct GameActivity {
    camera_mode: CameraMode,
    color_scheme: ColorScheme,
    game: GameData,
    maze_board: MazeBoard,
    show_debug: bool,
}

impl GameActivity {
    pub fn new(game: GameData, settings: &Settings) -> Self {
        let camera_mode = settings.get_camera_mode();
        let color_scheme = settings.get_color_scheme();
        let game = game;
        let maze_board = MazeBoard::new(&game.game, settings);

        Self {
            camera_mode,
            color_scheme,
            game,
            maze_board,
            show_debug: false,
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
        &self.maze_board.frames[self.game.game.get_player_pos().2 as usize]
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
        let view_mode = multisize_string(view_mode.to_multisize_strings(), max_width as usize);

        let tl = vp_pos - Dims(1, 2);
        let br = vp_pos + vp_size + Dims(1, 1);

        // draw them
        let mut draw = |text: &str, pos| frame.draw_styled(pos, text, color_scheme.texts());

        draw(&pos_text, tl);
        draw(&view_mode, Dims(br.0 - view_mode.len() as i32, tl.1));
        draw(&move_count, Dims(tl.0, br.1));
        draw(&from_start, Dims(br.0 - from_start.len() as i32, br.1));
    }

    pub fn render_visited_places(&self, frame: &mut Frame) {
        use CellWall::{Down, Up};

        let game = &self.game.game;
        for (move_pos, _) in game.get_moves() {
            let cell = game.get_maze().get_cell(*move_pos).unwrap();
            if move_pos.2 == game.get_player_pos().2 && cell.get_wall(Up) && cell.get_wall(Down) {
                let real_pos = maze2screen(*move_pos);
                frame.draw_styled(real_pos, '.', self.color_scheme.normals());
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
            match event {
                Event::Term(TermEvent::Key(key_event)) => {
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
                _ => {}
            }
        }

        // self.camera_mode = CameraMode::SmoothFollow(0.3);
        // self.camera_mode = CameraMode::CloseFollow;
        if self.game.view_mode == GameViewMode::Adventure {
            match self.camera_mode {
                CameraMode::CloseFollow => {
                    self.game.camera_pos = maze2screen_3d(self.game.game.get_player_pos());
                }
                CameraMode::EdgeFollow(xoff, yoff) => {
                    let (vp_size, does_fit) = self.viewport_size(data.screen_size);
                    if !does_fit {
                        let xoff = xoff.to_chars(vp_size.0);
                        let yoff = yoff.to_chars(vp_size.1);

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
                } // TODO: implement smooth follow, but as separate flag
                  // CameraMode::SmoothFollow(t) => {
                  //     let old = self.game.camera_pos;
                  //     let new = maze2screen_3d(self.game.game.get_player_pos());
                  //     let Dims(x, y) = Dims::from(new) - old.into();
                  //     let x = (x as f32 * t).round() as i32;
                  //     let y = (y as f32 * t).round() as i32;
                  //     self.game.camera_pos = old + Dims3D(x, y, new.2);
                  // }
            }
        }

        self.show_debug = data.use_data.show_debug;

        if self.game.game.get_state() == RunningGameState::Finished {
            let game = &self.game.game;
            let texts = vec![
                format!(
                    "Time:  {}",
                    ui::format_duration(game.get_elapsed().unwrap())
                ),
                format!("Moves: {}", game.get_move_count()),
                format!(
                    "Size:  {}x{}x{}",
                    game.get_maze().size().0,
                    game.get_maze().size().1,
                    game.get_maze().size().2,
                ),
            ];

            let color_scheme = &self.color_scheme;
            let popup = Popup::new("You won".to_string(), texts)
                .box_style(color_scheme.normals())
                .text_style(color_scheme.texts())
                .title_style(color_scheme.texts());
            let activity = Activity::new_base("won".to_string(), Box::new(popup));

            // TODO: add R to play a new game

            return Some(Change::replace_at(1, activity));
        };

        None
    }

    fn screen(&self) -> &dyn ui::Screen {
        self
    }
}

impl Screen for GameActivity {
    fn draw(&self, frame: &mut crate::renderer::Frame) -> std::io::Result<()> {
        let maze_frame = self.current_floor_frame();
        let color_scheme = &self.color_scheme;
        let game = &self.game.game;

        let (vp_size, does_fit) = self.viewport_size(frame.size);
        let maze_pos = match does_fit {
            true => Dims(0, 0),
            false => vp_size / 2 - self.game.camera_pos.into(),
        };

        // TODO: reuse the viewport between frames and resize it when needed
        let mut viewport = Frame::new(vp_size.into());

        // maze
        viewport.draw(maze_pos, maze_frame);
        self.render_visited_places(&mut viewport);

        // player
        if (game.get_player_pos().2 as usize) == game.get_player_pos().2 as usize {
            let player = game.get_player_pos();
            let cell = game.get_maze().get_cell(player).unwrap();
            if !cell.get_wall(CellWall::Up) || !cell.get_wall(CellWall::Down) {
                viewport[maze2screen(game.get_player_pos())]
                    .content_mut()
                    .unwrap()
                    .style
                    .foreground_color = Some(color_scheme.player);
            } else {
                viewport.draw_styled(
                    maze_pos + maze2screen(player),
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

    let xo = rulers.0.to_chars(vps.0);
    let yo = rulers.1.to_chars(vps.1);

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

    // not allowed on blocks, so we use a closure
    #[rustfmt::skip]
    (|| {
        draw(Dims(xo        , 0)         , V, false);
        draw(Dims(vps.0 - xo, 0)         , V, true);
        draw(Dims(xo        , vps.1 + 1) , V, false);
        draw(Dims(vps.0 - xo, vps.1 + 1) , V, true);

        draw(Dims(0         , yo)        , H, false);
        draw(Dims(0         , vps.1 - yo), H, true);
        draw(Dims(vps.0 + 1 , yo)        , H, false);
        draw(Dims(vps.0 + 1 , vps.1 - yo), H, true);
    })();
}

pub struct MazeBoard {
    frames: Vec<Frame>,
}

impl MazeBoard {
    pub fn new(game: &RunningGame, settings: &Settings) -> Self {
        let maze = game.get_maze();
        let scheme = settings.get_color_scheme();

        let mut frames = (0..maze.size().2)
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

        let mut draw =
            |pos, l: LineDir| frame.draw_styled(Dims::from(pos).into(), l.double(), normals);

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

    fn render_stairs(frame: &mut Frame, floors: &Vec<Vec<Cell>>, tower: bool, scheme: ColorScheme) {
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

                let pos = maze2screen(Dims(x as i32, y as i32).into());
                frame.draw_styled(pos, ch, style);
            }
        }
    }

    fn render_special(frames: &mut Vec<Frame>, game: &RunningGame, scheme: ColorScheme) {
        let goals = scheme.goals();

        let goal_pos = game.get_goal_pos();
        frames[goal_pos.2 as usize].draw_styled(maze2screen(goal_pos), '$', goals);
    }
}
