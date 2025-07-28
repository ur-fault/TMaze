use cmaze::{
    dims::*,
    gameboard::{CellWall, Maze},
};
use crossterm::event::{MouseButton, MouseEvent, MouseEventKind};

use crate::{
    app::app::AppData,
    helpers::line_center,
    make_even, make_odd,
    renderer::{FrameBuffer, FrameViewMut},
    settings::theme::{Theme, ThemeResolver},
    ui::{Button, ButtonStyles, Rect},
};

pub enum DPadType {
    _2D,
    _3D,
}

impl DPadType {
    pub fn from_maze(maze: &Maze) -> Self {
        if maze.size().2 > 1 {
            Self::_3D
        } else {
            Self::_2D
        }
    }

    pub fn is_2d(&self) -> bool {
        matches!(self, Self::_2D)
    }

    pub fn is_3d(&self) -> bool {
        matches!(self, Self::_3D)
    }

    pub fn button_count(&self) -> usize {
        match self {
            Self::_2D => 4,
            Self::_3D => 6,
        }
    }
}

pub struct DPad {
    buttons: smallvec::SmallVec<[Button; 6]>,
    abs_pos: Dims,
    pub swap_up_down: bool,
}

impl DPad {
    pub fn new(expected_space: Option<Rect>, swap_up_down: bool, type_: DPadType) -> Self {
        let rect = expected_space.unwrap_or_else(|| Rect::sized(Dims(11, 3)));
        let space = rect.size();

        let buttons = CellWall::get_in_order()
            .into_iter()
            .enumerate()
            .take(type_.button_count())
            .map(|(i, wall)| {
                let pos = Self::calc_button_pos(space, i, swap_up_down);
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

                let styles = ButtonStyles {
                    border: "ui.dpad.border",
                    highlight: "ui.dpad.highlight",
                    text: "ui.dpad.text",
                    disabled_border: "ui.dpad.disabled.border",
                    disabled_text: "ui.dpad.disabled.text",
                };

                Button::new(chr.to_string(), pos, size).with_styles(styles)
            })
            .collect();

        Self {
            buttons,
            abs_pos: rect.start,
            swap_up_down,
        }
    }

    pub fn disable_highlight(&mut self, disable_highlight: bool) {
        self.for_mut_buttons(|button| button.disable_highlight = disable_highlight);
    }

    pub fn render(&self, frame: &mut FrameViewMut, theme: &Theme) {
        self.for_buttons(|button| button.draw_colored(frame, theme));
    }

    /// Splits the screen into space for the viewport and the dpad.
    ///
    /// Returns a tuple containing:
    /// - Whether the dpad is vertical
    /// - Whether the dpad is on the left side
    /// - The split offset
    // pub fn split_screen(screen_size: Dims, on_left: bool) -> (bool, bool, Offset) {
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
                let (r1, r2) = screen_rect.split_x_end(offset);
                (r2, r1)
            }
        }
    }

    pub fn update_space(&mut self, rect: Rect) {
        let Dims(x, y) = rect.size();
        let space = Dims(make_odd!(x), make_odd!(y));
        self.abs_pos = rect.start;

        for (i, button) in self.buttons.iter_mut().enumerate() {
            button.pos = Self::calc_button_pos(space, i, self.swap_up_down);
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
    fn calc_button_pos(space: Dims, i: usize, swap_up_down: bool) -> Dims {
        let btn_size = Self::calc_button_size(space, i);

        let i = match (i, swap_up_down) {
            (4 | 5, true) => 9 - i,
            _ => i,
        };

        let x = match i {
            0 | 3 => (space.0 - btn_size.0) / 2,
            1 | 5 => 0,
            2 | 4 => space.0 - btn_size.0,
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

pub fn dpad_theme_resolver() -> ThemeResolver {
    let mut resolver = ThemeResolver::new();
    resolver
        .link("ui.dpad.border", "ui.button.border")
        .link("ui.dpad.highlight", "ui.button.highlight")
        .link("ui.dpad.text", "ui.button.text")
        .link("ui.dpad.disabled.border", "ui.button.disabled.border")
        .link("ui.dpad.disabled.text", "ui.button.disabled.text");

    resolver
}
