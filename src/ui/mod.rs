use std::time::Duration;

use crossterm::{
    event::{poll, read, Event, KeyCode, KeyEvent},
    terminal::size,
};
use masof::{Color, ContentStyle, Renderer};
use substring::Substring;

use crate::helpers;
use crate::tmcore::*;
use std::io::Stdout;

pub fn box_center_screen(box_dims: Dims) -> Result<Dims, Error> {
    let size_u16 = size()?;
    Ok(helpers::box_center(
        (0, 0),
        (size_u16.0 as i32, size_u16.1 as i32),
        box_dims,
    ))
}

pub fn draw_box(renderer: &mut Renderer, pos: Dims, size: Dims, style: ContentStyle) {
    draw_str(
        renderer,
        pos.0,
        pos.1,
        &format!("╭{}╮", "─".repeat(size.0 as usize - 2)),
        style,
    );

    for y in pos.1 + 1..pos.1 + size.1 - 1 {
        draw_char(renderer, pos.0, y, '│', style);
        draw_char(renderer, pos.0 + size.0 - 1, y, '│', style);
    }

    draw_str(
        renderer,
        pos.0,
        pos.1 + size.1 - 1,
        &format!("╰{}╯", "─".repeat(size.0 as usize - 2)),
        style,
    );
}

pub fn draw_str(renderer: &mut Renderer, mut x: i32, y: i32, mut text: &str, style: ContentStyle) {
    if y < 0 {
        return;
    }

    if x < 0 && text.len() as i32 > -x + 1 {
        text = text.substring(-x as usize, text.len() - 1);
        x = 0;
    }

    if x > u16::MAX as i32 || y > u16::MAX as i32 {
        return;
    }

    renderer.draw_str(x as u16, y as u16, text, style);
}

pub fn draw_char(renderer: &mut Renderer, mut x: i32, y: i32, mut text: char, style: ContentStyle) {
    if y < 0 || x < 0 || x > u16::MAX as i32 || y > u16::MAX as i32 {
        return;
    }

    renderer.draw_char(x as u16, y as u16, text, style);
}

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

pub fn popup_size(title: &str, texts: &[&str]) -> Dims {
    match texts.iter().map(|text| text.len()).max() {
        Some(l) => (
            2 + 2 + l.max(title.len()) as i32,
            2 + 2 + texts.len() as i32,
        ),
        None => (4 + title.len() as i32, 3),
    }
}

pub fn format_duration(dur: Duration) -> String {
    format!(
        "{}m{:.1}s",
        dur.as_secs() / 60,
        (dur.as_secs() % 60) as f32 + dur.subsec_millis() as f32 / 1000f32,
    )
}

pub fn run_menu(
    renderer: &mut Renderer,
    style: ContentStyle, stdout: &mut Stdout,
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

        render_menu(renderer,style, stdout, title, options, selected, counted)?;
    }
}

pub fn render_menu(
    renderer: &mut Renderer,
    style: ContentStyle, stdout: &mut Stdout,
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
