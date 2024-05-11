use crate::{
    core::*,
    gameboard::{
        algorithms::{GenErrorInstant, GenErrorThreaded, Progress, StopGenerationFlag},
        CellWall, Maze,
    },
};

use pausable_clock::{PausableClock, PausableInstant};

use std::time::Duration;
use std::{
    sync::{Arc, Mutex},
    thread::{self, JoinHandle},
};

#[derive(Debug)]
pub struct GameAlreadyRunningError {}
#[derive(Debug)]
pub struct GameNotRunningError {}
#[derive(Debug)]
pub struct GameNotPausedError {}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub enum RunningGameState {
    NotStarted,
    Running,
    Paused,
    Finished,
    Quitted,
}

pub type GeneratorFn =
    fn(Dims3D, bool) -> Result<ProgressComm<Result<Maze, GenErrorThreaded>>, GenErrorInstant>;

#[derive(Clone, Debug)]
pub struct GameProperities {
    pub game_mode: GameMode,
    pub generator: GeneratorFn,
}

pub enum MoveMode {
    Slow,
    Normal,
    Fast,
}

pub struct ProgressComm<R> {
    pub handle: JoinHandle<R>,
    pub stop_flag: StopGenerationFlag,
    pub recv: Arc<Mutex<Progress>>,
}

impl<R> ProgressComm<R> {
    pub fn progress(&self) -> Progress {
        *self.recv.lock().unwrap()
    }
}

pub struct RunningGame {
    maze: Maze,
    state: RunningGameState,
    game_mode: GameMode,
    #[allow(dead_code)]
    clock: Option<PausableClock>,
    start: Option<PausableInstant>,
    player_pos: Dims3D,
    goal_pos: Dims3D,
    moves: Vec<(Dims3D, CellWall)>,
}

impl RunningGame {
    pub fn new_threaded(
        props: GameProperities,
    ) -> Result<ProgressComm<Result<RunningGame, GenErrorThreaded>>, GenErrorInstant> {
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

        let ProgressComm {
            handle: maze_handle,
            stop_flag,
            recv: progress,
        } = generation_func(msize, is_tower)?;

        Ok(ProgressComm {
            handle: thread::spawn(move || {
                let maze = maze_handle.join().unwrap()?;
                Ok(RunningGame {
                    maze,
                    state: RunningGameState::NotStarted,
                    game_mode: maze_mode,
                    clock: None,
                    start: None,
                    player_pos,
                    goal_pos,
                    moves: vec![],
                })
            }),
            stop_flag,
            recv: progress,
        })
    }

    pub fn get_state(&self) -> RunningGameState {
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
        if let RunningGameState::NotStarted = self.get_state() {
            self.state = RunningGameState::Running;
            self.clock = Some(PausableClock::default());
            self.start = Some(self.clock.as_mut().unwrap().now());

            Ok(())
        } else {
            Err(GameAlreadyRunningError {})
        }
    }

    pub fn quit(&mut self) {
        self.state = RunningGameState::Quitted;
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
            self.state = RunningGameState::Finished;
            self.clock.as_mut().unwrap().pause();
        }

        Ok((self.player_pos, count))
    }

    pub fn check_running(&self) -> Result<(), GameNotRunningError> {
        match self.state {
            RunningGameState::Running => Ok(()),
            _ => Err(GameNotRunningError {}),
        }
    }

    pub fn check_paused(&self) -> Result<(), GameNotPausedError> {
        match self.state {
            RunningGameState::Paused => Ok(()),
            _ => Err(GameNotPausedError {}),
        }
    }

    pub fn get_elapsed(&self) -> Option<Duration> {
        self.clock.as_ref().map(|c| self.start.unwrap().elapsed(c))
    }

    pub fn pause(&mut self) -> Result<(), GameNotRunningError> {
        self.check_running()?;

        self.state = RunningGameState::Paused;
        self.clock.as_mut().unwrap().pause();

        Ok(())
    }

    pub fn resume(&mut self) -> Result<(), GameNotPausedError> {
        self.check_paused()?;

        self.state = RunningGameState::Running;
        self.clock.as_mut().unwrap().resume();

        Ok(())
    }

    pub fn reset(&mut self) {
        self.state = RunningGameState::NotStarted;
        self.moves.clear();
        self.player_pos = Dims3D(0, 0, 0);

        self.clock = None;
        self.start = None;
    }
}
