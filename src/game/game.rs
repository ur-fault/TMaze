use std::io::{stdout, Stdout};

use crate::maze::{CellWall, Maze};

use crate::maze::algorithms::*;
use crossterm::{
    event::{poll, read, Event, KeyCode, KeyEvent},
    terminal::size,
};
pub use helpers::Dims;
use masof::{Color, ContentStyle, Renderer};
use std::time::{Duration, Instant};
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
}

mod helpers {
    use crate::maze::Maze;

    pub type Dims = (u16, u16);

    pub fn menu_size(title: &str, options: &[&str], counted: bool) -> (u16, u16) {
        match options.iter().map(|opt| opt.len()).max() {
            Some(l) => (
                ((2 + if counted {
                    (options.len() + 1).to_string().len() + 2
                } else {
                    0
                } + l
                    - 2)
                .max(title.len() + 2)
                    + 2) as u16
                    + 2,
                options.len() as u16 + 2 + 2,
            ),
            None => (0, 0),
        }
    }

    pub fn line_center(container_start: u16, container_end: u16, item_width: u16) -> u16 {
        (container_end - container_start - item_width) / 2 + container_start
    }

    pub fn box_center(container_start: Dims, container_end: Dims, box_dims: Dims) -> Dims {
        (
            line_center(container_start.0, container_end.0, box_dims.0),
            line_center(container_start.1, container_end.1, box_dims.1),
        )
    }

    pub fn maze_render_size(maze: &Maze) -> Dims {
        let msize = maze.size();
        ((msize.0 * 2 + 1) as u16, (msize.1 * 2 + 1) as u16)
    }

    pub fn double_line_corner(left: bool, top: bool, right: bool, bottom: bool) -> &'static str {
        match (left, top, right, bottom) {
            (false, false, false, false) => "#",
            (false, false, false, true) => "#",
            (false, false, true, false) => "#",
            (false, false, true, true) => "╔",
            (false, true, false, false) => "#",
            (false, true, false, true) => "║",
            (false, true, true, false) => "╚",
            (false, true, true, true) => "╠",
            (true, false, false, false) => "#",
            (true, false, false, true) => "╗",
            (true, false, true, false) => "═",
            (true, false, true, true) => "╦",
            (true, true, false, false) => "╝",
            (true, true, false, true) => "╣",
            (true, true, true, false) => "╩",
            (true, true, true, true) => "╬",
        }
    }
}

pub struct GameSettings {
    slow: bool,
    show_path: bool,
}

pub struct Game {
    player: Vec<Dims>,
    start_time: Option<u64>,
    renderer: Renderer,
    stdout: Stdout,
    style: ContentStyle,
}

impl Game {
    pub fn new() -> Self {
        Game {
            player: vec![],
            start_time: None,
            renderer: Renderer::default(),
            stdout: stdout(),
            style: ContentStyle::default(),
        }
    }

    pub fn run(mut self) -> Result<(), Error> {
        self.renderer.term_on(&mut self.stdout)?;
        loop {
            match self.run_menu(
                "TMaze",
                &["New Game", "Settings", "Controls", "About", "Quit"],
                0,
                true,
            ) {
                Ok(res) => match res {
                    0 => match self.run_game() {
                        Ok(_) | Err(Error::Quit) => {}
                        Err(_) => break,
                    },

                    1 => {
                        self.run_popup("Not implemented yet")?;
                    }
                    2 => {
                        self.run_popup("Not implemented yet")?;
                    }
                    3 => {
                        self.run_popup("Not implemented yet")?;
                    }
                    4 => break,
                    _ => break,
                },
                Err(Error::Quit) => break,
                Err(_) => break,
            };
        }

        self.renderer.term_off(&mut self.stdout)?;
        Ok(())
    }

