pub use crossterm::{
    event::{poll, read, Event, KeyCode, KeyEvent},
    terminal::size,
};
pub use masof::{Color, ContentStyle, Renderer};
use pad::PadStr;
use std::io::stdout;

use crate::helpers::value_if;

use super::draw::*;
use super::*;

pub enum MenuError {
    CrosstermError(CrosstermError),
    EmptyMenu,
    Exit,
    FullQuit,
}

impl From<CrosstermError> for MenuError {
    fn from(error: CrosstermError) -> Self {
        Self::CrosstermError(error)
    }
}

impl From<crossterm::ErrorKind> for MenuError {
    fn from(error: crossterm::ErrorKind) -> Self {
        Self::CrosstermError(error.try_into().expect("Cannot convert crossterm error"))
    }
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

pub fn menu(
    renderer: &mut Renderer,
    box_style: ContentStyle,
    text_style: ContentStyle,
    title: &str,
    options: &[&str],
    default: Option<usize>,
    counted: bool,
) -> Result<u16, MenuError> {
    let mut selected = default.unwrap_or(0);
    let opt_count = options.len();

    if opt_count == 0 {
        return Err(MenuError::EmptyMenu);
    }

    let options = if default.is_some() {
        options
            .iter()
            .enumerate()
            .map(|(i, opt)| format!("{} {}", if i == default.unwrap() { "▪" } else { " " }, opt))
            .collect::<Vec<_>>()
    } else {
        options
            .iter()
            .map(|opt| String::from(*opt))
            .collect::<Vec<_>>()
    };

    render_menu(
        renderer, box_style, text_style, title, &options, selected, counted,
    )?;

    loop {
        let event = read()?;

        match event {
            Event::Key(KeyEvent { code, modifiers: _ }) => match code {
                KeyCode::Up | KeyCode::Char('w') | KeyCode::Char('W') => {
                    selected = if selected == 0 {
                        opt_count - 1
                    } else {
                        selected - 1
                    }
                }
                KeyCode::Down | KeyCode::Char('s') | KeyCode::Char('S') => {
                    selected = (selected + 1) % opt_count
                }
                KeyCode::Enter | KeyCode::Char(' ') => return Ok(selected as u16),
                KeyCode::Char(ch) => {
                    if counted {
                        selected = match ch {
                            'q' | 'Q' => return Err(MenuError::FullQuit),
                            '1'..='9' => ch as usize - '1' as usize,
                            _ => selected,
                        }
                        .clamp(0, opt_count - 1);
                    }
                }
                KeyCode::Esc => return Err(MenuError::Exit),
                _ => {}
            },
            Event::Mouse(_) => {}
            _ => {}
        }

        renderer.event(&event);

        render_menu(
            renderer, box_style, text_style, title, &options, selected, counted,
        )?;
    }
}

pub fn choice_menu<'a, T>(
    renderer: &mut Renderer,
    box_style: ContentStyle,
    text_style: ContentStyle,
    title: &str,
    options: &'a [(T, &str)],
    default: Option<usize>,
    counted: bool,
) -> Result<&'a T, MenuError> {
    let _options: Vec<&str> = options.iter().map(|opt| opt.1).collect();
    Ok(&options[menu(
        renderer, box_style, text_style, title, &_options, default, counted,
    )? as usize]
        .0)
}

pub fn render_menu(
    renderer: &mut Renderer,
    box_style: ContentStyle,
    text_style: ContentStyle,
    title: &str,
    options: &[String],
    selected: usize,
    counted: bool,
) -> Result<(), CrosstermError> {
    let menu_size = menu_size(title, &options, counted);
    let pos = box_center_screen(menu_size)?;
    let opt_count = options.len();

    let max_count = opt_count.to_string().len();

    renderer.begin()?;

    {
        let mut context = DrawContext {
            renderer,
            style: box_style,
        };

        context.draw_box(pos, menu_size);

        context.draw_str_styled(pos.0 + 2 + 1, pos.1 + 1, &format!("{}", &title), text_style);
        context.draw_str(
            pos.0 + 1,
            pos.1 + 1 + 1,
            &"─".repeat(menu_size.0 as usize - 2),
        );

        for (i, option) in options.iter().enumerate() {
            let style = if i == selected {
                ContentStyle {
                    background_color: Some(text_style.foreground_color.unwrap_or(Color::White)),
                    foreground_color: Some(text_style.background_color.unwrap_or(Color::Black)),
                    underline_color: None,
                    attributes: Default::default(),
                }
            } else {
                text_style
            };

            context.draw_str_styled(
                pos.0 + 1,
                i as i32 + pos.1 + 2 + 1,
                &format!(
                    "{} {}{}",
                    if i == selected { ">" } else { " " },
                    value_if(counted, || format!("{}.", i + 1)
                        .pad_to_width((max_count as f64).log10().floor() as usize + 3)),
                    option,
                )
                .pad_to_width(menu_size.0 as usize - 2),
                style,
            );
        }
    }
    renderer.end(&mut stdout())?;

    Ok(())
}
