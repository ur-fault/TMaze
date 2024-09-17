use cmaze::{core::Dims, gameboard::CellWall};
use crossterm::event::{MouseButton, MouseEvent, MouseEventKind};

use crate::{
    app::app::AppData,
    helpers::dim::Offset,
    make_odd,
    renderer::Frame,
    settings::Settings,
    ui::{Button, Rect},
};

pub struct DPad {
    buttons: [Button; 4],
    abs_pos: Dims,
}

impl DPad {
    pub fn new(rect: Rect) -> Self {
        let space = rect.size();

        const BTN_DEFINITIONS: [(char, CellWall); 4] = [
            ('↑', CellWall::Up),
            ('←', CellWall::Left),
            ('→', CellWall::Right),
            ('↓', CellWall::Down),
        ];

        let buttons = BTN_DEFINITIONS
            .iter()
            .enumerate()
            .map(|(i, &(ch, _))| {
                let pos = Self::calc_button_pos(space, i);
                let size = Self::calc_button_size(space, i);

                Button::new(&ch.to_string(), pos, size)
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
        let screen_rect = Rect::sized(Dims(0, 0), screen_size);

        let (side, is_vertical) = match screen_ratio {
            r if r > 1.0 => (screen_size.0, false),
            r if r <= 1.0 => (screen_size.1, true),
            _ => unreachable!(),
        };

        // TODO: load dpad ratio from settings
        let dpad_size = Offset::Rel(2. / 5.).to_abs(side).max(10);

        if is_vertical {
            screen_rect.split_y_end(Offset::Abs(dpad_size))
        } else {
            let on_right = true; // TODO: add to the settings
            let offset = Offset::Abs(dpad_size);

            if !on_right {
                screen_rect.split_x(offset)
            } else {
                screen_rect.split_x_end(offset)
            }
        }
    }

    pub fn update_space(&mut self, rect: Rect) {
        let space = rect.size();
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

        for (i, button) in self.buttons.iter_mut().enumerate() {
            if button.detect_over(touch_pos) {
                button.set = true;
                if pressed {
                    let dir = match i {
                        0 => CellWall::Top,
                        1 => CellWall::Left,
                        2 => CellWall::Right,
                        3 => CellWall::Bottom,
                        _ => unreachable!(),
                    };

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
        let x = make_odd!((space.0 - 1) / 2);

        let y = match i {
            0 | 3 => make_odd!(space.1 / 3),
            1 | 2 => make_odd!(space.1 - (2 * Self::calc_button_size(space, 0).1)),
            _ => panic!("invalid dpad index"),
        };

        Dims(x, y)
    }

    #[inline]
    fn calc_button_pos(space: Dims, i: usize) -> Dims {
        let btn_size = Self::calc_button_size(space, i);

        let x = match i {
            0 | 3 => (space.0 - btn_size.0) / 2,
            1 => 0,
            2 => space.0 - btn_size.0,
            _ => panic!("invalid dpad index"),
        };

        let y = match i {
            0 => 0,
            1 | 2 => Self::calc_button_size(space, 0).1,
            3 => btn_size.1 + Self::calc_button_size(space, 1).1,
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
