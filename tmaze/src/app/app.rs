use std::time::Duration;

use crossterm::event::read;

use crate::renderer::Renderer;

#[cfg(feature = "sound")]
use crate::sound::{track::MusicTrack, SoundPlayer};

use super::{
    activity::{Acitivties, Activity},
    event::Event,
};

pub struct App {
    renderer: Renderer,
    activities: Acitivties,

    #[cfg(feature = "sound")]
    sound_player: SoundPlayer,
    #[cfg(feature = "sound")]
    bgm_track: Option<MusicTrack>,
}

impl App {
    pub fn new(base_activity: Activity) -> Self {
        let renderer = Renderer::new().expect("Failed to create renderer");
        let activities = Acitivties::new(base_activity);

        #[cfg(feature = "sound")]
        let sound_player = SoundPlayer::new();

        Self {
            renderer,
            activities,
            #[cfg(feature = "sound")]
            sound_player,
            #[cfg(feature = "sound")]
            bgm_track: None,
        }
    }

    pub fn run(&mut self) {
        loop {
            let activity = self.activities.active().expect("No active activity");

            let mut events = vec![];

            if let Ok(true) = crossterm::event::poll(Duration::from_millis(90)) {
                let event = read().unwrap();

                events.push(Event::Term(event));
            }

            activity.screen().draw(&mut self.renderer.frame()).unwrap();

            self.renderer.show().unwrap();
        }
    }
}