    fn run_game(&mut self) -> Result<(), Error> {
        let mut msize: (usize, usize) = match self.run_menu(
            "Maze size",
            &["10x5", "30x10", "60x30", "100x30", "debug", "xtreme"],
            0,
            false,
        )? {
            0 => (10, 5),
            1 => (30, 10),
            2 => (60, 30),
            3 => (100, 30),
            4 => (100, 100),
            5 => (500, 500),
            _ => (0, 0),
        };

        let mut player_pos: Dims = (0, 0);
        let goal_pos: Dims = (msize.0 as u16 - 1, msize.1 as u16 - 1);

        let mut maze = {
            let mut last_progress = f64::MIN;
            let generation_func = match self.run_menu(
                "Maze generation algorithm",
                &["Depth-first search", "Randomized Kruskal's"],
                0,
                true,
            )? {
                0 => DepthFirstSearch::new,
                1 => RndKruskals::new,
                _ => panic!(),
            };
            generation_func(
                msize.0,
                msize.1,
                Some((player_pos.0 as usize, player_pos.1 as usize)),
                Some(|done, all| {
                    let current_progess = done as f64 / all as f64;
                    if current_progess - last_progress > 0.01 {
                        let res = self.render_progress(
                            &format!("Generating maze ({}x{}) {}/{}", msize.0, msize.1, done, all),
                            current_progess,
                        );
                        last_progress = current_progess;

                        // check for quit keys from user
                        if let Ok(true) = poll(Duration::from_nanos(1)) {
                            if let Ok(Event::Key(KeyEvent { code, modifiers })) = read() {
                                match code {
                                    KeyCode::Esc => {
                                        return Err(Error::Quit);
                                    }
                                    KeyCode::Char('q' | 'Q') => {
                                        return Err(Error::FullQuit);
                                    }
                                    _ => {}
                                }
                            }
                        }
                        res
                    } else {
                        Ok(())
                    }
                }),
            )?
        };

        self.render_game(
            &maze,
            player_pos,
            goal_pos,
            (
                &format!("Dims: {}w{}h", maze.size().0, maze.size().1),
                "",
                "",
                "",
            ),
        )?;
        // self.render_game(&maze, player_pos, goal_pos)?;

        let start_time = Instant::now();
        let mut move_count = 0;

        loop {
            if let Ok(true) = poll(Duration::from_millis(30)) {
                let event = read();

                fn move_player(maze: &Maze, mut pos: Dims, wall: CellWall, slow: bool) -> Dims {
                    if slow {
                        if maze.get_cells()[pos.1 as usize][pos.0 as usize].get_wall(wall) {
                            pos
                        } else {
                            (
                                (pos.0 as i16 + wall.to_coord().0 as i16) as u16,
                                (pos.1 as i16 + wall.to_coord().1 as i16) as u16,
                            )
                        }
                    } else {
                        loop {
                            let mut cell = &maze.get_cells()[pos.1 as usize][pos.0 as usize];
                            if cell.get_wall(wall) {
                                break pos;
                            }
                            pos = (
                                (pos.0 as i16 + wall.to_coord().0 as i16) as u16,
                                (pos.1 as i16 + wall.to_coord().1 as i16) as u16,
                            );
                            cell = &maze.get_cells()[pos.1 as usize][pos.0 as usize];

                            let perp = wall.perpendicular_walls();
                            if !cell.get_wall(perp.0) || !cell.get_wall(perp.1) {
                                break pos;
                            }
                        }
                    }
                }

                match event {
                    Ok(Event::Key(KeyEvent { code, modifiers })) => match code {
                        KeyCode::Up | KeyCode::Char('w' | 'W') => {
                            move_count += 1;
                            player_pos = move_player(&maze, player_pos, CellWall::Top, false)
                        }
                        KeyCode::Down | KeyCode::Char('s' | 'S') => {
                            move_count += 1;
                            player_pos = move_player(&maze, player_pos, CellWall::Bottom, false)
                        }
                        KeyCode::Left | KeyCode::Char('a' | 'A') => {
                            move_count += 1;
                            player_pos = move_player(&maze, player_pos, CellWall::Left, false)
                        }
                        KeyCode::Right | KeyCode::Char('d' | 'D') => {
                            move_count += 1;
                            player_pos = move_player(&maze, player_pos, CellWall::Right, false)
                        }
                        KeyCode::Char('q' | 'Q') => break Err(Error::FullQuit),
                        KeyCode::Enter => {}
                        KeyCode::Esc => break Err(Error::Quit),
                        _ => {}
                    },
                    Err(err) => {
                        break Err(Error::CrossTermError(err));
                    }
                    _ => {}
                }

                self.renderer.event(&event.unwrap());
            }

            let from_start = Instant::now() - start_time;
            self.render_game(
                &maze,
                player_pos,
                goal_pos,
                (
                    &format!("Dims: {}w{}h", maze.size().0, maze.size().1),
                    "",
                    &format!("{} moves", move_count),
                    &format!(
                        "{}m{}s{}ms",
                        from_start.as_secs() / 60,
                        from_start.as_secs() % 60,
                        from_start.subsec_millis()
                    ),
                ),
            )?;

            // check if player won
            if player_pos == goal_pos {
                self.run_popup("You won")?;
                break Ok(());
            }
        }
    }

