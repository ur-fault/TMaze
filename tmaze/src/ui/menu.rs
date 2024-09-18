use crossterm::{
    event::{Event as TermEvent, KeyCode, KeyEvent, MouseButton, MouseEvent, MouseEventKind},
    style::ContentStyle,
};

use pad::PadStr;
use unicode_width::UnicodeWidthStr;

use std::{
    borrow::Cow,
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
    helpers::{is_release, strings::MbyStaticStr, LineDir},
    renderer::Frame,
    settings::{ColorScheme, Settings},
};

use super::{center_box_in_screen, draw_box, invert_style, Rect, Screen};

pub fn panic_on_menu_push() -> ! {
    panic!("menu should only be popping itself or staying");
}

pub struct SliderDef {
    pub text: MbyStaticStr,
    pub val: i32,
    pub range: RangeInclusive<i32>,
    #[allow(clippy::type_complexity)]
    pub fun: Box<dyn FnMut(bool, &mut i32, &mut AppData)>,
    pub as_num: bool,
}

pub struct OptionDef {
    pub text: MbyStaticStr,
    pub val: bool,
    #[allow(clippy::type_complexity)]
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
                    (*show_as_number || *range.start() >= 0),
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

    // TODO: same as Display, make it get a buffer and write to it,
    // so we don't allocate a new string every time
    fn render(&self, width: usize) -> Cow<str> {
        match self {
            MenuItem::Text(text) => text.as_ref_cow(),
            MenuItem::Option(OptionDef { text, val, .. }) => {
                let prefix = if *val { "[▪]" } else { "[ ]" };
                let text_w = text.width();
                format!("{text} {prefix:>width$}", width = width - text_w - 1).into()
            }
            MenuItem::Slider(SliderDef {
                text,
                val,
                as_num,
                range,
                ..
            }) => {
                if *as_num {
                    format!("[{val}] {text}").into()
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

                    format!("{text}{indicator:>width$}", width = width - text_width).into()
                }
            }
            MenuItem::Separator => LineDir::Horizontal.round().to_string().repeat(width).into(),
        }
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
    pub box_style: Option<ContentStyle>,
    pub text_style: Option<ContentStyle>,
    pub title_style: Option<ContentStyle>,
    pub subtitle_style: Option<ContentStyle>,
    pub title: String,
    pub subtitles: Vec<String>,
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
            box_style: None,
            text_style: None,
            title_style: None,
            subtitle_style: None,
            title: title.into(),
            subtitles: vec![],
            options: options.into(),
            default: None,
            counted: false,
            q_to_quit: true,
        }
    }

    pub fn styles_from_settings(mut self, settings: &Settings) -> Self {
        let colorscheme = settings.get_color_scheme();
        self.box_style = Some(colorscheme.normals());
        self.text_style = Some(colorscheme.texts());
        self
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
        self.box_style = Some(style);
        self
    }

    pub fn text_style(mut self, style: ContentStyle) -> Self {
        self.text_style = Some(style);
        self
    }

    pub fn title_style(mut self, style: ContentStyle) -> Self {
        self.title_style = Some(style);
        self
    }

    pub fn subtitle_style(mut self, style: ContentStyle) -> Self {
        self.subtitle_style = Some(style);
        self
    }

    pub fn no_q(mut self) -> Self {
        self.q_to_quit = false;
        self
    }

    pub fn subtitle(mut self, subtitle: impl Into<String>) -> Self {
        self.subtitles.push(subtitle.into());
        self
    }

    pub fn subtitles(mut self, subtitles: impl Into<Vec<String>>) -> Self {
        self.subtitles.extend(subtitles.into());
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

    fn map_options<'s, T>(
        &'s self,
        f: impl Fn(&'s MenuItem) -> T + 'static,
    ) -> impl Iterator<Item = T> + 's {
        self.options.iter().map(f)
    }
}

pub struct Menu {
    config: MenuConfig,
    selected: usize, // isize for more readable code
    items_pos: Option<Rect>,
}

impl Menu {
    pub fn new(config: MenuConfig) -> Self {
        let MenuConfig { options, .. } = &config;

        let default = config.default.unwrap_or(0).clamp(0, options.len() - 1);

        Self {
            selected: default,
            config,
            items_pos: None,
        }
    }

    pub fn into_activity(self) -> Activity {
        Activity::new("tmaze", "menu", Box::new(self))
    }

    fn select(&mut self, down: bool) {
        let opt_count = self.config.options.len();
        loop {
            // negative numbers wrap around zero
            if down {
                self.selected = (self.selected + 1) % opt_count;
            } else {
                self.selected =
                    (self.selected as isize - 1).rem_euclid(opt_count as isize) as usize;
            }

            // skip separators
            if !matches!(self.config.options[self.selected], MenuItem::Separator) {
                break;
            }
        }
    }

