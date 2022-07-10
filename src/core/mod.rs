use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("CrossTerm error; {0}")]
    CrossTermError(#[from] crossterm::ErrorKind),
    #[error("Renderer error; {0}")]
    DrawBufferError(#[from] masof::renderer::Error),
    #[error("Quit")]
    Quit,
    #[error("FullQuit")]
    FullQuit,
    #[error("EmptyMenu")]
    EmptyMenu,
    #[error("InvalidValue")]
    InvalidValue,
    #[error("NewGame")]
    NewGame,
}

pub type Dims = (i32, i32);
pub type Dims3D = (i32, i32, i32);
#[allow(dead_code)]
pub type DimsU = (usize, usize);
pub type GameMode = (i32, i32, i32, bool);
