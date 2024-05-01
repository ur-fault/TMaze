use crate::{
    core::*,
    gameboard::{
        algorithms::{
            GenerationErrorInstant, GenerationErrorThreaded, MazeGeneratorComunication, Progress,
            StopGenerationFlag,
        },
        CellWall, Maze,
    },
};

use crossbeam::channel::Receiver;
use pausable_clock::{PausableClock, PausableInstant};

use std::thread::{self, JoinHandle};
use std::time::Duration;

#[derive(Debug)]
pub struct GameAlreadyRunningError {}
#[derive(Debug)]
pub struct GameNotRunningError {}
#[derive(Debug)]
pub struct GameNotPausedError {}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub enum GameState {
    NotStarted,
    Running,
    Paused,
    Finished,
    Quitted,
}

pub struct GameProperities {
    pub game_mode: GameMode,
    pub generator:
        fn(Dims3D, bool, bool) -> Result<MazeGeneratorComunication, GenerationErrorInstant>,
}

pub enum MoveMode {
    Slow,
    Normal,
    Fast,
}

pub type GameConstructorComunication = (
    JoinHandle<Result<Game, GenerationErrorThreaded>>,
    StopGenerationFlag,
    Receiver<Progress>,
);

pub struct Game {
    maze: Maze,
    state: GameState,
    game_mode: GameMode,
    #[allow(dead_code)]
    clock: Option<PausableClock>,
    start: Option<PausableInstant>,
    player_pos: Dims3D,
    goal_pos: Dims3D,
    moves: Vec<(Dims3D, CellWall)>,
}

impl Game {
    pub fn new_threaded(
        props: GameProperities,
    ) -> Result<GameConstructorComunication, GenerationErrorInstant> {
        let GameProperities {
            game_mode: maze_mode,
            generator: generation_func,
        } = props;

        let GameMode {
            size: msize,
            is_tower,
        } = maze_mode;

        let player_pos = Dims3D(0, 0, 0);
        let goal_pos = Dims3D(msize.0 - 1, msize.1 - 1, msize.2 - 1);

        let (maze_handle, stop_flag, progress) = generation_func(msize, is_tower, true)?;

        Ok((
            thread::spawn(move || {
                let maze = maze_handle.join().unwrap()?;
                Ok(Game {
                    maze,
                    state: GameState::NotStarted,
                    game_mode: maze_mode,
                    clock: None,
                    start: None,
                    player_pos,
                    goal_pos,
                    moves: vec![],
                })
            }),
            stop_flag,
            progress,
        ))
    }

    pub fn get_state(&self) -> GameState {
        self.state
    }

    pub fn get_maze(&self) -> &Maze {
        &self.maze
    }

    pub fn get_player_pos(&self) -> Dims3D {
        self.player_pos
    }

    pub fn get_goal_pos(&self) -> Dims3D {
        self.goal_pos
    }

    pub fn get_moves(&self) -> &Vec<(Dims3D, CellWall)> {
        &self.moves
    }

    pub fn get_move_count(&self) -> usize {
        self.moves.len()
    }

    pub fn start(&mut self) -> Result<(), GameAlreadyRunningError> {
        match self.get_state() {
            GameState::NotStarted => {
                self.state = GameState::Running;
                self.clock = Some(PausableClock::default());
                self.start = Some(self.clock.as_mut().unwrap().now());

                Ok(())
            }
            _ => Err(GameAlreadyRunningError {}),
        }
    }

    pub fn quit(&mut self) {
        self.state = GameState::Quitted;
        self.clock = None;
        self.start = None;
    }

    pub fn move_player(
        &mut self,
        dir: CellWall,
        move_mode: MoveMode,
        tower_auto_up: bool,
    ) -> Result<(Dims3D, usize), GameNotRunningError> {
        self.check_running()?;

        let mut count = 0;

        match move_mode {
            MoveMode::Slow => {
                return if self.maze.get_cell(self.player_pos).unwrap().get_wall(dir) {
                    Ok((self.player_pos, 0))
                } else {
                    self.moves.push((self.player_pos, dir));
                    self.player_pos += dir.to_coord();
                    Ok((self.player_pos, 1))
                }
            }

            MoveMode::Fast => {
                while !self.maze.get_cell(self.player_pos).unwrap().get_wall(dir) {
                    self.moves.push((self.player_pos, dir));
                    self.player_pos += dir.to_coord();
                    count += 1;
                }
            }

            MoveMode::Normal => loop {
                let mut cell = self.maze.get_cell(self.player_pos).unwrap();

                if cell.get_wall(dir) {
                    break;
                }

                count += 1;

                self.moves.push((self.player_pos, dir));
                self.player_pos += dir.to_coord();

                cell = self.maze.get_cell(self.player_pos).unwrap();

                let perps = dir.perpendicular_walls();
                if !cell.get_wall(perps.0)
                    || !cell.get_wall(perps.1)
                    || !cell.get_wall(perps.2)
                    || !cell.get_wall(perps.3)
                {
                    break;
                }
            },
        }

        if tower_auto_up
            && self.game_mode.is_tower
            && !self
                .maze
                .get_cell(self.player_pos)
                .unwrap()
                .get_wall(CellWall::Up)
        {
            self.moves.push((self.player_pos, CellWall::Up));
            self.player_pos += CellWall::Up.to_coord();
            count += 1;
        }

        if self.player_pos == self.goal_pos {
            self.state = GameState::Finished;
            self.clock.as_mut().unwrap().pause();
        }

        Ok((self.player_pos, count))
    }

    pub fn check_running(&self) -> Result<(), GameNotRunningError> {
        match self.state {
            GameState::Running => Ok(()),
            _ => Err(GameNotRunningError {}),
        }
    }

    pub fn check_paused(&self) -> Result<(), GameNotPausedError> {
        match self.state {
            GameState::Paused => Ok(()),
            _ => Err(GameNotPausedError {}),
        }
    }

    pub fn get_elapsed(&self) -> Option<Duration> {
        self.clock.as_ref().map(|c| self.start.unwrap().elapsed(c))
    }

    pub fn pause(&mut self) -> Result<(), GameNotRunningError> {
        self.check_running()?;

        self.state = GameState::Paused;
        self.clock.as_mut().unwrap().pause();

        Ok(())
    }

    pub fn resume(&mut self) -> Result<(), GameNotPausedError> {
        self.check_paused()?;

        self.state = GameState::Running;
        self.clock.as_mut().unwrap().resume();

        Ok(())
    }

    pub fn reset(&mut self) {
        self.state = GameState::NotStarted;
        self.moves.clear();
        self.player_pos = Dims3D(0, 0, 0);

        self.clock = None;
        self.start = None;
    }
}
