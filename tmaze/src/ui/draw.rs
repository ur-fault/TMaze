use std::{
    cell::RefCell,
    ops::{Deref, DerefMut},
};

use crate::core::*;
pub use crossterm::{
    event::{poll, read, Event, KeyCode, KeyEvent},
    terminal::size,
};
pub use masof::{Color, ContentStyle, Renderer};

pub use substring::Substring;

pub fn draw_box<'a>(
    mut renderer: impl DerefMut<Target = &'a mut Renderer>,
    pos: Dims,
    size: Dims,
    style: ContentStyle,
) {
    draw_str(
        &mut *renderer,
        pos.0,
        pos.1,
        &format!("╭{}╮", "─".repeat(size.0 as usize - 2)),
        style,
    );

    for y in pos.1 + 1..pos.1 + size.1 - 1 {
        draw_char(&mut *renderer, pos.0, y, '│', style);
        draw_char(&mut *renderer, pos.0 + size.0 - 1, y, '│', style);
    }

    draw_str(
        renderer,
        pos.0,
        pos.1 + size.1 - 1,
        &format!("╰{}╯", "─".repeat(size.0 as usize - 2)),
        style,
    );
}

pub fn draw_str<'a>(
    mut renderer: impl DerefMut<Target = &'a mut Renderer>,
    mut x: i32,
    y: i32,
    mut text: &str,
    style: ContentStyle,
) {
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

pub fn draw_char<'a>(
    mut renderer: impl DerefMut<Target = &'a mut Renderer>,
    x: i32,
    y: i32,
    text: char,
    style: ContentStyle,
) {
    if y < 0 || x < 0 || x > u16::MAX as i32 || y > u16::MAX as i32 {
        return;
    }

    renderer.draw_char(x as u16, y as u16, text, style);
}

pub struct DrawContext<'a> {
    pub renderer: &'a RefCell<&'a mut Renderer>,
    pub style: ContentStyle,
}

#[allow(dead_code)]
impl<'a> DrawContext<'a> {
    pub fn draw_char(&mut self, pos: Dims, text: char) {
        draw_char(self.renderer.borrow_mut(), pos.0, pos.1, text, self.style);
    }

    pub fn draw_str(&mut self, pos: Dims, text: &str) {
        draw_str(self.renderer.borrow_mut(), pos.0, pos.1, text, self.style);
    }

    pub fn draw_box(&mut self, pos: Dims, size: Dims) {
        draw_box(self.renderer.borrow_mut(), pos, size, self.style);
    }

    pub fn draw_char_styled(&mut self, pos: Dims, text: char, style: ContentStyle) {
        draw_char(self.renderer.borrow_mut(), pos.0, pos.1, text, style);
    }

    pub fn draw_str_styled(&mut self, pos: Dims, text: &str, style: ContentStyle) {
        draw_str(self.renderer.borrow_mut(), pos.0, pos.1, text, style);
    }

    pub fn draw_box_styled(&mut self, pos: Dims, size: Dims, style: ContentStyle) {
        draw_box(self.renderer.borrow_mut(), pos, size, style);
    }
}
