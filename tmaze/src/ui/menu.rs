use crossterm::{
    event::{Event as TermEvent, KeyCode, KeyEvent},
    style::{Color, ContentStyle},
};

use pad::PadStr;
use std::cell::RefCell;

use crate::{
    app::{
        activity::{ActivityHandler, Change},
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
}

pub struct Menu {
    config: MenuConfig,
    selected: isize, // for more readable code
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
            selected: 0,
        }
    }
}

impl ActivityHandler for Menu {
    fn update(&mut self, events: Vec<Event>) -> Option<Change> {
        let opt_count = self.config.options.len() as isize;

        for event in events {
            match event {
                Event::Term(TermEvent::Key(KeyEvent { code, kind, .. }))
                    if !is_release(kind) =>
                {
                    match code {
                        KeyCode::Up | KeyCode::Char('w' | 'W') => {
                            // negative numbers wrap around zero
                            self.selected = (self.selected).rem_euclid(opt_count);
                        }
                        KeyCode::Down | KeyCode::Char('s' | 'S') => {
                            self.selected = (self.selected + 1) % opt_count
                        }
                        KeyCode::Enter | KeyCode::Char(' ') => {
                            return Some(Change::PopTop {
                                res: Some(Box::new(self.selected)),
                            })
                        }
                        KeyCode::Char(ch) => {
                            if self.config.counted {
                                self.selected = match ch {
                                    'q' | 'Q' => {
                                        return Some(Change::PopTop { res: None });
                                    }
                                    '1'..='9' => ch as isize - '1' as isize,
                                    _ => self.selected,
                                }
                                .clamp(0, opt_count - 1);
                            }
                        }
                        KeyCode::Esc => return Some(Change::PopTop { res: None }),
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
            options,
            counted,
            ..
        } = &self.config;
        let menu_size = menu_size(title, options, *counted);
        let pos = box_center_screen(menu_size);
        let opt_count = options.len();

        let max_count = opt_count.to_string().len();

        {
            let mut context = DrawContext {
                renderer: &RefCell::new(renderer),
                style: *box_style,
                frame: None,
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
        }

        Ok(())
    }
}

// pub fn menu(
//     renderer: &mut Renderer,
//     box_style: ContentStyle,
//     text_style: ContentStyle,
//     title: &str,
//     options: &[&str],
//     default: Option<usize>,
//     counted: bool,
// ) -> Result<u16, MenuError> {
//     let mut selected = default.unwrap_or(0);
//     let opt_count = options.len();
//
//     if opt_count == 0 {
//         return Err(MenuError::EmptyMenu);
//     }
//
//     let options = if default.is_some() {
//         options
//             .iter()
//             .enumerate()
//             .map(|(i, opt)| format!("{} {}", if i == default.unwrap() { "▪" } else { " " }, opt))
//             .collect::<Vec<_>>()
//     } else {
//         options
//             .iter()
//             .map(|opt| String::from(*opt))
//             .collect::<Vec<_>>()
//     };
//
//     render_menu(
//         renderer, box_style, text_style, title, &options, selected, counted,
//     )?;
//
//     loop {
//         let event = read()?;
//
//         match event {
//             Event::Key(KeyEvent { code, kind, .. }) if !is_release(kind) => match code {
//                 KeyCode::Up | KeyCode::Char('w') | KeyCode::Char('W') => {
//                     selected = if selected == 0 {
//                         opt_count - 1
//                     } else {
//                         selected - 1
//                     }
//                 }
//                 KeyCode::Down | KeyCode::Char('s') | KeyCode::Char('S') => {
//                     selected = (selected + 1) % opt_count
//                 }
//                 KeyCode::Enter | KeyCode::Char(' ') => return Ok(selected as u16),
//                 KeyCode::Char(ch) => {
//                     if counted {
//                         selected = match ch {
//                             'q' | 'Q' => return Err(MenuError::FullQuit),
//                             '1'..='9' => ch as usize - '1' as usize,
//                             _ => selected,
//                         }
//                         .clamp(0, opt_count - 1);
//                     }
//                 }
//                 KeyCode::Esc => return Err(MenuError::Exit),
//                 _ => {}
//             },
//             Event::Mouse(_) => {}
//             _ => {}
//         }
//
//         renderer.on_event(&event)?;
//
//         render_menu(
//             renderer, box_style, text_style, title, &options, selected, counted,
//         )?;
//     }
// }

// pub fn choice_menu<'a, T>(
//     renderer: &mut Renderer,
//     box_style: ContentStyle,
//     text_style: ContentStyle,
//     title: &str,
//     options: &'a [(T, &str)],
//     default: Option<usize>,
//     counted: bool,
// ) -> Result<(usize, &'a T), MenuError> {
//     let _options: Vec<&str> = options.iter().map(|opt| opt.1).collect();
//     let idx = menu(
//         renderer, box_style, text_style, title, &_options, default, counted,
//     )? as usize;
//
//     Ok((idx, &options[idx].0))
// }

// pub fn render_menu(
//     renderer: &mut Renderer,
//     box_style: ContentStyle,
//     text_style: ContentStyle,
//     title: &str,
//     options: &[String],
//     selected: usize,
//     counted: bool,
// ) -> io::Result<()> {
//     let menu_size = menu_size(title, options, counted);
//     let pos = box_center_screen(menu_size)?;
//     let opt_count = options.len();
//
//     let max_count = opt_count.to_string().len();
//
//     {
//         let mut context = DrawContext {
//             renderer: &RefCell::new(renderer),
//             style: box_style,
//             frame: None,
//         };
//
//         context.draw_box(pos, menu_size);
//
//         context.draw_str_styled(pos + Dims(3, 1), title, text_style);
//         context.draw_str(pos + Dims(1, 2), &"─".repeat(menu_size.0 as usize - 2));
//
//         for (i, option) in options.iter().enumerate() {
//             let style = if i == selected {
//                 ContentStyle {
//                     background_color: Some(text_style.foreground_color.unwrap_or(Color::White)),
//                     foreground_color: Some(text_style.background_color.unwrap_or(Color::Black)),
//                     underline_color: None,
//                     attributes: Default::default(),
//                 }
//             } else {
//                 text_style
//             };
//
//             context.draw_str_styled(
//                 pos + Dims(1, i as i32 + 3),
//                 &format!(
//                     "{} {}{}",
//                     if i == selected { ">" } else { " " },
//                     value_if(counted, || format!("{}.", i + 1)
//                         .pad_to_width((max_count as f64).log10().floor() as usize + 3)),
//                     option,
//                 )
//                 .pad_to_width(menu_size.0 as usize - 2),
//                 style,
//             );
//         }
//     }
//
//     renderer.show()?;
//
//     Ok(())
// }
