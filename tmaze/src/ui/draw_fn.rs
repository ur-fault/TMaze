use crate::{renderer::Frame, settings::theme::Style};
use cmaze::dims::*;

pub use substring::Substring;

pub fn draw_box(frame: &mut Frame, pos: Dims, size: Dims, style: Style) {
    draw_char(frame, pos.0, pos.1, '╭', style);
    draw_line(
        frame,
        Dims(pos.0 + 1, pos.1),
        false,
        size.0 as usize - 2,
        style,
    );
    draw_char(frame, pos.0 + size.0 - 1, pos.1, '╮', style);

    for y in pos.1 + 1..pos.1 + size.1 - 1 {
        draw_char(frame, pos.0, y, '│', style);
        draw_char(frame, pos.0 + size.0 - 1, y, '│', style);
    }

    let bottom = pos.1 + size.1 - 1;
    draw_char(frame, pos.0, bottom, '╰', style);
    draw_line(
        frame,
        Dims(pos.0 + 1, bottom),
        false,
        size.0 as usize - 2,
        style,
    );
    draw_char(frame, pos.0 + size.0 - 1, bottom, '╯', style);
}

pub fn draw_line(frame: &mut Frame, pos: Dims, vertical: bool, len: usize, style: Style) {
    let d = if vertical { Dims(0, 1) } else { Dims(1, 0) };
    let chr = if vertical { '│' } else { '─' };

    for i in 0..len {
        let pos = pos + d * i as i32;
        draw_char(frame, pos.0, pos.1, chr, style);
    }
}

pub fn draw_str(frame: &mut Frame, mut x: i32, y: i32, mut text: &str, style: Style) {
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

    frame.draw(Dims(x, y), text, style);
}

pub fn draw_char(frame: &mut Frame, x: i32, y: i32, text: char, style: Style) {
    if y < 0 || x < 0 || x > u16::MAX as i32 || y > u16::MAX as i32 {
        return;
    }

    frame.draw(Dims(x, y), text, style);
}
