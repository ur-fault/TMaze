use cmaze::{core::Dims, gameboard::CellWall};
use crossterm::event::{MouseButton, MouseEvent, MouseEventKind};

use crate::{
    app::app::AppData,
    helpers::{dim::Offset, line_center},
    make_even, make_odd,
    renderer::Frame,
    settings::Settings,
    ui::{Button, Rect},
};

pub struct DPad {
    buttons: [Button; 6],
    abs_pos: Dims,
}

impl DPad {
    pub fn new(rect: Option<Rect>) -> Self {
        let rect = rect.unwrap_or_else(|| Rect::sized(Dims(11, 3)));
        let space = rect.size();

        let buttons = CellWall::get_in_order()
            .into_iter()
            .enumerate()
            .map(|(i, wall)| {
                let pos = Self::calc_button_pos(space, i);
                let size = Self::calc_button_size(space, i);

                use CellWall::*;
                let chr = match wall {
                    Top => "↑",
                    Left => "←",
                    Right => "→",
                    Bottom => "↓",
                    Up => "Up",
                    Down => "Down",
                };

                Button::new(chr.to_string(), pos, size)
            })
            .collect::<Vec<_>>()
            .try_into()
            .unwrap();

        Self {
            buttons,
            abs_pos: rect.start,
        }
    }

    pub fn styles_from_settings(&mut self, settings: &Settings) {
        self.for_mut_buttons(|button| button.load_styles_from_settings(settings));
    }

    pub fn render(&self, frame: &mut Frame) {
        self.for_buttons(|button| button.draw(frame));
    }

    /// Splits the screen into space for the viewport and the dpad.
    ///
    /// Returns the rects for the viewport and dpad, respectively.
    pub fn split_screen(data: &AppData) -> (Rect, Rect) {
        let screen_size = data.screen_size;
        let screen_ratio = (screen_size.0 as f32 / 2.0) / screen_size.1 as f32;
        let screen_rect = Rect::sized(screen_size);

        let (side, is_vertical) = match screen_ratio {
            1.0.. => (screen_size.0, false),
            ..=1.0 => (screen_size.1, true),
            _ => unreachable!(),
        };

        // TODO: load dpad ratio from settings
        let dpad_size = Offset::Rel(2. / 5.).to_abs(side).max(10);

        if is_vertical {
            screen_rect.split_y_end(Offset::Abs(dpad_size))
        } else {
            let on_right = !data.settings.get_landscape_dpad_on_left();
            let offset = Offset::Abs(dpad_size);

            if !on_right {
                let (dpad, vp) = screen_rect.split_x(offset);
                (vp, dpad)
            } else {
                screen_rect.split_x_end(offset)
            }
        }
    }

    pub fn update_space(&mut self, rect: Rect) {
        let Dims(x, y) = rect.size();
        let space = Dims(make_odd!(x), make_odd!(y));
        self.abs_pos = rect.start;

        for (i, button) in self.buttons.iter_mut().enumerate() {
            button.pos = Self::calc_button_pos(space, i);
            button.size = Self::calc_button_size(space, i);
        }
    }

    pub fn update_available_moves(&mut self, available_moves: [bool; 6]) {
        for (button, &is_available) in self.buttons.iter_mut().zip(available_moves.iter()) {
            button.disabled = !is_available;
        }
    }

    pub fn apply_mouse_event(&mut self, event: MouseEvent) -> Option<CellWall> {
        let mut touch_pos = (event.column, event.row).into();
        touch_pos -= self.abs_pos;

        let pressed = match event.kind {
            MouseEventKind::Up(MouseButton::Left) => true,
            MouseEventKind::Moved => false,
            _ => {
                return None;
            }
        };

        self.for_mut_buttons(|button| button.set = false);

        for (button, dir) in self
            .buttons
            .iter_mut()
            .zip(CellWall::get_in_order().into_iter())
        {
            if button.detect_over(touch_pos) {
                button.set = true;
                if pressed && !button.disabled {
                    return Some(dir);
                } else {
                    return None;
                }
            }
        }

        None
    }

    #[inline]
    fn calc_button_size(space: Dims, i: usize) -> Dims {
        let x = match i {
            0..=3 => make_odd!((space.0 - 1) / 2),
            4 | 5 => make_even!((space.0 - Self::calc_button_size(space, 0).0) / 2),
            _ => panic!("invalid dpad index"),
        };

        let y = make_odd!(space.1 / 3);

        Dims(x, y)
    }

    #[inline]
    fn calc_button_pos(space: Dims, i: usize) -> Dims {
        let btn_size = Self::calc_button_size(space, i);

        let x = match i {
            0 | 3 => (space.0 - btn_size.0) / 2,
            1 | 4 => 0,
            2 | 5 => space.0 - btn_size.0,
            _ => panic!("invalid dpad index"),
        };

        let y = match i {
            0 | 4 | 5 => 0,
            1 | 2 => line_center(0, space.1, btn_size.1),
            3 => space.1 - btn_size.1,
            _ => panic!("invalid dpad index"),
        };

        Dims(x, y)
    }

    #[inline]
    fn for_buttons(&self, mut f: impl FnMut(&Button)) {
        for button in self.buttons.iter() {
            f(button);
        }
    }

    #[inline]
    fn for_mut_buttons(&mut self, mut f: impl FnMut(&mut Button)) {
        for button in self.buttons.iter_mut() {
            f(button);
        }
    }
}
