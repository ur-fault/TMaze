use std::io;
pub use std::time::Duration;

use thiserror::Error;

use crate::core::*;
use crate::helpers;
use crate::renderer::helpers::term_size;

pub mod draw;
pub mod menu;
pub mod popup;
pub mod progressbar;

pub use draw::*;
pub use menu::*;
pub use popup::*;
pub use progressbar::*;

pub fn box_center_screen(box_dims: Dims) -> io::Result<Dims> {
    let size_u16 = term_size();
    Ok(helpers::box_center(
        Dims(0, 0),
        Dims(size_u16.0 as i32, size_u16.1 as i32),
        box_dims,
    ))
}

pub fn format_duration(dur: Duration) -> String {
    format!(
        "{}m{:.1}s",
        dur.as_secs() / 60,
        (dur.as_secs() % 60) as f32 + dur.subsec_millis() as f32 / 1000f32,
    )
}
