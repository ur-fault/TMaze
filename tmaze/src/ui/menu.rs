use std::ops::RangeInclusive;

use crossterm::event::{
    Event as TermEvent, KeyCode, KeyEvent, KeyModifiers, MouseButton, MouseEvent, MouseEventKind,
};

use unicode_width::{UnicodeWidthChar, UnicodeWidthStr};

use cmaze::dims::Dims;

use crate::{
    app::{
        activity::{ActivityHandler, Change},
        app::AppData,
        ActivityEvent,
    },
    helpers::{is_release, LineDir},
    renderer::{draw::Align, CellContent, GMutView, Padding},
    settings::theme::{Style, Theme, ThemeResolver},
};

use super::{Rect, Screen, ScreenError};

pub fn panic_on_menu_push() -> ! {
    panic!("menu should only be popping itself or staying");
}

pub type MenuItemObj = Box<dyn MenuItem>;

pub trait MenuItem {
    fn min_width(&self) -> usize;

    fn render(&self, frame: &mut GMutView, style: Style);

    fn on_key(
        &mut self,
        _key: KeyCode,
        _modifiers: KeyModifiers,
        _data: &mut AppData,
    ) -> Option<Change> {
        None
    }

    fn on_mouse(
        &mut self,
        _event: MouseEventKind,
        _pos: Dims,
        _modifiers: KeyModifiers,
        _data: &mut AppData,
    ) -> Option<Change> {
        None
    }

    fn on_select(&mut self, _data: &mut AppData) -> Option<Change> {
        None
    }

    /// Returns whether this menu item can be selected by the user.
    ///
    /// *Must* stay constant for the lifetime of the menu item.
    fn selectable(&self) -> bool {
        true
    }

    /// Returns whether this menu item should be counted in the menu's numbering.
    ///
    /// *Must* stay constant for the lifetime of the menu item.
    fn indexed(&self) -> bool {
        false
    }

    fn style(&self) -> &str {
        "ui.menu.item"
    }
}

pub struct Text {
    text: String,
    first_col: char,
    last_col: char,
    indexed: bool,
    click_fn: Option<Box<dyn FnMut(&mut AppData) -> Option<Change>>>,
}

impl Text {
    pub fn new(text: impl Into<String>) -> Box<Self> {
        Box::new(Self {
            text: text.into(),
            first_col: '\0',
            last_col: '\0',
            indexed: false,
            click_fn: None,
        })
    }

    pub fn click<R: CallbackReturnValue>(
        mut self: Box<Self>,
        mut click_fn: impl FnMut(&mut AppData) -> R + 'static,
    ) -> Box<Self> {
        self.click_fn = Some(Box::new(move |d| click_fn(d).to_change()));
        self
    }

    pub fn indexed(mut self: Box<Self>) -> Box<Self> {
        self.indexed = true;
        self
    }

    pub fn first_col(mut self: Box<Self>, first_col: char) -> Box<Self> {
        self.first_col = first_col;
        self
    }

    pub fn last_col(mut self: Box<Self>, last_col: char) -> Box<Self> {
        self.last_col = last_col;
        self
    }
}

impl MenuItem for Text {
    fn min_width(&self) -> usize {
        fn col_width(c: char) -> usize {
            c.width().map(|w| w + 1).unwrap_or(0)
        }

        self.text.width() + col_width(self.first_col) + col_width(self.last_col)
    }

    fn render(&self, frame: &mut GMutView, style: Style) {
        frame.draw::<Style>(Dims(0, 0), &self.text, style);
    }

    fn on_key(&mut self, key: KeyCode, _: KeyModifiers, data: &mut AppData) -> Option<Change> {
        let Some(fun) = &mut self.click_fn else {
            return None;
        };

        match key {
            KeyCode::Enter | KeyCode::Char(' ') => fun(data),
            _ => None,
        }
    }

    fn on_select(&mut self, _data: &mut AppData) -> Option<Change> {
        if let Some(fun) = &mut self.click_fn {
            return fun(_data);
        }

        None
    }

