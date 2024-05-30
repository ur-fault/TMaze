use crossterm::{
    event::{Event as TermEvent, KeyCode, KeyEvent},
    style::{Color, ContentStyle},
};

use pad::PadStr;
use unicode_width::UnicodeWidthStr;

use std::{
    fmt::{self, Write},
    io,
    ops::RangeInclusive,
};

use cmaze::core::Dims;

use crate::{
    app::{
        activity::{Activity, ActivityHandler, Change},
        app::AppData,
        event::Event,
    },
    helpers::{is_release, LineDir, MbyStaticStr},
    renderer::Frame,
};

use super::{center_box_in_screen, draw_box, Screen};

pub fn panic_on_menu_push() -> ! {
    panic!("menu should only be popping itself or staying");
}

pub struct SliderDef {
    pub text: MbyStaticStr,
    pub val: i32,
    pub range: RangeInclusive<i32>,
    pub fun: Box<dyn FnMut(bool, &mut i32, &mut AppData)>,
    pub as_num: bool,
}

pub struct OptionDef {
    pub text: MbyStaticStr,
    pub val: bool,
    pub fun: Box<dyn FnMut(&mut bool, &mut AppData)>,
}

pub enum MenuItem {
    Text(MbyStaticStr),
    Option(OptionDef),
    Slider(SliderDef),
    Separator,
}

impl MenuItem {
    fn width(&self, special: usize) -> Option<usize> {
        match self {
            MenuItem::Text(text) => Some(text.width()),
            MenuItem::Option(OptionDef { text, .. }) => Some(text.width() + 4),
            MenuItem::Slider(SliderDef {
                text,
                range,
                as_num: show_as_number,
                ..
            }) => {
                assert!(range.start() <= range.end());
                assert!(
                    !(!*show_as_number && *range.start() < 0),
                    "if range is not shown as number, it must be positive"
                );

                let text_width = text.width();
                if *show_as_number {
                    let min = range.start().to_string().len();
                    let max = range.end().to_string().len();
                    Some(text_width + min.max(max) + 3)
                } else {
                    let boxes = range.end() - range.start();
                    Some(text_width + boxes as usize + 4)
                }
            }
            MenuItem::Separator => None,
        }
        .map(|w| w + special)
    }
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

impl fmt::Debug for MenuItem {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            MenuItem::Text(s) => write!(f, "Text({})", s),
            MenuItem::Option(OptionDef { text, val, .. }) => write!(f, "Option({}, {})", text, val),
            MenuItem::Slider(SliderDef {
                text, val, range, ..
            }) => write!(f, "Slider({}, {}, {:?})", text, val, range),
            MenuItem::Separator => write!(f, "Separator"),
        }
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

    fn special_width(&self) -> usize {
        let mut special = 2; // 2 is for cursor

        if self.default.is_some() {
            special += 2;
        }

        if self.counted {
            let max_num_w = (self.options.len() as f64).log10().ceil() as usize;
            special += max_num_w + 2;
        }

        special
    }

    fn render_option(&self, option: &MenuItem, width: usize) -> String {
        match option {
            MenuItem::Text(text) => text.to_string(),
            MenuItem::Option(OptionDef { text, val, .. }) => {
                let prefix = if *val { "[▪]" } else { "[ ]" };
                let text_w = text.width();
                format!("{text} {prefix:>width$}", width = width - text_w - 1)
            }
            MenuItem::Slider(SliderDef {
                text,
                val,
                as_num,
                range,
                ..
            }) => {
                if *as_num {
                    format!("[{val}] {text}")
                } else {
                    // TODO: find the best character to use
                    // const FILLED: char = '█';
                    const FILLED: char = '#';
                    // const FILLED: char = '-';

                    let count = (range.end() - range.start()) as usize;

                    let filled = (*val - range.start()) as usize;
                    let empty = count - filled;

                    let filled = FILLED.to_string().repeat(filled);
                    let empty = " ".repeat(empty);

                    let progress = filled + &empty;
                    let text_width = text.width();

                    let indicator = format!(" [{progress}]");

                    format!("{text}{indicator:>width$}", width = width - text_width)
                }
            }
            MenuItem::Separator => LineDir::Horizontal.round().to_string().repeat(width),
        }
    }

