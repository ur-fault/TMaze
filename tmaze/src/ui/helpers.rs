use std::{io, time::Duration};

use cmaze::core::Dims;

use crate::{helpers, renderer::helpers::term_size};

pub fn box_center_screen(box_dims: Dims) -> Dims {
    let size_u16 = term_size();
    helpers::box_center(
        Dims(0, 0),
        Dims(size_u16.0 as i32, size_u16.1 as i32),
        box_dims,
    )
}

pub fn format_duration(dur: Duration) -> String {
    format!(
        "{}m{:.1}s",
        dur.as_secs() / 60,
        (dur.as_secs() % 60) as f32 + dur.subsec_millis() as f32 / 1000f32,
    )
}
