pub fn term_size() -> (u16, u16) {
    let (w, h) = crossterm::terminal::size().unwrap_or((100, 100));
    (w, h)
}
