use rand::{self, seq::SliceRandom, thread_rng};

const AVAILABLE_PLAYER_CHARS: [char; 8] = ['O', '□', '◇', '☆', '○', '■', '●', '¤'];

pub const GOAL_CHAR: char = '$';

pub fn get_random_player_char() -> char {
    *AVAILABLE_PLAYER_CHARS.choose(&mut thread_rng()).unwrap()
}

pub mod colors {
    pub mod fun {
        use crossterm::style::Color;

        pub fn white() -> Color {
            Color::White
        }

        pub fn red() -> Color {
            Color::Red
        }
    }
}
