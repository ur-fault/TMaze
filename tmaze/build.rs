use std::{fs::File, io::Write, path::Path};

use dunce::canonicalize;
use walkdir::WalkDir;

fn main() {
    let out_dir = std::env::var("OUT_DIR").unwrap();
    let out_dir = Path::new(&out_dir).canonicalize().unwrap();

    {
        // audio
        let audio_dir = Path::new("src/sound/assets");
        println!("cargo:rerun-if-changed={}", audio_dir.display());

        let audio_dir = canonicalize(audio_dir).unwrap();
        let audio_out_dir = out_dir.join("audio");
        std::fs::create_dir_all(&audio_out_dir).unwrap();

        enum AudioFormat {
            Flac,
            Mp3,
        }

        let format = AudioFormat::Mp3;

        println!(
            "cargo:rustc-env=AUDIO_EXT={}",
            match format {
                AudioFormat::Flac => "flac",
                AudioFormat::Mp3 => "mp3",
            }
        );

        for item in WalkDir::new(&audio_dir) {
            let item = item.unwrap();

            if item.file_type().is_file() {
                let item_path = item.path();
                let item_path = canonicalize(item_path).unwrap();

                let item_rel = item_path.strip_prefix(&audio_dir).unwrap();

                let item_out_path = audio_out_dir.join(item_rel);

                let item_dir = item_out_path.parent().unwrap();

                std::fs::create_dir_all(&item_dir).unwrap();

                eprintln!("{} -> {}", item_path.display(), item_out_path.display());

                let (header, samples) = read_wav(&item_path);
                match format {
                    AudioFormat::Flac => save_flac(&item_out_path, header, samples),
                    AudioFormat::Mp3 => save_mp3(&item_out_path, header, samples),
                }
            }
        }
    }

    // panic!();
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
    use flacenc::component::BitRepr;

    let path = path.with_extension("flac");

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

fn save_mp3(path: &Path, header: Header, samples: Vec<i16>) {
    let path = path.with_extension("mp3");

    use mp3lame_encoder::{Builder, FlushNoGap, InterleavedPcm, Quality};

    let mut encoder = Builder::new().unwrap();
    encoder
        .set_num_channels(header.channels as u8)
        .expect("Failed to set num channels.");
    encoder.set_sample_rate(header.sample_rate as u32).unwrap();
    encoder.set_quality(Quality::Decent).unwrap();
    let mut encoder = encoder.build().unwrap();

    let data = InterleavedPcm(&samples);

    let mut mp3 = Vec::with_capacity(mp3lame_encoder::max_required_buffer_size(samples.len()));
    let encoded_size = encoder.encode(data, mp3.spare_capacity_mut()).unwrap();

    unsafe {
        mp3.set_len(mp3.len().wrapping_add(encoded_size));
    }

    let encoded_size = encoder
        .flush::<FlushNoGap>(mp3.spare_capacity_mut())
        .unwrap();
    unsafe {
        mp3.set_len(mp3.len().wrapping_add(encoded_size));
    }

    let mut file = File::create(path).expect("Failed to create file.");
    file.write_all(&mp3).unwrap();
}