    fn switch(&mut self, data: &mut AppData) -> Option<Change> {
        let selected_opt = &mut self.config.options[self.selected];

        match selected_opt {
            MenuItem::Text(_) => return Some(Change::pop_top_with(self.selected)),
            MenuItem::Option(OptionDef { val, fun, .. }) => fun(val, data),
            MenuItem::Slider(_) | MenuItem::Separator => {}
        }

        None
    }

    fn update_slider(&mut self, right: bool, data: &mut AppData) {
        if let MenuItem::Slider(SliderDef {
            val, range, fun, ..
        }) = &mut self.config.options[self.selected]
        {
            fun(right, val, data);
            *val = (*val).clamp(*range.start(), *range.end());
        }
    }

    fn get_opt_by_mouse_pos(&self, Dims(x, y): Dims) -> Option<usize> {
        let Rect { start, end } = self.items_pos?;
        let size = end - start;

        // TODO: check using ranges instead
        if y < start.1 || y > start.1 + size.1 || x < start.0 || x > start.0 + size.0 {
            return None;
        }

        let selected = (y - start.1) as usize;

        if matches!(self.config.options[selected], MenuItem::Separator) {
            return None;
        }

        Some(selected)
    }
}

impl ActivityHandler for Menu {
    fn update(&mut self, events: Vec<Event>, app_data: &mut AppData) -> Option<Change> {
        let opt_count = self.config.options.len() as isize;
        let non_sep_count = self
            .config
            .map_options(|opt| !matches!(opt, MenuItem::Separator))
            .count() as isize;

        if non_sep_count == 1 {
            log::warn!("Menu with only one option, returning that");
            let first_non_separator = self
                .config
                .options
                .iter()
                .position(|opt| !matches!(opt, MenuItem::Separator))
                .unwrap();
            return Some(Change::pop_top_with(first_non_separator));
        } else if non_sep_count == 0 {
            log::warn!("Empty menu, returning `None`");
            return Some(Change::pop_top());
        }

        /// Return if the expression is `Some`, otherwise do nothing.
        /// Can be thought of as a `?` or `try` for `Some` instead of `None`.
        macro_rules! return_if_some {
            ($change:expr) => {
                if let Some(change) = $change {
                    return Some(change);
                }
            };
        }

        let dims = MenuDimenstions::calc(&self.config);
        self.items_pos = Some(Rect::sized_at(dims.items_pos, dims.items_size));

        for event in events {
            match event {
                Event::Term(TermEvent::Key(KeyEvent { code, kind, .. })) if !is_release(kind) => {
                    match code {
                        KeyCode::Up | KeyCode::Char('w') => {
                            self.select(false);
                        }
                        KeyCode::Down | KeyCode::Char('s') => {
                            self.select(true);
                        }
                        KeyCode::Enter | KeyCode::Char(' ') => {
                            return_if_some!(self.switch(app_data));
                        }
                        KeyCode::Char('q') if !self.config.q_to_quit => {
                            return Some(Change::pop_top())
                        }
                        KeyCode::Char('q') if self.config.q_to_quit => {
                            return Some(Change::pop_all())
                        }
                        KeyCode::Char(ch @ '1'..='9') if self.config.counted => {
                            let old_sel = self.selected;
                            self.selected =
                                (ch as isize - '1' as isize).clamp(0, opt_count - 1) as usize;

                            if old_sel == self.selected {
                                return_if_some!(self.switch(app_data));
                            }
                        }
                        KeyCode::Esc => return Some(Change::pop_top()),
                        KeyCode::Left => {
                            self.update_slider(false, app_data);
                        }
                        KeyCode::Right => {
                            self.update_slider(true, app_data);
                        }
                        _ => {}
                    }
                }
                Event::Term(TermEvent::Mouse(MouseEvent {
                    kind, column, row, ..
                })) => {
                    let mouse_pos = (column, row).into();
                    match kind {
                        MouseEventKind::Moved => {
                            if let Some(selected) = self.get_opt_by_mouse_pos(mouse_pos) {
                                self.selected = selected;
                            }
                        }
                        MouseEventKind::ScrollDown => {
                            self.select(true);
                        }
                        MouseEventKind::ScrollUp => {
                            self.select(false);
                        }
                        MouseEventKind::Up(MouseButton::Left) => {
                            if let Some(selected) = self.get_opt_by_mouse_pos(mouse_pos) {
                                self.selected = selected;
                            }
                            return_if_some!(self.switch(app_data));
                        }

                        // TODO: Test these
                        MouseEventKind::ScrollLeft => {
                            self.update_slider(false, app_data);
                        }
                        MouseEventKind::ScrollRight => {
                            self.update_slider(true, app_data);
                        }
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
    fn draw(&self, frame: &mut Frame, color_scheme: &ColorScheme) -> Result<(), io::Error> {
        let MenuConfig {
            box_style,
            text_style,
            title_style,
            subtitle_style,
            title,
            counted,
            ..
        } = &self.config;

        let title_style = title_style.or(*text_style).unwrap_or(color_scheme.texts());
        let subtitle_style = subtitle_style
            .or(*text_style)
            .unwrap_or(color_scheme.texts());

        let box_style = box_style.unwrap_or(color_scheme.normals());
        let text_style = text_style.unwrap_or(color_scheme.texts());

        let MenuDimenstions {
            size,
            title_pos,
            items_pos,
            subtitles_pos,
            ..
        } = MenuDimenstions::calc(&self.config);

        let max_item_width = size.0 as usize - 2 - self.config.special_width();

        let options = self
            .config
            .options
            .iter()
            .map(|opt| opt.render(max_item_width))
            .collect::<Vec<_>>();

        let pos = center_box_in_screen(size);
        let opt_count = options.len();

        let max_count = opt_count.to_string().len();

        draw_box(frame, pos, size, box_style);

        frame.draw_styled(title_pos, title.as_str(), title_style);

        for (i, subtitle) in self.config.subtitles.iter().enumerate() {
            frame.draw_styled(
                subtitles_pos + Dims(0, i as i32),
                subtitle.as_str(),
                subtitle_style,
            );
        }

        frame.draw_styled(
            items_pos - Dims(0, 1),
            LineDir::Horizontal
                .round()
                .to_string()
                .repeat(size.0 as usize - 2),
            box_style,
        );

        for (i, option) in options.iter().enumerate() {
            let style = if i == self.selected {
                invert_style(text_style)
            } else {
                text_style
            };

            let mut buf = String::new();

            if i == self.selected {
                write!(&mut buf, "> ").unwrap();
            } else {
                write!(&mut buf, "  ").unwrap();
            }

            if *counted {
                write!(&mut buf, "{:width$}. ", i + 1, width = max_count).unwrap();
            }
            write!(&mut buf, "{}", option).unwrap();

            let padded = buf.pad_to_width(size.0 as usize - 2);

            frame.draw_styled(items_pos + Dims(0, i as i32), padded, style);
        }

        Ok(())
    }
}

struct MenuDimenstions {
    size: Dims,
    title_pos: Dims,
    items_pos: Dims,
    items_size: Dims,
    subtitles_pos: Dims,
    // subtitles_size: Dims,
}

impl MenuDimenstions {
    fn calc(config: &MenuConfig) -> Self {
        let menu_size = {
            let special = config.special_width();

            let subtitles_width = config
                .subtitles
                .iter()
                .map(|s| s.width())
                .max()
                .unwrap_or(0);

            let items_width = config
                .map_options(move |opt| opt.width(special).unwrap_or(0))
                .max()
                .unwrap_or(0);

            let width = subtitles_width
                .max(items_width)
                .max(config.title.width() + 2) // title is offseted by 2
                + 2;

            let width = width + 2;
            let height = config.options.len() + 4 + config.subtitles.len();

            Dims(width as i32, height as i32)
        };

        let pos = center_box_in_screen(menu_size);

        let items_pos = Dims(pos.0 + 1, pos.1 + config.subtitles.len() as i32 + 3);

        Self {
            size: menu_size,
            title_pos: pos + Dims(3, 1),
            items_pos,
            items_size: Dims(menu_size.0 - 2, config.options.len() as i32),
            subtitles_pos: pos + Dims(2, 2),
            // subtitles_size: Dims(menu_size.0 - 2, config.subtitles.len() as i32),
        }
    }
}

pub type MenuAction<R> = Box<dyn Fn(&mut AppData) -> R>;

#[macro_export]
macro_rules! menu_actions {
    ($($name:literal $(on $feature:literal)? -> $data:pat => $action:expr),* $(,)?) => {
        {
            let opts: Vec<(_, MenuAction<_>)> = vec![
                $(
                    $(#[cfg(feature = $feature)])?
                    { ($crate::ui::menu::MenuItem::from($name), Box::new(|$data: &mut AppData| $action)) },
                )*
            ];

            opts
        }
    };
}

pub fn split_menu_actions<R>(
    actions: Vec<(MenuItem, MenuAction<R>)>,
) -> (Vec<MenuItem>, Vec<MenuAction<R>>) {
    let (names, actions) = actions.into_iter().unzip();

    (names, actions)
}