    fn render_progress(&mut self, title: &str, progress: f64) -> Result<(), Error> {
        let progress_size = (title.len() as u16 + 2, 4);
        let pos = self.box_center(progress_size)?;

        self.renderer.begin()?;

        self.draw_box(pos, progress_size, self.style);
        self.renderer
            .draw_str(pos.0 + 1, pos.1 + 1, title, self.style);
        self.renderer.draw_str(
            pos.0 + 1,
            pos.1 + 2,
            &"#".repeat((title.len() as f64 * progress) as usize),
            self.style,
        );

        self.renderer.end(&mut self.stdout)?;

        Ok(())
    }

    fn render_game(
        &mut self,
        maze: &Maze,
        player_pos: Dims,
        goal_pos: Dims,
        texts: (&str, &str, &str, &str),
    ) -> Result<(), Error> {
        let real_size = helpers::maze_render_size(maze);
        let pos = self.box_center(real_size)?;

        self.renderer.begin()?;

        // self.clear_screen(self.style)?;

        // corners
        self.renderer.draw_str(
            pos.0,
            pos.1,
            &format!(
                "{}{}",
                helpers::double_line_corner(false, false, true, true),
                helpers::double_line_corner(true, false, true, false)
            ),
            self.style,
        );
        self.renderer.draw_str(
            pos.0 + real_size.0 - 2,
            pos.1,
            &format!(
                "{}{}",
                helpers::double_line_corner(true, false, true, false),
                helpers::double_line_corner(true, false, false, true)
            ),
            self.style,
        );
        self.renderer.draw_str(
            pos.0,
            pos.1 + real_size.1 - 2,
            &format!("{}", helpers::double_line_corner(false, true, false, true),),
            self.style,
        );
        self.renderer.draw_str(
            pos.0,
            pos.1 + real_size.1 - 1,
            &format!("{}", helpers::double_line_corner(false, true, true, false),),
            self.style,
        );
        self.renderer.draw_str(
            pos.0 + real_size.0 - 1,
            pos.1 + real_size.1 - 2,
            &format!("{}", helpers::double_line_corner(false, true, false, true),),
            self.style,
        );
        self.renderer.draw_str(
            pos.0 + real_size.0 - 2,
            pos.1 + real_size.1 - 1,
            &format!(
                "{}{}",
                helpers::double_line_corner(true, false, true, false),
                helpers::double_line_corner(true, true, false, false)
            ),
            self.style,
        );

        // horizontal edge lines
        for x in 0..maze.size().0 - 1 {
            self.renderer.draw_str(
                x as u16 * 2 + pos.0 + 1,
                pos.1,
                &format!(
                    "{}{}",
                    helpers::double_line_corner(true, false, true, false),
                    helpers::double_line_corner(
                        true,
                        false,
                        true,
                        maze.get_cells()[0][x].get_wall(CellWall::Right),
                    )
                ),
                self.style,
            );

            self.renderer.draw_str(
                x as u16 * 2 + pos.0 + 1,
                pos.1 + real_size.1 - 1,
                &format!(
                    "{}{}",
                    helpers::double_line_corner(true, false, true, false),
                    helpers::double_line_corner(
                        true,
                        maze.get_cells()[maze.size().1 - 1][x].get_wall(CellWall::Right),
                        true,
                        false,
                    )
                ),
                self.style,
            );
        }

        // vertical edge lines
        for y in 0..maze.size().1 - 1 {
            self.renderer.draw_str(
                pos.0,
                y as u16 * 2 + pos.1 + 1,
                &format!("{}", helpers::double_line_corner(false, true, false, true)),
                self.style,
            );

            self.renderer.draw_str(
                pos.0,
                y as u16 * 2 + pos.1 + 2,
                &format!(
                    "{}",
                    helpers::double_line_corner(
                        false,
                        true,
                        maze.get_cells()[y][0].get_wall(CellWall::Bottom),
                        true,
                    )
                ),
                self.style,
            );

            self.renderer.draw_str(
                pos.0 + real_size.0 - 1,
                y as u16 * 2 + pos.1 + 1,
                &format!("{}", helpers::double_line_corner(false, true, false, true)),
                self.style,
            );

            self.renderer.draw_str(
                pos.0 + real_size.0 - 1,
                y as u16 * 2 + pos.1 + 2,
                &format!(
                    "{}",
                    helpers::double_line_corner(
                        maze.get_cells()[y][maze.size().0 as usize - 1].get_wall(CellWall::Bottom),
                        true,
                        false,
                        true,
                    )
                ),
                self.style,
            );
        }

        for (iy, row) in maze.get_cells().iter().enumerate() {
            for (ix, cell) in row.iter().enumerate() {
                if cell.get_wall(CellWall::Right) && ix != maze.size().0 - 1 {
                    self.renderer.draw_str(
                        ix as u16 * 2 + 2 + pos.0,
                        iy as u16 * 2 + 1 + pos.1,
                        helpers::double_line_corner(false, true, false, true),
                        self.style,
                    );
                }
                if cell.get_wall(CellWall::Bottom) && iy != maze.size().1 - 1 {
                    self.renderer.draw_str(
                        ix as u16 * 2 + 1 + pos.0,
                        iy as u16 * 2 + 2 + pos.1,
                        helpers::double_line_corner(true, false, true, false),
                        self.style,
                    );
                }

                if iy == maze.size().1 - 1 || ix == maze.size().0 - 1 {
                    continue;
                }

                let cell2 = &maze.get_cells()[iy + 1][ix + 1];

                self.renderer.draw_str(
                    ix as u16 * 2 + 2 + pos.0,
                    iy as u16 * 2 + 2 + pos.1,
                    helpers::double_line_corner(
                        cell.get_wall(CellWall::Bottom),
                        cell.get_wall(CellWall::Right),
                        cell2.get_wall(CellWall::Top),
                        cell2.get_wall(CellWall::Left),
                    ),
                    self.style,
                );
            }
        }

        self.renderer.draw_char(
            goal_pos.0 * 2 + 1 + pos.0,
            goal_pos.1 * 2 + 1 + pos.1,
            '$',
            ContentStyle {
                foreground_color: Some(Color::DarkYellow),
                background_color: Default::default(),
                attributes: Default::default(),
            },
        );

        self.renderer.draw_char(
            player_pos.0 * 2 + 1 + pos.0,
            player_pos.1 * 2 + 1 + pos.1,
            'O',
            ContentStyle {
                foreground_color: Some(Color::Green),
                background_color: Default::default(),
                attributes: Default::default(),
            },
        );

        let str_pos_tl = (pos.0, pos.1 - 1);
        let str_pos_tr = (pos.0 + real_size.0 - texts.1.len() as u16, pos.1 - 1);
        let str_pos_bl = (pos.0, pos.1 + real_size.1);
        let str_pos_br = (
            pos.0 + real_size.0 - texts.3.len() as u16,
            pos.1 + real_size.1,
        );

        self.renderer
            .draw_str(str_pos_tl.0, str_pos_tl.1, texts.0, self.style);
        self.renderer
            .draw_str(str_pos_tr.0, str_pos_tr.1, texts.1, self.style);
        self.renderer
            .draw_str(str_pos_bl.0, str_pos_bl.1, texts.2, self.style);
        self.renderer
            .draw_str(str_pos_br.0, str_pos_br.1, texts.3, self.style);

        self.renderer.end(&mut self.stdout)?;

        Ok(())
    }

