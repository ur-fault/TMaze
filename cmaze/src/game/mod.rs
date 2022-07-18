use crate::core::*;
use crate::maze::{CellWall, GenerationError, Maze, ReportCallbackError};
use core::fmt;
use pausable_clock::{PausableClock, PausableInstant};
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

pub type GameProperities<R, A, T> = (
    GameMode,                                                           // Game mode
    fn(Dims3D, bool, Option<T>) -> Result<Maze, GenerationError<R, A>>, // Maze generator
    Option<T>,                                                          // Maze generator callback
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
    pub fn new<R, A, T>(props: GameProperities<R, A, T>) -> Result<Game, GenerationError<R, A>>
    where
        R: fmt::Debug,
        A: fmt::Debug,
        T: FnMut(usize, usize) -> Result<(), ReportCallbackError<R, A>>,
    {
        let maze_mode = props.0;
        let (
            GameMode {
                size: maze_size,
                is_tower,
            },
            generation_func,
            callback,
        ) = props;

        let msize: Dims3D = Dims3D(maze_size.0, maze_size.1, maze_size.2);

        let player_pos = Dims3D(0, 0, 0);
        let goal_pos = Dims3D(msize.0 - 1, msize.1 - 1, msize.2 - 1);

        let maze = generation_func(msize, is_tower, callback)?;

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
        slow: bool,
        tower_auto_up: bool,
    ) -> Result<(Dims3D, usize), GameNotRunningError> {
        self.check_running()?;

        if slow {
            if self.maze.get_cells()[self.player_pos.2 as usize][self.player_pos.1 as usize]
                [self.player_pos.0 as usize]
                .get_wall(dir)
            {
                Ok((self.player_pos, 0))
            } else {
                self.moves.push((self.player_pos, dir));
                self.player_pos += dir.to_coord();
                Ok((self.player_pos, 1))
            }
        } else {
            let mut count = 0;
            loop {
                let mut cell = &self.maze.get_cells()[self.player_pos.2 as usize]
                    [self.player_pos.1 as usize][self.player_pos.0 as usize];

                if cell.get_wall(dir) {
                    break Ok((self.player_pos, count));
                }

                count += 1;

                self.moves.push((self.player_pos, dir));
                self.player_pos += dir.to_coord();

                cell = &self.maze.get_cells()[self.player_pos.2 as usize]
                    [self.player_pos.1 as usize][self.player_pos.0 as usize];

                let perps = dir.perpendicular_walls();
                if !cell.get_wall(perps.0)
                    || !cell.get_wall(perps.1)
                    || !cell.get_wall(perps.2)
                    || !cell.get_wall(perps.3)
                {
                    break Ok((self.player_pos, count));
                }
            }?;

            if tower_auto_up
                && self.game_mode.is_tower
                && self
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