    fn indexed(&self) -> bool {
        self.indexed
    }
}

impl From<String> for Box<Text> {
    fn from(s: String) -> Self {
        Text::new(s)
    }
}

impl From<&'static str> for Box<Text> {
    fn from(s: &'static str) -> Self {
        Text::new(s)
    }
}

impl From<String> for MenuItemObj {
    fn from(s: String) -> Self {
        Text::new(s)
    }
}

impl From<&'static str> for MenuItemObj {
    fn from(s: &'static str) -> Self {
        Text::new(s)
    }
}

pub struct Separator;

impl MenuItem for Separator {
    fn min_width(&self) -> usize {
        0
    }

    fn render(&self, frame: &mut GMutView, style: Style) {
        let line = LineDir::Horizontal.round();
        for _ in 0..frame.size().0 {
            frame.draw(Dims(0, 0), line, style);
        }
    }

    fn selectable(&self) -> bool {
        false
    }
}

pub fn separator() -> Box<dyn MenuItem> {
    Box::new(Separator)
}

pub struct Switch {
    pub text: String,
    pub val: bool,
    pub update_fn: Box<dyn FnMut(bool, &mut AppData)>,
}

impl Switch {
    pub fn new(
        text: impl Into<String>,
        val: bool,
        update_fn: impl FnMut(bool, &mut AppData) + 'static,
    ) -> Box<Self> {
        Box::new(Self {
            text: text.into(),
            val,
            update_fn: Box::new(update_fn),
        })
    }
}

impl MenuItem for Switch {
    fn min_width(&self) -> usize {
        self.text.width() + 5
    }

    fn render(&self, frame: &mut GMutView, style: Style) {
        // TODO: this is not a prefix tho ?!?
        let indicator = if self.val { "[▪]" } else { "[ ]" };
        frame.draw(Dims(0, 0), &self.text, style);
        frame.draw_aligned(Align::CenterRight, indicator, style);
    }

    fn on_key(
        &mut self,
        key: KeyCode,
        _modifiers: KeyModifiers,
        data: &mut AppData,
    ) -> Option<Change> {
        match key {
            KeyCode::Enter | KeyCode::Char(' ') => {
                self.val = !self.val;
                (self.update_fn)(self.val, data);
            }
            _ => {}
        }

        None
    }
}

pub struct Slider<V> {
    pub text: String,
    pub val: V,
    pub range: RangeInclusive<V>,
    pub step: V,
    pub update_fn: Box<dyn FnMut(V, &mut AppData) + 'static>,
    pub width: u32,
    pub display: SliderDisplay,
}

impl<V> Slider<V>
where
    V: SliderValue + 'static,
{
    pub fn new(
        text: impl Into<String>,
        val: V,
        range: RangeInclusive<V>,
        update_fn: impl FnMut(V, &mut AppData) + 'static,
    ) -> Box<Self> {
        assert!(
            range.start() <= range.end(),
            "Slider range start must be less than or equal to end"
        );

        Box::new(Self {
            text: text.into(),
            val,
            range,
            step: val.default_step(),
            update_fn: Box::new(update_fn),
            width: 5,
            display: SliderDisplay::Bar,
        })
    }

    pub fn step(mut self: Box<Self>, step: V) -> Box<Self> {
        self.step = step;
        self
    }

    pub fn width(mut self: Box<Self>, width: u32) -> Box<Self> {
        self.width = width;
        self
    }

    pub fn display(mut self: Box<Self>, display: SliderDisplay) -> Box<Self> {
        self.display = display;
        self
    }
}

