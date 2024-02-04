use std::io;

use cmaze::gameboard::Maze;

use rand::{seq::SliceRandom, thread_rng};

mod assets_sounds {
    pub const MUSIC_EASY: &[u8] = include_bytes!(concat!(
        env!("CARGO_MANIFEST_DIR",),
        "/assets/dist/audio/",
        "music_easy-level.",
        "mp3",
    ));
    pub const MUSIC_MEDIUM: &[u8] = include_bytes!(concat!(
        env!("CARGO_MANIFEST_DIR",),
        "/assets/dist/audio/",
        "music_medium-level.",
        "mp3",
    ));
    pub const MUSIC_HARD: &[u8] = include_bytes!(concat!(
        env!("CARGO_MANIFEST_DIR",),
        "/assets/dist/audio/",
        "music_hard-level.",
        "mp3",
    ));
    pub const MUSIC_MENU: &[u8] = include_bytes!(concat!(
        env!("CARGO_MANIFEST_DIR",),
        "/assets/dist/audio/",
        "music_menu.",
        "mp3",
    ));
}

pub type Track = Box<dyn rodio::Source<Item = i16> + Send>;

// TODO: maan, i know this is not the best
// BUT, for now it's gonna be enough
// PS: once there are mods (copium),
// this gonna need to be reworked from the ground up
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MusicTracks {
    Easy,
    Medium,
    Hard,
    Menu,
}

impl MusicTracks {
    pub fn get_data(&self) -> &'static [u8] {
        match self {
            MusicTracks::Easy => assets_sounds::MUSIC_EASY,
            MusicTracks::Medium => assets_sounds::MUSIC_MEDIUM,
            MusicTracks::Hard => assets_sounds::MUSIC_HARD,
            MusicTracks::Menu => assets_sounds::MUSIC_MENU,
        }
    }

    pub fn get_track(&self) -> Track {
        let data = Self::get_data(self);

        let cursor = io::Cursor::new(data);
        let source = rodio::Decoder::new(cursor).unwrap();

        Box::new(source)
    }

    /// Choose a random track for the Maze
    /// # Arguments
    /// * `maze` - The maze to choose the track for
    /// # Returns
    /// * A random track for the maze
    ///
    /// # Notes
    /// * We do *NOT* have yet a for determining the difficulty of the maze, so we just choose a random track
    pub fn choose_for_maze(_maze: &Maze) -> Self {
        use MusicTracks::*;
        *[Easy, Medium, Hard].choose(&mut thread_rng()).unwrap()
    }
}
