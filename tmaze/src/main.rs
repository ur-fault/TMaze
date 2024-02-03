mod constants;
mod data;
mod game;
mod helpers;
mod renderer;
mod settings;
#[cfg(feature = "sound")]
mod sound;
mod ui;
#[cfg(feature = "updates")]
mod updates;

use clap::Parser;
use cmaze::{core, gameboard};
use game::{App, GameError};

#[derive(Parser, Debug)]
#[clap(version, author, about, name = "tmaze")]
struct Args {
    #[clap(short, long, action, help = "Reset config to default and quit")]
    reset_config: bool,
    #[clap(short, long, action, help = "Show config path and quit")]
    show_config_path: bool,
    #[clap(long, help = "Show config in debug format and quit")]
    debug_config: bool,

    // TODO: remove this
    #[clap(long, help = "Just run sounds (for testing)")]
    sounds: bool,
}

fn main() -> Result<(), GameError> {
    let _args = Args::parse();

    if _args.reset_config {
        settings::Settings::reset_config(settings::Settings::default_path());
        return Ok(());
    }

    if _args.show_config_path {
        if let Some(s) = settings::Settings::default_path().to_str() {
            println!("{}", s);
        } else {
            println!("{:?}", settings::Settings::default_path());
        }
        return Ok(());
    }

    if _args.debug_config {
        println!(
            "{:#?}",
            settings::Settings::load(settings::Settings::default_path())
        );
        return Ok(());
    }

    if _args.sounds {
        use rodio::Source;

        let player = sound::SoundPlayer::new();
        let track = sound::track::MusicTracks::Menu.get_track();
        player.enqueue(track);
        std::thread::sleep(std::time::Duration::from_secs(2));
        let track = sound::track::MusicTracks::Easy.get_track();

        let track = Box::new(
            track
                .take_duration(std::time::Duration::from_secs(4))
                .repeat_infinite(),
        );

        player.play_track(track);
        player.wait();
        return Ok(());
    }

    App::new().run()
}