impl<V> MenuItem for Slider<V>
where
    V: SliderValue,
{
    fn min_width(&self) -> usize {
        self.text.width() + self.width as usize + 4
    }

    fn render(&self, frame: &mut GMutView, style: Style) {
        frame.draw(Dims(0, 0), &self.text, style);

        let val = self.val.to_f64();
        let start = *self.range.start();
        let end = *self.range.end();
        let width = self.width as usize;

        match self.display {
            SliderDisplay::Percentage => todo!(),
            SliderDisplay::Value => {
                let val_str = if self.val.is_float() {
                    format!("[{:.2}]", val)
                } else {
                    format!("[{}", val as i64)
                };

                frame.draw_aligned(Align::CenterRight, &format!("{}", val_str), style);
            }
            SliderDisplay::Bar => {
                let filled = ((val - start.to_f64()) / (end.to_f64() - start.to_f64())
                    * width as f64)
                    .round() as usize;
                let empty = width - filled;

                let progress = format!("[{}{}]", "#".repeat(filled), " ".repeat(empty));
                frame.draw_aligned(Align::CenterRight, &progress, style);
            }
        }
    }

    fn on_key(
        &mut self,
        key: KeyCode,
        _modifiers: KeyModifiers,
        data: &mut AppData,
    ) -> Option<Change> {
        match key {
            KeyCode::Left | KeyCode::Char('a') => self.val.sub_assign(self.step, None),
            KeyCode::Right | KeyCode::Char('d') => self.val.add_assign(self.step, None),
            _ => return None,
        };
        self.val = self.val.clamp(&self.range);
        (self.update_fn)(self.val, data);

        None
    }
}

pub struct MenuConfig {
    pub title: String,
    pub subtitles: Vec<String>,
    pub options: Vec<MenuItemObj>,
    pub default: Option<usize>,
    pub q_to_quit: bool,
    pub styles: MenuStyles,

    // Callbacks
    pub on_enter: Option<Box<dyn FnMut(&mut AppData) -> Option<Change>>>,
    pub on_update: Option<Box<dyn FnMut(&mut AppData) -> Option<Change>>>,
}

impl MenuConfig {
    pub fn new_from_strings(title: impl Into<String>, options: impl Into<Vec<String>>) -> Self {
        let options: Vec<_> = Into::<Vec<_>>::into(options)
            .into_iter()
            .map(MenuItemObj::from)
            .collect();

        Self::new(title, options)
    }

    pub fn new(title: impl Into<String>, options: impl Into<Vec<MenuItemObj>>) -> Self {
        Self {
            title: title.into(),
            subtitles: vec![],
            options: options.into(),
            default: None,
            q_to_quit: true,
            // auto_select_single: false,
            styles: MenuStyles::default(),
            on_enter: None,
            on_update: None,
        }
    }

    pub fn default(mut self, default: usize) -> Self {
        self.default = Some(default);
        self
    }

    pub fn maybe_default(mut self, default: Option<usize>) -> Self {
        self.default = default;
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

    pub fn styled(mut self, styles: MenuStyles) -> Self {
        self.styles = styles;
        self
    }

    pub fn on_enter<R: CallbackReturnValue>(
        mut self,
        mut f: impl FnMut(&mut AppData) -> R + 'static,
    ) -> Self {
        self.on_enter = Some(Box::new(move |data| f(data).to_change()));
        self
    }

    pub fn on_update<R: CallbackReturnValue>(
        mut self,
        mut f: impl FnMut(&mut AppData) -> R + 'static,
    ) -> Self {
        self.on_update = Some(Box::new(move |data| f(data).to_change()));
        self
    }

    fn special_width(&self) -> usize {
        let mut special = 2;

        if self.default.is_some() {
            special += 2;
        }

        if self.indexed_count() > 0 {
            special += (self.indexed_count() as f64).log10().ceil() as usize + 2;
        };

        special
    }

    fn map_options<'s, T>(
        &'s self,
        f: impl Fn(&'s dyn MenuItem) -> T + 'static,
    ) -> impl Iterator<Item = T> + 's {
        self.options.iter().map(move |opt| f(&**opt))
    }

    fn indexed_count(&self) -> usize {
        self.options.iter().filter(|opt| opt.indexed()).count()
    }

    fn is_indexed(&self) -> bool {
        self.indexed_count() > 0
    }
}

pub struct MenuStyles {
    pub title: &'static str,
    pub subtitle: &'static str,
    pub border: &'static str,
    pub separator: &'static str,
    pub selector: &'static str,
    pub number: &'static str,
    pub item: &'static str,
}

