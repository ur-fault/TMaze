use crossterm::{
    event::{Event as TermEvent, KeyCode, KeyEvent},
    style::{Color, ContentStyle},
};
use pad::PadStr;
use unicode_width::UnicodeWidthStr;

use std::io;

use cmaze::core::Dims;

use crate::{
    app::{
        activity::{Activity, ActivityHandler, Change},
        app::AppData,
        event::Event,
    },
    helpers::{is_release, value_if, MbyStaticStr},
    renderer::Frame,
};

use super::{box_center_screen, draw_box, Screen};

pub fn panic_on_menu_push() -> ! {
    panic!("menu should only be popping itself or staying");
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

pub enum MenuItem {
    Text(MbyStaticStr),
    Option(MbyStaticStr, bool, Box<dyn FnMut() -> bool>),
    Slider(MbyStaticStr, f32, Box<dyn FnMut(bool) -> f32>),
    Separator,
}

impl From<String> for MenuItem {
    fn from(s: String) -> Self {
        MenuItem::Text(s.into())
    }
}

impl From<&str> for MenuItem {
    fn from(s: &str) -> Self {
        MenuItem::Text(s.to_string().into())
    }
}

pub struct MenuConfig {
    pub box_style: ContentStyle,
    pub text_style: ContentStyle,
    pub title: String,
    pub options: Vec<MenuItem>,
    pub default: Option<usize>,
    pub counted: bool,
    pub q_to_quit: bool,
}

impl MenuConfig {
    pub fn new_from_strings(title: impl Into<String>, options: impl Into<Vec<String>>) -> Self {
        let options: Vec<_> = Into::<Vec<_>>::into(options)
            .into_iter()
            .map(MenuItem::from)
            .collect();

        Self::new(title, options)
    }

    pub fn new(title: impl Into<String>, options: impl Into<Vec<MenuItem>>) -> Self {
        Self {
            box_style: ContentStyle::default(),
            text_style: ContentStyle::default(),
            title: title.into(),
            options: options.into(),
            default: None,
            counted: false,
            q_to_quit: true,
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

    pub fn maybe_default(mut self, default: Option<usize>) -> Self {
        self.default = default;
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

    pub fn no_q(mut self) -> Self {
        self.q_to_quit = false;
        self
    }

    fn option_width(&self, option: &MenuItem) -> Option<usize> {
        let mut special = 2; // 2 is for cursor

        if self.default.is_some() {
            special += 2;
        }

        if self.counted {
            let max_num_w = (self.options.len() as f64).log10().ceil() as usize;
            special += max_num_w + 2;
        }

        match option {
            MenuItem::Text(text) => Some(text.width()),
            MenuItem::Option(text, _, _) => Some(text.width() + 4),
            MenuItem::Slider(text, _, _) => Some(text.width() + 4),
            MenuItem::Separator => None,
        }
        .map(|w| w + special)
    }

    fn render_option(&self, option: &MenuItem, width: usize) -> String {
        match option {
            MenuItem::Text(text) => text.to_string(),
            MenuItem::Option(text, selected, _) => {
                let prefix = if *selected { "[▪]" } else { "[ ]" };
                format!("{} {}", prefix, text)
            }
            MenuItem::Slider(text, value, _) => {
                let slider_len = width - text.width() - 2;
                let slider = "█".repeat((slider_len as f32 * value).round() as usize);
                let empty = " ".repeat(slider_len - slider.len());
                format!("{} [{}{}]", text, slider, empty)
            }
            MenuItem::Separator => "-".repeat(width),
        }
    }
}

pub struct Menu {
    config: MenuConfig,
    selected: isize, // isize for more readable code
}

impl Menu {
    pub fn new(config: MenuConfig) -> Self {
        let MenuConfig { options, .. } = &config;

        let default = config
            .default
            .map(|d| d as isize)
            .unwrap_or(0)
            .clamp(0, options.len() as isize - 1);

        Self {
            selected: default,
            config,
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
            return Some(Change::pop_top_with::<usize>(0));
        } else if opt_count == 0 {
            log::warn!("Empty menu, returning `None`");
            return Some(Change::pop_top());
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
                            // let selected_opt = &self.config.options[self.selected as usize];
                            return Some(Change::pop_top_with(self.selected as usize));
                        }
                        KeyCode::Char('q') if !self.config.q_to_quit => {
                            return Some(Change::pop_top())
                        }
                        KeyCode::Char('q') if self.config.q_to_quit => {
                            return Some(Change::pop_all())
                        }
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
    fn draw(&self, frame: &mut Frame) -> Result<(), io::Error> {
        let MenuConfig {
            box_style,
            text_style,
            title,
            counted,
            ..
        } = &self.config;

        let menu_size = {
            let items_width = self
                .config
                .options
                .iter()
                .map(|opt| self.config.option_width(opt).unwrap_or(0))
                .max()
                .unwrap_or(0)
                .max(title.width())
                // .max(10) // why copilot, i didn't ask for it
                .min(frame.size.0 as usize);

            let width = items_width + 2;

            let height = self.config.options.len() + 4;

            Dims(width as i32, height as i32)
        };

        let options = self
            .config
            .options
            .iter()
            .map(|opt| self.config.render_option(opt, frame.size.0 as usize))
            .collect::<Vec<_>>();

        let pos = box_center_screen(menu_size);
        let opt_count = options.len();

        let max_count = opt_count.to_string().len();

        draw_box(frame, pos, menu_size, *box_style);

        frame.draw_styled(pos + Dims(3, 1), title.as_str(), *text_style);
        frame.draw_styled(
            pos + Dims(1, 2),
            "─".repeat(menu_size.0 as usize - 2),
            *box_style,
        );

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
            let item = format!(
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
            .pad_to_width(menu_size.0 as usize - 2);

            frame.draw_styled(pos + Dims(1, i as i32 + 3), item, style);
        }

        Ok(())
    }
}
