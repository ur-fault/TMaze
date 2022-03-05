use crate::tmcore::*;
pub use crossterm::{
    event::{poll, read, Event, KeyCode, KeyEvent},
    terminal::size,
};
pub use masof::{Color, ContentStyle, Renderer};
use std::io::Stdout;

use super::draw::*;
use super::*;

pub fn menu_size(title: &str, options: &[&str], counted: bool) -> Dims {
    match options.iter().map(|opt| opt.len()).max() {
        Some(l) => (
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
        None => (0, 0),
    }
}

pub fn run_menu(
    renderer: &mut Renderer,
    style: ContentStyle,
    stdout: &mut Stdout,
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

    render_menu(renderer, style, stdout, title, options, selected, counted)?;

    loop {
        let event = read()?;

        match event {
            Event::Key(KeyEvent { code, modifiers }) => match code {
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
                KeyCode::Char(ch) => match ch {
                    'q' | 'Q' => return Err(Error::FullQuit),
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
                KeyCode::Esc => return Err(Error::Quit),
                _ => {}
            },
            Event::Mouse(_) => {}
            _ => {}
        }

        renderer.event(&event);

        render_menu(renderer, style, stdout, title, options, selected, counted)?;
    }
}

pub fn render_menu(
    renderer: &mut Renderer,
    style: ContentStyle,
    stdout: &mut Stdout,
    title: &str,
    options: &[&str],
    selected: usize,
    counted: bool,
) -> Result<(), Error> {
    let menu_size = menu_size(title, options, counted);
    let pos = box_center_screen(menu_size)?;
    let opt_count = options.len();

    let max_count = opt_count.to_string().len();

    renderer.begin()?;

    draw_box(renderer, pos, menu_size, style);

    draw_str(
        renderer,
        pos.0 + 2 + 1,
        pos.1 + 1,
        &format!("{}", &title),
        style,
    );
    draw_str(
        renderer,
        pos.0 + 1,
        pos.1 + 1 + 1,
        &"─".repeat(menu_size.0 as usize - 2),
        style,
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

        draw_str(
            renderer,
            pos.0 + 1,
            i as i32 + pos.1 + 2 + 1,
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
    renderer.end(stdout)?;

    Ok(())
}