impl MenuStyles {
    fn apply(&self, theme: &Theme) -> AppliedStyles {
        AppliedStyles {
            title: theme[self.title],
            subtitle: theme[self.subtitle],
            border: theme[self.border],
            separator: theme[self.separator],
            selector: theme[self.selector],
            number: theme[self.number],
            item: theme[self.item],
        }
    }
}

impl Default for MenuStyles {
    fn default() -> Self {
        Self {
            title: "ui.menu.title",
            subtitle: "ui.menu.subtitle",
            border: "ui.menu.border",
            separator: "ui.menu.separator",
            selector: "ui.menu.selector",
            number: "ui.menu.number",
            item: "ui.menu.item",
        }
    }
}

struct AppliedStyles {
    title: Style,
    subtitle: Style,
    border: Style,
    separator: Style,
    selector: Style,
    number: Style,
    item: Style,
}

pub struct Menu {
    config: MenuConfig,
    selected: usize,
    items_pos: Option<Rect>,
}

impl Menu {
    pub fn new(config: MenuConfig) -> Self {
        let title = config.title.clone();
        Self::try_new(config)
            .unwrap_or_else(|| panic!("invalid menu config `{}`: no options provided", title))
    }

    pub fn try_new(config: MenuConfig) -> Option<Self> {
        let MenuConfig { options, .. } = &config;
        if options.iter().all(|opt| !opt.selectable()) {
            log::warn!("Menu `{}` with no selectable options", config.title);
            return None;
        }

        let default = config.default.unwrap_or(0).clamp(0, options.len() - 1);

        Some(Self {
            selected: default,
            config,
            items_pos: None,
        })
    }

    pub fn config(&self) -> &MenuConfig {
        &self.config
    }

    fn select(&mut self, down: bool) {
        let opt_count = self.config.options.len() as isize;
        loop {
            if down {
                self.selected = (self.selected + 1) % opt_count as usize;
            } else {
                self.selected = (self.selected as isize - 1).rem_euclid(opt_count) as usize;
            }

            if self.config.options[self.selected].selectable() {
                break;
            }
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

        if !self.config.options[selected].selectable() {
            return None;
        }

        Some(selected)
    }

    fn get_selected(&mut self) -> &mut dyn MenuItem {
        &mut *self.config.options[self.selected]
    }
}

impl ActivityHandler for Menu {
    fn update(&mut self, events: Vec<ActivityEvent>, app_data: &mut AppData) -> Option<Change> {
        let opt_count = self.config.indexed_count() as isize;

        /// Return if the expression is `Some`, otherwise do nothing.
        /// Can be thought of as a `?` for `Some` instead of `None`.
        macro_rules! return_if_some {
            ($change:expr) => {
                if let Some(change) = $change {
                    return Some(change);
                }
            };
        }

        if let on_enter @ Some(_) = &mut self.config.on_enter {
            let mut swapped = None;
            std::mem::swap(&mut swapped, on_enter);
            return_if_some!(swapped.unwrap()(app_data));
        }

        if let Some(on_update) = &mut self.config.on_update {
            return_if_some!(on_update(app_data));
        }

        for event in events {
            match event {
                ActivityEvent::Term(TermEvent::Key(KeyEvent {
                    code,
                    kind,
                    modifiers,
                    ..
                })) if !is_release(kind) => match code {
                    KeyCode::Up | KeyCode::Char('w') => {
                        self.select(false);
                    }
                    KeyCode::Down | KeyCode::Char('s') => {
                        self.select(true);
                    }
                    KeyCode::Char('q') if !self.config.q_to_quit => return Some(Change::pop_top()),
                    KeyCode::Char('q') if self.config.q_to_quit => return Some(Change::pop_all()),
                    KeyCode::Char(ch @ '1'..='9') if self.config.is_indexed() => {
                        let old_sel = self.selected;
                        let index = (ch as isize - '1' as isize).clamp(0, opt_count - 1) as usize;
                        if let Some((i, _)) = self
                            .config
                            .options
                            .iter()
                            .enumerate()
                            .filter(|(_, opt)| opt.indexed())
                            .nth(index)
                        {
                            self.selected = i;
                        }

                        if old_sel == self.selected {
                            return self.get_selected().on_select(app_data);
                        }
                    }
                    KeyCode::Esc => return Some(Change::pop_top()),
                    code => {
                        return_if_some!(self.get_selected().on_key(code, modifiers, app_data));
                    }
                },
                ActivityEvent::Term(TermEvent::Mouse(MouseEvent {
                    kind,
                    column,
                    row,
                    modifiers,
                })) => {
                    let mouse_pos = (column, row).into();
                    match kind {
                        MouseEventKind::Moved => {
                            if let Some(selected) = self.get_opt_by_mouse_pos(mouse_pos) {
                                self.selected = selected;
                            }
                        }
                        MouseEventKind::ScrollDown if modifiers == KeyModifiers::empty() => {
                            self.select(true);
                        }
                        MouseEventKind::ScrollUp if modifiers == KeyModifiers::empty() => {
                            self.select(false);
                        }
                        MouseEventKind::Up(MouseButton::Left) => {
                            if let Some(selected) = self.get_opt_by_mouse_pos(mouse_pos) {
                                self.selected = selected;
                                // register the click only on the item itself
                                return_if_some!(self.get_selected().on_select(app_data));
                            }
                        }
                        MouseEventKind::Up(MouseButton::Right) => {
                            return Some(Change::pop_top());
                        }
                        kind => {
                            return_if_some!(self
                                .get_selected()
                                .on_mouse(kind, mouse_pos, modifiers, app_data))
                        }
                    }
                }
                _ => {}
            }
        }

        None
    }

