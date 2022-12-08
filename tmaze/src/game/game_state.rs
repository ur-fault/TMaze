use cmaze::{game::Game, maze::Dims3D};

#[derive(PartialEq, Eq, Clone, Copy)]
pub enum GameViewMode {
    Adventure,
    Spectator,
}

pub struct GameState {
    pub game: Game,
    pub camera_offset: Dims3D,
    pub view_mode: GameViewMode,
    pub player_char: char,
    pub is_tower: bool,
}
