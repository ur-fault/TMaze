use std::io;
pub use std::time::Duration;

use fyodor::helpers::term_size;
pub use substring::Substring;

use crate::core::*;
use crate::helpers;
use crate::helpers::fyodor2dims;

pub mod draw;
pub mod popup;
pub mod progressbar;

pub use draw::*;
pub use popup::*;
pub use progressbar::*;

pub fn box_center_screen(box_dims: Dims) -> io::Result<Dims> {
    Ok(helpers::box_center(Dims(0, 0), fyodor2dims(term_size()), box_dims))
}

pub fn format_duration(dur: Duration) -> String {
    format!(
        "{}m{:.1}s",
        dur.as_secs() / 60,
        (dur.as_secs() % 60) as f32 + dur.subsec_millis() as f32 / 1000f32,
    )
}
