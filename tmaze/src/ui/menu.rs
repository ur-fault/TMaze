use crossterm::{
    event::{Event as TermEvent, KeyCode, KeyEvent},
    style::{Color, ContentStyle},
};

use pad::PadStr;
use std::cell::RefCell;

use crate::{
    app::{
        activity::{Activity, ActivityHandler, Change},
        app::{App, AppData},
        event::Event,
    },
    helpers::{is_release, value_if},
};

use super::{draw::*, *};

#[derive(Debug, Error)]
pub enum MenuError {
    #[error(transparent)]
    CrosstermError(#[from] io::Error),
    #[error("Empty menu, nothing to select")]
    EmptyMenu,
    #[error("Exit")]
    Exit,
    #[error("Full quit")]
    FullQuit,
}

pub fn menu_size(title: &str, options: &[String], counted: bool) -> Dims {
    match options.iter().map(|opt| opt.len()).max() {
        Some(l) => Dims(
            ((2 + if counted {
                (options.len() + 1).to_string().len() + 2
            } else {
                0
            } + l
                - 2)
            .max(title.len() + 2)
                + 2) as i32
                + 2,
            options.len() as i32 + 2 + 2,
        ),
        None => Dims(0, 0),
    }
}

// TODO: `struct MenuOption` with text and other stuff,
// text should be either 'static or String

pub struct MenuConfig {
    pub box_style: ContentStyle,
    pub text_style: ContentStyle,
    pub title: String,
    pub options: Vec<String>,
    pub default: Option<usize>,
    pub counted: bool,
}

impl MenuConfig {
    pub fn new(title: impl Into<String>, options: impl Into<Vec<String>>) -> Self {
        Self {
            box_style: ContentStyle::default(),
            text_style: ContentStyle::default(),
            title: title.into(),
            options: options.into(),
            default: None,
            counted: false,
        }
    }

    pub fn counted(mut self) -> Self {
        self.counted = true;
        self
    }

    pub fn default(mut self, default: usize) -> Self {
        self.default = Some(default);
        self
    }

    pub fn box_style(mut self, style: ContentStyle) -> Self {
        self.box_style = style;
        self
    }

    pub fn text_style(mut self, style: ContentStyle) -> Self {
        self.text_style = style;
        self
    }
}

pub struct Menu {
    config: MenuConfig,
    shown_options: Vec<String>,
    selected: isize, // isize for more readable code
}

impl Menu {
    pub fn new(config: MenuConfig) -> Self {
        let MenuConfig {
            default, options, ..
        } = &config;

        let options = if default.is_some() {
            options
                .iter()
                .enumerate()
                .map(|(i, opt)| {
                    format!("{} {}", if i == default.unwrap() { "▪" } else { " " }, opt)
                })
                .collect::<Vec<_>>()
        } else {
            options
                .iter()
                .map(|opt| String::from(opt))
                .collect::<Vec<_>>()
        };

        Self {
            config,
            shown_options: options,
            selected: 0,
        }
    }

    pub fn into_activity(self) -> Activity {
        Activity::new("tmaze", "menu", Box::new(self))
    }
}

impl ActivityHandler for Menu {
    fn update(&mut self, events: Vec<Event>, _: &mut AppData) -> Option<Change> {
        let opt_count = self.config.options.len() as isize;

        if opt_count == 1 {
            log::warn!("Menu with only one option, returning that");
            return Some(Change::pop_top_with(Box::new(0)));
        } else if opt_count == 0 {
            log::warn!("Empty menu, returning `None`");
            return Some(Change::pop_top_with(Box::new(0)));
        }

        for event in events {
            match event {
                Event::Term(TermEvent::Key(KeyEvent { code, kind, .. })) if !is_release(kind) => {
                    match code {
                        KeyCode::Up | KeyCode::Char('w' | 'W') => {
                            // negative numbers wrap around zero
                            self.selected = (self.selected - 1).rem_euclid(opt_count);
                        }
                        KeyCode::Down | KeyCode::Char('s' | 'S') => {
                            self.selected = (self.selected + 1) % opt_count
                        }
                        KeyCode::Enter | KeyCode::Char(' ') => {
                            return Some(Change::pop_top_with(self.selected as usize))
                        }
                        KeyCode::Char('q' | 'Q') => return Some(Change::pop_top()),
                        KeyCode::Char(ch @ '1'..='9') if self.config.counted => {
                            self.selected = (ch as isize - '1' as isize).clamp(0, opt_count - 1);
                        }
                        KeyCode::Char(_) => {}
                        KeyCode::Esc => return Some(Change::pop_top()),
                        _ => {}
                    }
                }
                _ => {}
            }
        }

        None
    }

    fn screen(&self) -> &dyn Screen {
        self
    }
}

impl Screen for Menu {
    fn draw(&self, renderer: &mut Frame) -> Result<(), io::Error> {
        let MenuConfig {
            box_style,
            text_style,
            title,
            counted,
            ..
        } = &self.config;

        let options = &self.shown_options;

        let menu_size = menu_size(title, options, *counted);
        let pos = box_center_screen(menu_size);
        let opt_count = options.len();

        let max_count = opt_count.to_string().len();

        let mut context = DrawContext {
            frame: &RefCell::new(renderer),
            style: *box_style,
            rect: None,
        };

        context.draw_box(pos, menu_size);

        context.draw_str_styled(pos + Dims(3, 1), title, *text_style);
        context.draw_str(pos + Dims(1, 2), &"─".repeat(menu_size.0 as usize - 2));

        for (i, option) in options.iter().enumerate() {
            let style = if i == self.selected as usize {
                ContentStyle {
                    background_color: Some(text_style.foreground_color.unwrap_or(Color::White)),
                    foreground_color: Some(text_style.background_color.unwrap_or(Color::Black)),
                    underline_color: None,
                    attributes: Default::default(),
                }
            } else {
                *text_style
            };

            context.draw_str_styled(
                pos + Dims(1, i as i32 + 3),
                &format!(
                    "{} {}{}",
                    if i == self.selected as usize {
                        ">"
                    } else {
                        " "
                    },
                    value_if(*counted, || format!("{}.", i + 1)
                        .pad_to_width((max_count as f64).log10().floor() as usize + 3)),
                    option,
                )
                .pad_to_width(menu_size.0 as usize - 2),
                style,
            );
        }

        Ok(())
    }
}