    fn map_options<'s, T>(
        &'s self,
        f: impl Fn(&'s MenuItem) -> T + 'static,
    ) -> impl Iterator<Item = T> + 's {
        self.options.iter().map(f)
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

    fn menu_size(&self, frame: &mut Frame) -> Dims {
        let special = self.config.special_width();
        let items_width = self
            .config
            .map_options(move |opt| opt.width(special).unwrap_or(0))
            .max()
            .unwrap_or(0)
            .max(self.config.title.width())
            // .max(10) // why copilot, i didn't ask for it
            .min(frame.size.0 as usize);
        let width = items_width + 2;
        let height = self.config.options.len() + 4;
        Dims(width as i32, height as i32)
    }
}

impl ActivityHandler for Menu {
    fn update(&mut self, events: Vec<Event>, app_data: &mut AppData) -> Option<Change> {
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
                            loop {
                                // negative numbers wrap around zero
                                self.selected = (self.selected - 1).rem_euclid(opt_count);
                                if !matches!(
                                    self.config.options[self.selected as usize],
                                    MenuItem::Separator
                                ) {
                                    break;
                                }
                            }
                        }
                        KeyCode::Down | KeyCode::Char('s' | 'S') => {
                            loop {
                                // negative numbers wrap around zero
                                self.selected = (self.selected + 1) % opt_count;
                                if !matches!(
                                    self.config.options[self.selected as usize],
                                    MenuItem::Separator
                                ) {
                                    break;
                                }
                            }
                        }
                        KeyCode::Enter | KeyCode::Char(' ') => {
                            let selected_opt = &mut self.config.options[self.selected as usize];

                            match selected_opt {
                                MenuItem::Text(_) => {
                                    return Some(Change::pop_top_with(self.selected as usize))
                                }
                                MenuItem::Option(OptionDef { val, fun, .. }) => fun(val, app_data),
                                MenuItem::Slider(_) | MenuItem::Separator => {}
                            }
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
                        KeyCode::Esc => return Some(Change::pop_top()),
                        KeyCode::Left => {
                            if let MenuItem::Slider(SliderDef {
                                val, range, fun, ..
                            }) = &mut self.config.options[self.selected as usize]
                            {
                                fun(false, val, app_data);
                                *val = (*val).clamp(*range.start(), *range.end());
                            }
                        }
                        KeyCode::Right => {
                            if let MenuItem::Slider(SliderDef {
                                val, range, fun, ..
                            }) = &mut self.config.options[self.selected as usize]
                            {
                                fun(true, val, app_data);
                                *val = (*val).clamp(*range.start(), *range.end());
                            }
                        }
                        KeyCode::Char(_) => {}
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

        let menu_size = self.menu_size(frame);

        let max_item_width = menu_size.0 as usize - 2 - self.config.special_width();

        let options = self
            .config
            .options
            .iter()
            .map(|opt| self.config.render_option(opt, max_item_width))
            .collect::<Vec<_>>();

        let pos = center_box_in_screen(menu_size);
        let opt_count = options.len();

        let max_count = opt_count.to_string().len();

        draw_box(frame, pos, menu_size, *box_style);

        frame.draw_styled(pos + Dims(3, 1), title.as_str(), *text_style);
        frame.draw_styled(
            pos + Dims(1, 2),
            LineDir::Horizontal
                .round()
                .to_string()
                .repeat(menu_size.0 as usize - 2),
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

            let mut buf = String::new();

            if i == self.selected as usize {
                write!(&mut buf, "> ").unwrap();
            } else {
                write!(&mut buf, "  ").unwrap();
            }

            if *counted {
                write!(&mut buf, "{:width$}. ", i + 1, width = max_count).unwrap();
            }
            write!(&mut buf, "{}", option).unwrap();

            let padded = buf.pad_to_width(menu_size.0 as usize - 2);

            frame.draw_styled(pos + Dims(1, i as i32 + 3), padded, style);
        }

        Ok(())
    }
}
