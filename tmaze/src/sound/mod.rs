use rodio::{Source, Device, Sink, OutputStream, OutputStreamHandle};

struct SoundPlayer {
    stream: OutputStream,
    handle: OutputStreamHandle,
}


