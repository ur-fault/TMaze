use std::{
    fs::File,
    io::Write,
    path::{Component, Path, PathBuf},
};

use dunce::canonicalize;
use flacenc::component::BitRepr;
use walkdir::WalkDir;

fn main() {
    let out_dir = std::env::var("OUT_DIR").unwrap();
    let out_dir = Path::new(&out_dir).canonicalize().unwrap();

    // audio
    let audio_dir = Path::new("src/sound/assets");
    println!("cargo:rerun-if-changed={}", audio_dir.display());

    let audio_dir = canonicalize(audio_dir).unwrap();
    let audio_out_dir = out_dir.join("audio");
    std::fs::create_dir_all(&audio_out_dir).unwrap();
    for item in WalkDir::new(&audio_dir) {
        let item = item.unwrap();

        if item.file_type().is_file() {
            let item_path = item.path();
            let item_path = canonicalize(item_path).unwrap();

            let item_rel = item_path.strip_prefix(&audio_dir).unwrap();

            let item_out_path = audio_out_dir.join(item_rel);
            let item_out_path = item_out_path.with_extension("flac");

            let item_dir = item_out_path.parent().unwrap();

            std::fs::create_dir_all(&item_dir).unwrap();

            let (header, samples) = read_wav(&item_path);
            save_flac(&item_out_path, header, samples);
        }
    }

    panic!();
}

struct Header {
    channels: usize,
    sample_rate: usize,
}

fn read_wav(path: &Path) -> (Header, Vec<i16>) {
    let mut file = File::open(path).expect("File {path} not found.");
    let (header, samples) = wav::read(&mut file).expect("Failed to read WAV file.");

    let header = Header {
        channels: header.channel_count as usize,
        sample_rate: header.sampling_rate as usize,
    };

    (
        header,
        samples
            .try_into_sixteen()
            .expect("Only 16-bit samples are supported."),
    )
}

fn save_flac(path: &Path, header: Header, samples: Vec<i16>) {
    let samples = samples.into_iter().map(|s| s as i32).collect::<Vec<_>>();

    let config = flacenc::config::Encoder::default();
    let source =
        flacenc::source::MemSource::from_samples(&samples, header.channels, 16, header.sample_rate);
    let flac_stream = flacenc::encode_with_fixed_block_size(&config, source, config.block_sizes[0])
        .expect("Encode failed.");

    let mut sink = flacenc::bitsink::ByteSink::new();
    flac_stream
        .write(&mut sink)
        .expect("Write of flac stream to bitsink failed.");

    let mut file = File::create(path).expect("Failed to create file.");
    file.write_all(sink.as_slice()).unwrap();
}
