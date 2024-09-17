use substring::Substring;
use unicode_width::UnicodeWidthStr as _;

pub fn trim_center(text: &str, width: usize) -> &str {
    let str_width = text.width();
    if str_width <= width {
        return text;
    }

    let offset = (str_width - width) / 2;
    text.substring(offset, offset + width)
}
