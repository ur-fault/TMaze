use std::time::Duration;

use crossterm::event::read;

use crate::{
    logging,
    renderer::{drawable::Drawable, Renderer},
};

#[cfg(feature = "sound")]
use crate::sound::{track::MusicTrack, SoundPlayer};

use super::{
    activity::{Activities, Activity, ActivityResult, Change},
    event::Event,
};

#[allow(dead_code)]
pub struct App {
    renderer: Renderer,
    activities: Activities,

    #[cfg(feature = "sound")]
    sound_player: SoundPlayer,
    #[cfg(feature = "sound")]
    bgm_track: Option<MusicTrack>,
}

impl App {
    pub fn new(base_activity: Activity) -> Self {
        let renderer = Renderer::new().expect("Failed to create renderer");
        let activities = Activities::new(base_activity);

        logging::init();

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

    pub fn empty() -> Self {
        let renderer = Renderer::new().expect("Failed to create renderer");
        let activities = Activities::empty();

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

    pub fn run(&mut self) -> Option<ActivityResult> {
        log::trace!("Starting main loop");

        let rem_events = 'mainloop: loop {
            let mut events = vec![];

            let mut delay = 45;
            while let Ok(true) = crossterm::event::poll(Duration::from_millis(delay)) {
                let event = read().unwrap();

                self.renderer.on_event(&event);

                events.push(Event::Term(event));

                // just so we read all events in the frame
                delay = 1;
            }

            while let Some(change) = match self.activities.active_mut() {
                Some(active) => {
                    log::trace!("Active activity: '{}'", active.name());
                    active
                }
                None => break 'mainloop events,
            }
            .update(events.drain(..).collect())
            {
                match change {
                    Change::Push(activity) => self.activities.push(activity),
                    Change::Pop { n, res } => {
                        self.activities.pop_n(n);
                        events.push(Event::ActiveAfterPop(res));
                        log::trace!("Popped {} activities", n);
                    }
                    Change::PopTop(res) => {
                        self.activities.pop();
                        events.push(Event::ActiveAfterPop(res));
                        log::trace!("Popped top activity");
                    }
                }
            }

            self.activities
                .active()
                .expect("No active active")
                .screen()
                .draw(self.renderer.frame())
                .unwrap();

            logging::get_logger().draw((0, 0), self.renderer.frame());

            self.renderer.show().unwrap();
        };

        log::trace!("Main loop ended");

        let res = rem_events.into_iter().find_map(|e| match e {
            Event::ActiveAfterPop(Some(res)) => Some(res),
            _ => None,
        });

        res
    }

    pub fn activity_count(&self) -> usize {
        self.activities.len()
    }

    pub fn activities(&self) -> &Activities {
        &self.activities
    }

    pub fn activities_mut(&mut self) -> &mut Activities {
        &mut self.activities
    }

    pub fn active_name(&self) -> Option<&str> {
        self.activities.active().map(|a| a.name())
    }
}
