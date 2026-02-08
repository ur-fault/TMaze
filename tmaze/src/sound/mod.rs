pub mod track;

use menu::OptionDef;
use rodio::{OutputStream, OutputStreamHandle, Sink};

use crate::{
    app::{app::AppData, event::EventReceiver, Activity, Event},
    settings::Settings,
    ui::{menu, MenuItem, SliderDef},
};

use self::track::Track;

struct SoundHandles {
    _stream: OutputStream,
    handle: OutputStreamHandle,
    sink: Sink,
}

pub struct SoundPlayer {
    handles: Option<SoundHandles>,
    settings: Settings,
}

impl SoundPlayer {
    pub fn new(settings: Settings) -> Self {
        let Ok((stream, handle)) = rodio::OutputStream::try_default() else {
            log::warn!("Failed to create audio stream, no sound will be played");
            return Self {
                handles: None,
                settings,
            };
        };

        let sink = Sink::try_new(&handle).expect("Failed to create sink");

        Self {
            handles: Some(SoundHandles {
                _stream: stream,
                handle,
                sink,
            }),
            settings,
        }
    }

    #[inline]
    fn apply(&self, f: impl FnOnce(&Sink)) {
        if let Some(handles) = &self.handles {
            f(&handles.sink);
        }
    }

    #[allow(dead_code)]
    pub fn enqueue(&self, track: Track) {
        self.apply(|sink| {
            sink.append(track);
            sink.play();
        });
    }

    pub fn play_track(&self, track: Track) {
        self.apply(|sink| {
            sink.stop();
            sink.append(track);
            sink.play();
        });
    }

    #[allow(dead_code)]
    pub fn play_sound(&self, track: Track) {
        let Some(handle) = self.handles.as_ref().map(|h| &h.handle) else {
            return;
        };
        let sink = Sink::try_new(handle).expect("Failed to create sink");
        sink.set_volume(self.settings.read().audio.audio_volume as f32);
        sink.append(track);
        sink.play();
        sink.detach();
    }

    #[allow(dead_code)]
    pub fn wait(&self) {
        self.apply(|sink| sink.sleep_until_end());
    }

    pub fn set_volume(&self, volume: f32) {
        self.apply(|sink| sink.set_volume(volume));
    }
}

impl EventReceiver for &SoundPlayer {
    fn register(self) -> crate::app::event::EventReceiverFn {
        Box::new(move |event, data| {
            if let Event::SettingsChanged = event {
                let cfg = &data.settings.read().audio;

                if cfg.enable_audio && cfg.enable_music {
                    data.sound_player
                        .set_volume((cfg.audio_volume * cfg.music_volume) as f32);
                } else {
                    data.sound_player.set_volume(0.0);
                }
            }
        })
    }
}

pub fn create_audio_settings(data: &mut AppData) -> Activity {
    let config = &data.settings.read().audio;

    let menu_config = menu::MenuConfig::new(
        "Audio settings",
        [
            MenuItem::Option(OptionDef {
                text: "Global mute".into(),
                val: !config.enable_audio,
                update_fn: Box::new(|mute, data| {
                    data.settings.update_ui(|cfg| {
                        *cfg.audio().enable_audio() = !mute;
                    });
                }),
                reset_fn: Some(Box::new(|data| {
                    data.settings.update_ui(|cfg| {
                        cfg.audio().enable_audio = None;
                    });
                    !data.settings.read().audio.enable_audio
                })),
            }),
            MenuItem::Slider(SliderDef {
                text: "Global volume".into(),
                val: (config.audio_volume * 5.0) as i32,
                range: 0..=5,
                as_num: false,
                update_fn: Box::new(|vol, data| {
                    data.settings.update_ui(|cfg| {
                        *cfg.audio().audio_volume() = vol as f64 / 5.0;
                    });
                }),

                reset_fn: Some(Box::new(|data| {
                    data.settings.update_ui(|cfg| {
                        cfg.audio().audio_volume = None;
                    });
                    (data.settings.read().audio.audio_volume * 5.0) as i32
                })),
            }),
            MenuItem::Option(OptionDef {
                text: "Music mute".into(),
                val: !config.enable_music,
                update_fn: Box::new(|mute, data| {
                    data.settings.update_ui(|cfg| {
                        *cfg.audio().enable_music() = !mute;
                    });
                }),
                reset_fn: Some(Box::new(|data| {
                    data.settings.update_ui(|cfg| {
                        cfg.audio().enable_music = None;
                    });
                    !data.settings.read().audio.enable_music
                })),
            }),
            MenuItem::Slider(SliderDef {
                text: "Music volume".into(),
                val: (config.music_volume * 5.0) as i32,
                range: 0..=5,
                as_num: false,
                update_fn: Box::new(|vol, data| {
                    data.settings.update_ui(|cfg| {
                        *cfg.audio().music_volume() = vol as f64 / 5.0;
                    });
                }),
                reset_fn: Some(Box::new(|data| {
                    data.settings.update_ui(|cfg| {
                        cfg.audio().music_volume = None;
                    });
                    (data.settings.read().audio.music_volume * 5.0) as i32
                })),
            }),
            MenuItem::Separator,
            MenuItem::Text("Exit".into()),
        ],
    );

    Activity::new_base_boxed("audio settings", menu::Menu::new(menu_config))
}
