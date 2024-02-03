pub mod track;

use rodio::{OutputStream, OutputStreamHandle, Sink};

use self::track::Track;

pub struct SoundPlayer {
    stream: OutputStream,
    handle: OutputStreamHandle,
    sink: Sink,
}

impl SoundPlayer {
    pub fn new() -> Self {
        let (stream, handle) =
            rodio::OutputStream::try_default().expect("Failed to create output stream");

        let sink = Sink::try_new(&handle).expect("Failed to create sink");

        Self {
            stream,
            handle,
            sink,
        }
    }

    pub fn enqueue(&self, track: Track) {
        self.sink.append(track);
        self.sink.play();
    }

    pub fn play_track(&self, track: Track) {
        self.sink.stop();
        self.sink.append(track);
        self.sink.play();
    }

    pub fn wait(&self) {
        self.sink.sleep_until_end();
    }
}
