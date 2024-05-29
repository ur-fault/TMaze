use tmaze::{
    app::app::App,
    logging,
    sound::track,
    ui::{menu, MenuItem, OptionDef, SliderDef},
};

fn main() {
    logging::get_logger().switch_debug();

    let mut app = App::empty();

    let menu_config = menu::MenuConfig::new(
        "Menu",
        vec![
            MenuItem::Option(OptionDef {
                text: "Mute".into(),
                val: false,
                fun: Box::new(|mute, data| {
                    *mute = !*mute;
                    data.settings.set_enable_audio(!*mute);
                    let settings = &data.settings;
                    if settings.get_enable_audio() && settings.get_enable_music() {
                        data.sound_player
                            .sink()
                            .set_volume(settings.get_audio_volume() * settings.get_music_volume());
                    } else {
                        data.sound_player.sink().set_volume(0.0);
                    }
                }),
            }),
            MenuItem::Slider(SliderDef {
                text: "Volume".into(),
                val: 5,
                range: 0..=10,
                as_num: false,
                fun: Box::new(|up, vol, data| {
                    *vol += if up { 1 } else { -1 };
                    data.settings.set_audio_volume(*vol as f32 / 10.0);
                    let settings = &data.settings;
                    if settings.get_enable_audio() && settings.get_enable_music() {
                        data.sound_player
                            .sink()
                            .set_volume(settings.get_audio_volume() * settings.get_music_volume());
                    }
                }),
            }),
            MenuItem::Separator,
            MenuItem::Text("Quit".into()),
        ],
    );

    let menu = menu::Menu::new(menu_config).into_activity();
    app.activities_mut().push(menu);

    app.data_mut().play_bgm(track::MusicTrack::Menu);

    app.run();
}
