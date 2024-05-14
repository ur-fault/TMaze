use std::fmt::Display;

use cmaze::{
    core::Dims3D,
    game::{MoveMode, RunningGame},
    gameboard::CellWall,
};
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use tap::Tap;

use crate::{
    helpers::{is_release, maze2screen_3d},
    settings::Settings,
};

#[derive(PartialEq, Eq, Clone, Copy)]
pub enum GameViewMode {
    Adventure,
    Spectator,
}

impl Display for GameViewMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            GameViewMode::Adventure => write!(f, "Adventure"),
            GameViewMode::Spectator => write!(f, "Spectator"),
        }
    }
}

pub struct ShowPauseMenu;

pub struct GameData {
    pub game: RunningGame,
    pub camera_pos: Dims3D,
    pub view_mode: GameViewMode,
    pub player_char: char,
}

impl GameData {
    pub fn handle_event(&mut self, settings: &Settings, event: KeyEvent) -> Result<(), bool> {
        let KeyEvent {
            code,
            modifiers,
            kind,
            ..
        } = event;
        if is_release(kind) {
            return Ok(());
        }

        let is_fast = modifiers.contains(KeyModifiers::SHIFT);

        match code {
            KeyCode::Up | KeyCode::Char('w' | 'W') => {
                self.apply_move(settings, CellWall::Top, is_fast);
            }
            KeyCode::Down | KeyCode::Char('s' | 'S') => {
                self.apply_move(settings, CellWall::Bottom, is_fast);
            }
            KeyCode::Left | KeyCode::Char('a' | 'A') => {
                self.apply_move(settings, CellWall::Left, is_fast);
            }
            KeyCode::Right | KeyCode::Char('d' | 'D') => {
                self.apply_move(settings, CellWall::Right, is_fast);
            }
            KeyCode::Char('Q') => return Err(true),
            KeyCode::Char('f' | 'q' | 'l') => {
                self.apply_move(settings, CellWall::Down, is_fast);
            }
            KeyCode::Char('r' | 'e' | 'p') => {
                self.apply_move(settings, CellWall::Up, is_fast);
            }
            KeyCode::Char(' ') => match self.view_mode {
                GameViewMode::Spectator => {
                    self.camera_pos = maze2screen_3d(self.game.get_player_pos());
                    self.view_mode = GameViewMode::Adventure;
                    log::info!("Switched to Adventure mode");
                }
                GameViewMode::Adventure => {
                    self.view_mode = GameViewMode::Spectator;
                    log::info!("Switched to Spectator mode");
                }
            },
            KeyCode::Char('.') => {
                self.view_mode = GameViewMode::Spectator;
                self.camera_pos = self.game.get_player_pos() - self.game.get_goal_pos();
                self.camera_pos.2 *= -1;
            }
            KeyCode::Esc => return Err(false),
            _ => {}
        }

        Ok(())
    }

    pub fn apply_move(&mut self, settings: &Settings, wall: CellWall, fast: bool) {
        match self.view_mode {
            GameViewMode::Spectator => {
                let cam_off =
                    wall.reverse_wall().to_coord().tap_mut(|c| c.2 *= -1) + self.camera_pos;

                // TODO: allow `fast` movement

                self.camera_pos = Dims3D(
                    cam_off.0,
                    cam_off.1,
                    (-self.game.get_player_pos().2).max(
                        (self.game.get_maze().size().2 - self.game.get_player_pos().2 - 1)
                            .min(cam_off.2),
                    ),
                )
            }
            GameViewMode::Adventure => {
                self.game
                    .move_player(
                        wall,
                        if settings.get_slow() {
                            MoveMode::Slow
                        } else if fast {
                            MoveMode::Fast
                        } else {
                            MoveMode::Normal
                        },
                        !settings.get_disable_tower_auto_up(),
                    )
                    .unwrap();
            }
        }
    }
}
