pub mod track;

use rodio::{OutputStream, OutputStreamHandle, Sink};

use crate::settings::Settings;

use self::track::Track;

pub struct SoundPlayer {
    _stream: OutputStream,
    handle: OutputStreamHandle,
    sink: Sink,
    settings: Settings,
}

impl SoundPlayer {
    pub fn new(settings: Settings) -> Self {
        let (stream, handle) =
            rodio::OutputStream::try_default().expect("Failed to create output stream");

        let sink = Sink::try_new(&handle).expect("Failed to create sink");

        Self {
            _stream: stream,
            handle,
            sink,
            settings,
        }
    }

    #[allow(dead_code)]
    pub fn enqueue(&self, track: Track) {
        self.sink.append(track);
        self.sink.play();
    }

    pub fn play_track(&self, track: Track) {
        self.sink.stop();
        self.sink.append(track);
        self.sink.play();
    }

    #[allow(dead_code)]
    pub fn play_sound(&self, track: Track) {
        let sink = Sink::try_new(&self.handle).expect("Failed to create sink");
        sink.set_volume(self.settings.get_audio_volume());
        sink.append(track);
        sink.play();
        sink.detach();
    }

    #[allow(dead_code)]
    pub fn wait(&self) {
        self.sink.sleep_until_end();
    }

    pub fn sink(&self) -> &Sink {
        &self.sink
    }

    pub fn set_volume(&self, volume: f32) {
        self.sink.set_volume(volume);
    }
}
