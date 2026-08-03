pub mod track;

use rodio::{OutputStream, OutputStreamHandle, Sink};

use crate::{
    app::{
        activity::ActivityHandlerExt, app::AppData, event::EventReceiver, Activity, Change,
        GlobalEvent,
    },
    settings::Settings,
    ui::{Slider, Switch, Text},
    update_settings,
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
        sink.set_volume(self.settings.read().audio.global.volume as f32);
        sink.append(track);
        sink.play();
        sink.detach();
    }

    pub fn set_volume(&self, volume: f32) {
        self.apply(|sink| sink.set_volume(volume));
    }
}

impl EventReceiver for &SoundPlayer {
    fn register(self) -> crate::app::event::EventReceiverFn {
        Box::new(move |event, data| {
            if let GlobalEvent::SettingsChanged = event {
                let cfg = &data.settings.read().audio;

                let preamp = (cfg.global.enable && cfg.music.enable) as i32 as f64;

                data.sound_player
                    .set_volume((preamp * cfg.global.volume * cfg.music.volume) as f32);
            }
        })
    }
}

pub fn create_audio_settings(data: &mut AppData) -> Activity {
    use crate::ui::menu::{Menu, MenuConfig, MenuItemObj};

    let config = &data.settings.read().audio;

    Menu::new(MenuConfig::new(
        "Audio settings",
        vec![
            Switch::new("Enable global sound", config.global.enable, |m, d| {
                update_settings!(d.settings, audio.global.enable = m)
            }) as MenuItemObj,
            Slider::new("Global volume", config.global.volume, 0.0..=1.0, |v, d| {
                update_settings!(d.settings, audio.global.volume = v)
            })
            .step(0.1)
            .width(10),
            Switch::new("Enable Music", config.music.enable, |m, d| {
                update_settings!(d.settings, audio.music.enable = m)
            }),
            Slider::new("Music volume", config.music.volume, 0.0..=1.0, |v, d| {
                update_settings!(d.settings, audio.music.volume = v)
            })
            .step(0.1)
            .width(10),
            Text::new("Back").click(|_| Some(Change::pop_top())),
        ],
    ))
    .to_base_activity("audio settings")
}
