use std::fmt::Display;

use cmaze::{
    game::{Game, MoveMode},
    gameboard::{cell::Way, Dims3D, Passage},
};
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

use crate::{helpers::is_release, settings::Settings};

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

pub struct ShowMenu;

pub struct GameState {
    pub game: Game,
    pub camera_offset: Dims3D,
    pub view_mode: GameViewMode,
    pub player_char: char,
    pub is_tower: bool,
    pub settings: Settings,
}

impl GameState {
    pub fn handle_event(&mut self, event: KeyEvent) -> Result<(), ShowMenu> {
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
                self.apply_move(Way::Top, is_fast);
            }
            KeyCode::Down | KeyCode::Char('s' | 'S') => {
                self.apply_move(Way::Bottom, is_fast);
            }
            KeyCode::Left | KeyCode::Char('a' | 'A') => {
                self.apply_move(Way::Left, is_fast);
            }
            KeyCode::Right | KeyCode::Char('d' | 'D') => {
                self.apply_move(Way::Right, is_fast);
            }
            KeyCode::Char('f' | 'F' | 'q' | 'Q' | 'l' | 'L') => {
                self.apply_move(Way::Down, is_fast);
            }
            KeyCode::Char('r' | 'R' | 'e' | 'E' | 'p' | 'P') => {
                self.apply_move(Way::Up, is_fast);
            }
            KeyCode::Char('z' | 'Z' | 'o' | 'O') => {
                self.apply_move(Way::Portal, is_fast);
            }
            KeyCode::Char(' ') => {
                if self.view_mode == GameViewMode::Spectator {
                    self.camera_offset = Dims3D(0, 0, 0);
                    self.view_mode = GameViewMode::Adventure;
                } else {
                    self.view_mode = GameViewMode::Spectator;
                }
            }
            KeyCode::Char('.') => {
                self.view_mode = GameViewMode::Spectator;
                self.camera_offset = self.game.get_player_pos() - self.game.get_goal_pos();
                self.camera_offset.2 *= -1;
            }
            KeyCode::Esc => return Err(ShowMenu),
            _ => {}
        }

        Ok(())
    }

    pub fn apply_move(&mut self, way: Way, fast: bool) {
        match self.view_mode {
            GameViewMode::Spectator => {
                let Some(rev_passage) = way.reverse() else {
                    return;
                };

                let cam_off = rev_passage.offset().unwrap() + self.camera_offset;

                self.camera_offset = Dims3D(
                    cam_off.0,
                    cam_off.1,
                    (-self.game.get_player_pos().2).max(
                        (self.game.get_maze().size().2 - self.game.get_player_pos().2 - 1)
                            .min(cam_off.2),
                    ),
                )
            }
            GameViewMode::Adventure => {
                let mode = match (self.settings.get_slow(), fast) {
                    (true, _) => MoveMode::Slow,
                    (false, true) => MoveMode::Fast,
                    (false, false) => MoveMode::Normal,
                };

                self.game
                    .move_player(way, mode, !self.settings.get_disable_tower_auto_up())
                    .unwrap();
            }
        }
    }
}
