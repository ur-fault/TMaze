use cmaze::{
    game::{Game, MoveMode},
    maze::{CellWall, Dims3D},
};
use crossterm::event::KeyModifiers;
use masof::{KeyCode, KeyEvent};

use crate::{settings::Settings, ui};

use super::app::GameError;

#[derive(PartialEq, Eq, Clone, Copy)]
pub enum GameViewMode {
    Adventure,
    Spectator,
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
        let KeyEvent { code, modifiers } = event;

        match code {
            KeyCode::Up | KeyCode::Char('w' | 'W') => {
                self.apply_move(CellWall::Top, modifiers.contains(KeyModifiers::SHIFT));
            }
            KeyCode::Down | KeyCode::Char('s' | 'S') => {
                self.apply_move(CellWall::Bottom, modifiers.contains(KeyModifiers::SHIFT));
            }
            KeyCode::Left | KeyCode::Char('a' | 'A') => {
                self.apply_move(CellWall::Left, modifiers.contains(KeyModifiers::SHIFT));
            }
            KeyCode::Right | KeyCode::Char('d' | 'D') => {
                self.apply_move(CellWall::Right, modifiers.contains(KeyModifiers::SHIFT));
            }
            KeyCode::Char('f' | 'F' | 'q' | 'Q' | 'l' | 'L') => {
                self.apply_move(CellWall::Down, modifiers.contains(KeyModifiers::SHIFT));
            }
            KeyCode::Char('r' | 'R' | 'e' | 'E' | 'p' | 'P') => {
                self.apply_move(CellWall::Up, modifiers.contains(KeyModifiers::SHIFT));
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

    pub fn apply_move(&mut self, wall: CellWall, fast: bool) {
        if self.view_mode == GameViewMode::Spectator {
            let cam_off = wall.reverse_wall().to_coord() + self.camera_offset;

            self.camera_offset = Dims3D(
                cam_off.0,
                cam_off.1,
                (-self.game.get_player_pos().2).max(
                    (self.game.get_maze().size().2 - self.game.get_player_pos().2 - 1)
                        .min(cam_off.2),
                ),
            )
        } else {
            self.game
                .move_player(
                    wall,
                    if self.settings.get_slow() {
                        MoveMode::Slow
                    } else if fast {
                        MoveMode::Fast
                    } else {
                        MoveMode::Normal
                    },
                    !self.settings.get_disable_tower_auto_up(),
                )
                .unwrap();
        }
    }
}
