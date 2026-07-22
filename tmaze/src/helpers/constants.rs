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

pub mod paths {
    use std::path::PathBuf;

    #[cfg(not(feature = "local_paths"))]
    pub fn base() -> PathBuf {
        use dirs::preference_dir;

        preference_dir().unwrap().join("tmaze")
    }

    #[cfg(feature = "local_paths")]
    pub fn base() -> PathBuf {
        PathBuf::from("./")
    }

    pub fn theme() -> PathBuf {
        base().join("themes/")
    }

    pub fn theme_file(theme_name: &str) -> PathBuf {
        theme().join(theme_name)
    }

    pub fn config() -> PathBuf {
        base().join("settings.json5")
    }

    pub fn all_dirs() -> impl Iterator<Item = PathBuf> {
        vec![theme(), managed::path()].into_iter()
    }

    pub mod managed {
        use super::base;
        use std::path::PathBuf;

        pub fn path() -> PathBuf {
            base().join(".managed/")
        }

        pub fn ui_settings() -> PathBuf {
            path().join("ui_settings.json")
        }

        pub fn save_data() -> PathBuf {
            path().join("data.json")
        }

        pub fn log_file() -> PathBuf {
            path().join("log.txt")
        }
    }
}