    fn run_popup(&mut self, text: &str) -> Result<(), Error> {
        self.render_popup(text)?;

        loop {
            let event = read()?;
            if let Event::Key(KeyEvent { code, modifiers }) = event {
                break Ok(());
            }

            self.renderer.event(&event);

            self.render_popup(text)?;
        }
    }

    fn render_popup(&mut self, text: &str) -> Result<(), Error> {
        self.renderer.begin()?;

        let box_size = (text.len() as u16 + 4, 3);
        let pos = self.box_center(box_size)?;

        self.draw_box(pos, box_size, self.style);
        self.renderer
            .draw_str(pos.0 + 1, pos.1 + 1, &format!(" {} ", text), self.style);

        self.renderer.end(&mut self.stdout)?;

        Ok(())
    }

    fn run_menu(
        &mut self,
        title: &str,
        options: &[&str],
        default: usize,
        counted: bool,
    ) -> Result<u16, Error> {
        let mut selected: usize = default;
        let opt_count = options.len();

        if opt_count == 0 {
            return Err(Error::EmptyMenu);
        }

        self.render_menu(title, options, selected, counted)?;

        loop {
            let event = read()?;

            match event {
                Event::Key(KeyEvent { code, modifiers }) => match code {
                    KeyCode::Up => {
                        selected = if selected == 0 {
                            opt_count - 1
                        } else {
                            selected - 1
                        }
                    }
                    KeyCode::Down => selected = (selected + 1) % opt_count,
                    KeyCode::Char(ch) => match ch {
                        'q' | 'Q' => return Err(Error::Quit),
                        '1' if counted && 1 <= opt_count => selected = 1 - 1,
                        '2' if counted && 2 <= opt_count => selected = 2 - 1,
                        '3' if counted && 3 <= opt_count => selected = 3 - 1,
                        '4' if counted && 4 <= opt_count => selected = 4 - 1,
                        '5' if counted && 5 <= opt_count => selected = 5 - 1,
                        '6' if counted && 6 <= opt_count => selected = 6 - 1,
                        '7' if counted && 7 <= opt_count => selected = 7 - 1,
                        '8' if counted && 8 <= opt_count => selected = 8 - 1,
                        '9' if counted && 9 <= opt_count => selected = 9 - 1,
                        _ => {}
                    },
                    KeyCode::Enter => return Ok(selected as u16),
                    KeyCode::Esc => return Err(Error::Quit),
                    _ => {}
                },
                Event::Mouse(_) => {}
                _ => {}
            }

            self.renderer.event(&event);

            self.render_menu(title, options, selected, counted)?;
        }
    }

