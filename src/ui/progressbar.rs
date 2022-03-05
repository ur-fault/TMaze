use crate::tmcore::*;
pub use crossterm::{
    event::{poll, read, Event, KeyCode, KeyEvent},
    terminal::size,
};
pub use masof::{Color, ContentStyle, Renderer};
use std::io::Stdout;

use super::draw::*;
use super::*;

pub fn render_progress(
    renderer: &mut Renderer,
    style: ContentStyle,
    stdout: &mut Stdout,
    title: &str,
    progress: f64,
) -> Result<(), Error> {
    let progress_size = (title.len() as i32 + 2, 4);
    let pos = box_center_screen(progress_size)?;

    renderer.begin()?;

    draw_box(renderer, pos, progress_size, style);
    if pos.1 + 1 >= 0 {
        renderer
            .draw_str(pos.0 as u16 + 1, pos.1 as u16 + 1, title, style);
    }
    if pos.1 + 2 >= 0 {
        draw_str(
            renderer,
            pos.0 + 1,
            pos.1 + 2,
            &"#".repeat((title.len() as f64 * progress) as usize),
            style,
        );
    }

    renderer.end(stdout)?;

    Ok(())
}
