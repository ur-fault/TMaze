pub fn term_size() -> (u16, u16) {
    crossterm::terminal::size().unwrap_or((100, 100))
}