    fn render_menu(
        &mut self,
        title: &str,
        options: &[&str],
        selected: usize,
        counted: bool,
    ) -> Result<(), Error> {
        let menu_size = helpers::menu_size(title, options, counted);
        let pos = self.box_center(menu_size)?;
        let opt_count = options.len();

        let max_count = opt_count.to_string().len();

        self.renderer.begin()?;

        self.draw_box(pos, menu_size, self.style);

        self.renderer
            .draw_str(pos.0 + 2 + 1, pos.1 + 1, &format!("{}", &title), self.style);
        self.renderer.draw_str(
            pos.0 + 1,
            pos.1 + 1 + 1,
            &"─".repeat(menu_size.0 as usize - 2),
            self.style,
        );

        for (i, option) in options.iter().enumerate() {
            let style = if i == selected {
                ContentStyle {
                    background_color: Some(Color::White),
                    foreground_color: Some(Color::Black),
                    attributes: Default::default(),
                }
            } else {
                ContentStyle::default()
            };

            let off_x = if counted {
                i.to_string().len() as u16 + 2
            } else {
                0
            };

            self.renderer.draw_str(
                pos.0 + 1,
                i as u16 + pos.1 + 2 + 1,
                &format!(
                    "{} {}{}",
                    if i == selected { ">" } else { " " },
                    if counted {
                        format!(
                            "{}. {}",
                            i + 1,
                            " ".repeat(max_count - (i + 1).to_string().len())
                        )
                    } else {
                        String::from("")
                    },
                    option
                ),
                style,
            );
        }
        self.renderer.end(&mut self.stdout)?;

        Ok(())
    }

    // Helpers

    fn box_center(&self, box_dims: Dims) -> Result<Dims, Error> {
        Ok(helpers::box_center((0, 0), size()?, box_dims))
    }

    fn draw_box(&mut self, pos: Dims, size: Dims, style: ContentStyle) {
        self.renderer.draw_str(
            pos.0,
            pos.1,
            &format!("╭{}╮", "─".repeat(size.0 as usize - 2)),
            style,
        );

        for y in pos.1 + 1..pos.1 + size.1 - 1 {
            self.renderer.draw_char(pos.0, y, '│', style);
            self.renderer.draw_char(pos.0 + size.0 - 1, y, '│', style);
        }

        self.renderer.draw_str(
            pos.0,
            pos.1 + size.1 - 1,
            &format!("╰{}╯", "─".repeat(size.0 as usize - 2)),
            style,
        );
    }
}