    fn screen(&mut self) -> &mut dyn Screen {
        self
    }
}

impl Screen for Menu {
    fn draw(&mut self, frame: &mut GMutView, theme: &Theme) -> Result<(), ScreenError> {
        let MenuConfig {
            title,
            options,
            subtitles,
            ..
        } = &self.config;
        let AppliedStyles {
            title: title_style,
            subtitle: subtitle_style,
            border: border_style,
            separator: separator_style,
            selector: selector_style,
            number: number_style,
            item: item_style,
        } = self.config.styles.apply(theme);

        let MenuDimenstions {
            size,
            title: title_pos,
            subtitles: subtitles_pos,
            separator,
            items,
            item_content_margin,
            counts,
            item_texts,
        } = MenuDimenstions::calc(&self.config);

        if frame.size().0 < size.0 || frame.size().1 < size.1 {
            return Err(ScreenError::SmallScreen);
        }

        frame.centered(size, |f| {
            f.border(border_style);

            f.draw(title_pos, title, title_style);

            for (i, subtitle) in subtitles.iter().enumerate() {
                f.draw(subtitles_pos + Dims(0, i as i32), subtitle, subtitle_style);
            }
            f.draw(Dims(0, 0), separator, separator_style);

            f.bounds(items, |f| {
                self.items_pos = Some(f.absolute_bounds());

                let mut indexed = 1;
                for (i, option) in options.iter().enumerate() {
                    f.xline(i as i32, |f| {
                        let prep_style = |style: Style| {
                            if i == self.selected {
                                style.invert()
                            } else {
                                style
                            }
                        };

                        f.fill(CellContent::styled(' ', prep_style(item_style)));
                        if i == self.selected {
                            f.draw(Dims(0, 0), "> ", selector_style.invert());
                        }

                        let padding = Padding {
                            right: item_content_margin,
                            left: item_texts,
                            ..Padding::default()
                        };
                        f.pad(padding, |f| option.render(f, prep_style(item_style)));

                        if option.indexed() {
                            f.draw(
                                Dims(counts, 0),
                                format!("{}. ", indexed,),
                                prep_style(number_style),
                            );
                            indexed += 1;
                        }
                    });
                }
            });
        });

        Ok(())
    }
}

struct MenuDimenstions {
    size: Dims,
    title: Dims,
    subtitles: Dims,
    separator: Rect,
    items: Rect,
    item_content_margin: i32,
    counts: i32,
    item_texts: i32,
}

impl MenuDimenstions {
    fn calc(config: &MenuConfig) -> Self {
        let menu_size = {
            let subtitles_width = config
                .subtitles
                .iter()
                .map(|s| s.width())
                .max()
                .unwrap_or(0);

            let items_width = config
                .map_options(move |opt| opt.min_width())
                .max()
                .unwrap_or(0);

            let width = subtitles_width
                .max(items_width + 2 + config.special_width())
                .max(config.title.width() + 4)
                + 2;

            let width = width;
            let height = config.options.len() + 4 + config.subtitles.len();

            Dims(width as i32, height as i32)
        };

        let items = Rect::sized_at(
            Dims(1, config.subtitles.len() as i32 + 3),
            Dims(menu_size.0 - 2, config.options.len() as i32),
        );

        let count_pos = if config.is_indexed() { 2 } else { 0 };

        let item_texts = if config.is_indexed() {
            let max_count = config.indexed_count().to_string().len();
            2 + max_count as i32 + 2
        } else {
            2
        };

        let separator = Rect::sized_at(
            Dims(1, config.subtitles.len() as i32 + 2),
            Dims(menu_size.0 - 2, 1),
        );

        Self {
            size: menu_size,
            title: Dims(3, 1),
            subtitles: Dims(2, 2),
            separator,
            items,
            item_content_margin: 2,
            counts: count_pos,
            item_texts,
        }
    }
}

pub type MenuAction<R> = Box<dyn Fn(&mut AppData) -> R>;

#[macro_export]
macro_rules! menu_actions_2 {
    (move $($name:literal $(on $feature:literal)? -> $data:pat => $action:expr),* $(,)?) => {
        {
            let opts: Vec<$crate::ui::menu::MenuItemObj> = vec![
                $(
                    $(#[cfg(feature = $feature)])?
                    $crate::ui::menu::Text::new($name).click(move |$data: &mut AppData| Some($action)),
                )*
            ];

            opts
        }
    };

    ($($name:literal $(on $feature:literal)? -> $data:pat => $action:expr),* $(,)?) => {
        {
            let opts: Vec<$crate::ui::menu::MenuItemObj> = vec![
                $(
                    $(#[cfg(feature = $feature)])?
                    $crate::ui::menu::Text::new($name).click(|$data: &mut AppData| Some($action)),
                )*
            ];

            opts
        }
    };
}

pub fn split_menu_actions<R>(
    actions: Vec<(MenuItemObj, MenuAction<R>)>,
) -> (Vec<MenuItemObj>, Vec<MenuAction<R>>) {
    actions.into_iter().unzip()
}

pub fn menu_theme_resolver() -> ThemeResolver {
    let mut resolver = ThemeResolver::new();

    resolver
        .link("ui.menu.border", "border")
        .link("ui.menu.text", "text")
        .link("ui.menu.title", "ui.menu.text")
        .link("ui.menu.subtitle", "ui.menu.text")
        .link("ui.menu.separator", "ui.menu.border")
        .link("ui.menu.selector", "ui.menu.text")
        .link("ui.menu.number", "ui.menu.text")
        .link("ui.menu.item", "ui.menu.text");

    resolver
}

#[derive(Debug, Clone, Default)]
pub struct SimpleMenuOptions {
    pub default: Option<usize>,
}

pub fn simple_menu(
    title: impl Into<String>,
    items: Vec<(String, Box<dyn Fn(&mut AppData) -> Change>)>,
) -> impl ActivityHandler {
    simple_menu_ex(title, items, SimpleMenuOptions::default())
}

pub fn simple_menu_ex(
    title: impl Into<String>,
    items: Vec<(String, Box<dyn Fn(&mut AppData) -> Change>)>,
    options: SimpleMenuOptions,
) -> impl ActivityHandler {
    let mut config = MenuConfig::new(
        title,
        items
            .into_iter()
            .map(|(text, action)| {
                Text::new(text).click(move |data| Some(action(data))) as MenuItemObj
            })
            .collect::<Vec<_>>(),
    );

    if let Some(default) = options.default {
        config = config.maybe_default(Some(default));
    }

    Menu::new(config)
}

pub trait SliderValue: PartialOrd + Copy + ToString {
    fn from_f64(val: f64) -> Self;

    fn to_f64(&self) -> f64;

    fn add_assign(&mut self, other: Self, saturate: Option<Self>);

    fn sub_assign(&mut self, other: Self, saturate: Option<Self>);

    fn default_step(&self) -> Self;

    fn clamp(&self, range: &RangeInclusive<Self>) -> Self {
        if self < range.start() {
            *range.start()
        } else if self > range.end() {
            *range.end()
        } else {
            *self
        }
    }

    fn is_float(&self) -> bool;
}

macro_rules! impl_slider_value_int {
    ($($t:ty : $s:expr),*) => {
        $(
            impl SliderValue for $t {
                fn from_f64(val: f64) -> Self {
                    val as $t
                }

                fn to_f64(&self) -> f64 {
                    *self as _
                }

                fn add_assign(&mut self, other: Self, saturate: Option<Self>) {
                    match (self.checked_add(other), saturate) {
                        (Some(new_val), Some(sat)) if new_val > sat => *self = sat,
                        (Some(new_val), _) => *self = new_val,
                        (None, Some(sat)) => *self = sat,
                        (None, None) => {}
                    }
                }

                fn sub_assign(&mut self, other: Self, saturate: Option<Self>) {
                    match (self.checked_sub(other), saturate) {
                        (Some(new_val), Some(sat)) if new_val < sat => *self = sat,
                        (Some(new_val), _) => *self = new_val,
                        (None, Some(sat)) => *self = sat,
                        (None, None) => {}
                    }
                }

                fn default_step(&self) -> Self { $s }

                fn is_float(&self) -> bool { false }
            }
        )*
    };
}

macro_rules! impl_slider_value_float {
    ($($t:ty : $s:expr),*) => {
        $(
            impl SliderValue for $t {
                fn from_f64(val: f64) -> Self {
                    val as $t
                }

                fn to_f64(&self) -> f64 {
                    *self as _
                }

                fn add_assign(&mut self, other: Self, saturate: Option<Self>) {
                    let new_val = *self + other;
                    match (new_val.is_nan(), saturate) {
                        (true, Some(sat)) => *self = sat,
                        (true, None) => {}
                        (false, Some(sat)) if new_val > sat => *self = sat,
                        _ => *self = new_val,
                    }
                }

                fn sub_assign(&mut self, other: Self, saturate: Option<Self>) {
                    let new_val = *self - other;
                    match (new_val.is_nan(), saturate) {
                        (true, Some(sat)) => *self = sat,
                        (true, None) => {}
                        (false, Some(sat)) if new_val < sat => *self = sat,
                        _ => *self = new_val,
                    }
                }

                fn default_step(&self) -> Self { $s }

                fn is_float(&self) -> bool { true }
            }
        )*
    };
}

impl_slider_value_int!(
    i8 : 1, i16 : 1, i32 : 1, i64 : 1, isize : 1,
    u8 : 1, u16 : 1, u32 : 1, u64 : 1, usize : 1
);

impl_slider_value_float!(
    f32 : 0.1, f64 : 0.1
);

pub enum SliderDisplay {
    Percentage,
    Value,
    Bar,
}

pub trait CallbackReturnValue {
    fn to_change(self) -> Option<Change>;
}

impl CallbackReturnValue for Option<Change> {
    fn to_change(self) -> Option<Change> {
        self
    }
}

impl CallbackReturnValue for Change {
    fn to_change(self) -> Option<Change> {
        Some(self)
    }
}

impl CallbackReturnValue for () {
    fn to_change(self) -> Option<Change> {
        None
    }
}
