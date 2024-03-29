use rand::{self, seq::SliceRandom, thread_rng};

const AVAILABLE_PLAYER_CHARS: [char; 8] = ['O', '□', '◇', '☆', '○', '■', '●', '¤'];

pub const GOAL_CHAR: char = '$';

pub fn get_random_player_char() -> char {
    *AVAILABLE_PLAYER_CHARS.choose(&mut thread_rng()).unwrap()
}
