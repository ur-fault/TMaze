use std::cell::RefCell;

use crossterm::style::ContentStyle;

use super::draw::*;
use super::*;
use crate::renderer::Renderer;

pub fn render_progress(
    renderer: &mut Renderer,
    box_style: ContentStyle,
    text_style: ContentStyle,
    title: &str,
    progress: f64,
) -> io::Result<()> {
    let progress_size = Dims(title.len() as i32 + 2, 4);
    let pos = box_center_screen(progress_size);

    {
        let mut context = DrawContext {
            frame: &RefCell::new(renderer.frame()),
            style: box_style,
            rect: None,
        };

        context.draw_box(pos, progress_size);
        if pos.1 + 1 >= 0 {
            context.draw_str_styled(pos + Dims(1, 1), title, text_style);
        }
        if pos.1 + 2 >= 0 {
            context.draw_str(
                pos + Dims(1, 2),
                &"█".repeat((title.len() as f64 * progress) as usize),
            );
        }
    }

    renderer.show()?;

    Ok(())
}
