use std::time::{Duration, Instant};

use cmaze::core::Dims;

use crossterm::event::{read, KeyCode, KeyEvent, KeyEventKind};

use crate::{
    data::SaveData,
    helpers::on_off,
    logging::{self, get_logger},
    renderer::{drawable::Drawable, Renderer},
    settings::Settings,
};

#[cfg(feature = "sound")]
use crate::sound::{track::MusicTrack, SoundPlayer};

#[cfg(feature = "sound")]
use rodio::Source;

use super::{
    activity::{Activities, Activity, ActivityResult, Change},
    event::Event,
    jobs::Qer,
    Jobs,
};

#[allow(dead_code)]
pub struct App {
    renderer: Renderer,
    activities: Activities,
    data: AppData,
}

pub struct AppData {
    pub settings: Settings,
    pub save: SaveData,
    pub use_data: AppStateData,
    pub screen_size: Dims,
    jobs: Jobs,
    app_start: Instant,

    #[cfg(feature = "sound")]
    pub sound_player: SoundPlayer,
    #[cfg(feature = "sound")]
    bgm_track: Option<MusicTrack>,
}

impl AppData {
    pub fn from_start(&self) -> Duration {
        self.app_start.elapsed()
    }

    #[cfg(feature = "sound")]
    pub fn play_bgm(&mut self, track: MusicTrack) {
        if let Some(prev_track) = self.bgm_track {
            if prev_track == track {
                return;
            }
        }

        let volume = if self.settings.get_enable_audio() && self.settings.get_enable_music() {
            self.settings.get_audio_volume() * self.settings.get_music_volume()
        } else {
            0.0
        };
        self.sound_player.sink().set_volume(volume);

        self.bgm_track = Some(track);
        let track = track.get_track().repeat_infinite();
        self.sound_player.play_track(Box::new(track));
    }

    pub fn queuer(&self) -> Qer {
        self.jobs.queuer()
    }
}

impl App {
    pub fn new(base_activity: Activity) -> Self {
        let mut s = Self::empty();
        s.activities.push(base_activity);
        s
    }

    pub fn empty() -> Self {
        let renderer = Renderer::new().expect("Failed to create renderer");
        let activities = Activities::empty();

        let settings = Settings::load(Settings::default_path()).expect("Failed to load settings");
        let save = SaveData::load().expect("Failed to load save data");
        let use_data = AppStateData::default();
        let jobs = Jobs::new();
        let app_start = Instant::now();
        let frame_size = renderer.frame_size();

        logging::init();

        #[cfg(feature = "sound")]
        let sound_player = SoundPlayer::new(settings.clone());

        Self {
            renderer,
            activities,
            data: AppData {
                app_start,
                settings,
                save,
                use_data,
                screen_size: frame_size,
                jobs,

                #[cfg(feature = "sound")]
                sound_player,
                #[cfg(feature = "sound")]
                bgm_track: None,
            },
        }
    }

    pub fn run(&mut self) -> Option<ActivityResult> {
        log::trace!("Starting main loop");

        let rem_events = 'mainloop: loop {
            while let Some(job) = self.data.jobs.pop() {
                job.call(&mut self.data);
            }

            let mut events = vec![];

            let mut delay = Duration::from_millis(45);
            while let Ok(true) = crossterm::event::poll(delay) {
                let event = read().unwrap();

                self.renderer.on_event(&event);
                self.data.screen_size = self.renderer.frame_size();

                match event {
                    crossterm::event::Event::Key(KeyEvent {
                        code: KeyCode::F(3),
                        kind: KeyEventKind::Press,
                        ..
                    }) => self.switch_debug(),
                    event @ crossterm::event::Event::Mouse(_) => {
                        if self.data.settings.get_enable_mouse() {
                            events.push(Event::Term(event));
                        }
                    }
                    event => events.push(Event::Term(event)),
                }

                // just so we read all events in the frame
                delay = Duration::from_nanos(1)
            }

            while let Some(change) = match self.activities.active_mut() {
                Some(active) => {
                    log::trace!("Active activity: '{}'", active.name());
                    active
                }
                None => break 'mainloop events,
            }
            .update(std::mem::take(&mut events), &mut self.data)
            {
                match change {
                    Change::Push(activity) => {
                        log::trace!(
                            "Pushed new activity `{}/{}`",
                            activity.source(),
                            activity.name()
                        );
                        self.activities.push(activity);
                    }
                    Change::Pop { n, res } => {
                        self.activities.pop_n(n);
                        events.push(Event::ActiveAfterPop(res));
                        log::trace!("Popped {} activities", n);
                    }
                    Change::PopUntil { name, res } => {
                        self.activities.pop_until(&name);
                        events.push(Event::ActiveAfterPop(res));
                        log::trace!("Popped until '{}'", name);
                    }
                    Change::Replace(activity) => self.activities.replace(activity),
                    Change::ReplaceAt { index, activity } => {
                        self.activities.replace_at(index, activity);
                    }
                }
            }

            self.activities
                .active()
                .expect("No active active")
                .screen()
                .draw(
                    self.renderer.frame(),
                    &self.data.settings.get_color_scheme(),
                )
                .unwrap();

            logging::get_logger().draw(Dims(0, 0), self.renderer.frame());

            // TODO: let activities show debug info and about the app itself
            // then we can draw it here

            self.renderer.show().unwrap();
        };

        log::trace!("Main loop ended");

        rem_events.into_iter().find_map(|e| match e {
            Event::ActiveAfterPop(Some(res)) => Some(res),
            _ => None,
        })
    }

    fn switch_debug(&mut self) {
        self.data.use_data.show_debug = !self.data.use_data.show_debug;
        get_logger().switch_debug();
        log::warn!(
            "Debug mode: {}",
            on_off(self.data.use_data.show_debug, false)
        );
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

    pub fn data(&self) -> &AppData {
        &self.data
    }

    pub fn data_mut(&mut self) -> &mut AppData {
        &mut self.data
    }
}

#[derive(Default)]
pub struct AppStateData {
    pub last_selected_preset: Option<usize>,
    pub show_debug: bool,
}
