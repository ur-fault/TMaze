pub mod app;
pub mod game_state;
pub use app::App;
pub use game_state::{GameState, GameViewMode};

use crate::{
    ui::GenericUIError,
    ui::{CrosstermError, MenuError},
};

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
            // TODO: this shouldn't be EmptyMaze or at least it doesn't make sense
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

impl From<GenericUIError> for GameError {
    fn from(error: GenericUIError) -> Self {
        match error {
            GenericUIError::Back => Self::Back,
            GenericUIError::Quit => Self::FullQuit,
            GenericUIError::Crossterm(error) => Self::CrosstermError(error),
        }
    }
}
