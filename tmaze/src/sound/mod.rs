pub mod track;

use menu::OptionDef;
use rodio::{OutputStream, OutputStreamHandle, Sink};

use crate::{
    app::{app::AppData, Activity},
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
        sink.set_volume(self.settings.get_audio_volume());
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

pub fn create_audio_settings(data: &mut AppData) -> Activity {
    fn update_vol(data: &mut AppData) {
        if data.settings.get_enable_audio() && data.settings.get_enable_music() {
            data.sound_player
                .set_volume(data.settings.get_audio_volume() * data.settings.get_music_volume());
        } else {
            data.sound_player.set_volume(0.0);
        }
    }

    let menu_config = menu::MenuConfig::new(
        "Audio settings",
        [
            MenuItem::Option(OptionDef {
                text: "Global mute".into(),
                val: !data.settings.get_enable_audio(),
                fun: Box::new(|mute, data| {
                    *mute = !*mute;
                    data.settings.set_enable_audio(!*mute);
                    update_vol(data);
                }),
            }),
            MenuItem::Slider(SliderDef {
                text: "Global volume".into(),
                val: (data.settings.get_audio_volume() * 5.0) as i32,
                range: 0..=5,
                as_num: false,
                fun: Box::new(|up, vol, data| {
                    *vol += if up { 1 } else { -1 };
                    data.settings.set_audio_volume(*vol as f32 / 5.0);
                    update_vol(data);
                }),
            }),
            MenuItem::Option(OptionDef {
                text: "Music mute".into(),
                val: !data.settings.get_enable_music(),
                fun: Box::new(|mute, data| {
                    *mute = !*mute;
                    data.settings.set_enable_music(!*mute);
                    update_vol(data);
                }),
            }),
            MenuItem::Slider(SliderDef {
                text: "Music volume".into(),
                val: (data.settings.get_music_volume() * 5.0) as i32,
                range: 0..=5,
                as_num: false,
                fun: Box::new(|up, vol, data| {
                    *vol += if up { 1 } else { -1 };
                    data.settings.set_music_volume(*vol as f32 / 5.0);
                    update_vol(data);
                }),
            }),
            MenuItem::Separator,
            MenuItem::Text("Exit".into()),
        ],
    );

    Activity::new_base_boxed("audio settings", menu::Menu::new(menu_config))
}
