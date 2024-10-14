#[cfg(feature = "sound")]
use tmaze::{
    app::app::{App, AppData},
    sound::track,
    ui::{menu, MenuItem, OptionDef, SliderDef},
};

#[cfg(feature = "sound")]
fn main() {
    let mut app = App::empty(true);

    let menu_config = menu::MenuConfig::new(
        "Audio settings",
        [
            MenuItem::Option(OptionDef {
                text: "Global mute".into(),
                val: !app.data().settings.get_enable_audio(),
                fun: Box::new(|mute, data| {
                    *mute = !*mute;
                    data.settings.set_enable_audio(!*mute);
                    update_vol(data);
                }),
            }),
            MenuItem::Slider(SliderDef {
                text: "Global volume".into(),
                val: (app.data().settings.get_audio_volume() * 5.0) as i32,
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
                val: !app.data().settings.get_enable_music(),
                fun: Box::new(|mute, data| {
                    *mute = !*mute;
                    data.settings.set_enable_music(!*mute);
                    update_vol(data);
                }),
            }),
            MenuItem::Slider(SliderDef {
                text: "Music volume".into(),
                val: (app.data().settings.get_music_volume() * 5.0) as i32,
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

    let menu = menu::Menu::new(menu_config).into_activity();
    app.activities_mut().push(menu);

    app.data_mut().play_bgm(track::MusicTrack::Menu);

    app.run();
}

#[cfg(feature = "sound")]
fn update_vol(data: &mut AppData) {
    if data.settings.get_enable_audio() && data.settings.get_enable_music() {
        data.sound_player
            .set_volume(data.settings.get_audio_volume() * data.settings.get_music_volume());
    } else {
        data.sound_player.set_volume(0.0);
    }
}

#[cfg(not(feature = "sound"))]
fn main() {
    panic!("Cannot run `sound` example without the `sound` feature");
}
