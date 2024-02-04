use anyhow::Result;
use clap::{Parser, ValueEnum};
use dunce::{canonicalize, simplified};
use walkdir::WalkDir;

use std::{fs, path::Path};

#[derive(ValueEnum, Default, Debug, Clone, Copy, PartialEq, Eq)]
enum UnsupportedFilesAction {
    Ignore,
    Copy,
    #[default]
    Abort,
}

#[derive(Parser, Debug)]
#[clap(version, author, about, name = "compress")]
struct Args {
    #[clap(help = "Assets source directory", default_value = "./assets/raw/")]
    source: String,

    #[clap(
        help = "Assets destination directory",
        default_value = "./assets/dist/"
    )]
    dest: String,

    #[clap(short, long, help = "Action on unsupported files", default_value_t = Default::default())]
    #[arg(value_enum)]
    unsupported_files_action: UnsupportedFilesAction,

    #[clap(short, long, action, help = "Skip empty directories")]
    skip_empty_directories: bool,

    #[clap(short, long, action, help = "Override existing files")]
    force: bool,

    #[clap(short, long, action, help = "Dry run")]
    dry_run: bool,
}

fn main() -> Result<()> {
    let args = Args::try_parse()?;

    #[allow(unused_variables)]
    let Args {
        source,
        dest,
        unsupported_files_action,
        skip_empty_directories,
        force,
        dry_run,
    } = args;

    if dry_run {
        eprintln!(
            "Warning: dry run is enabled, no files will be written to the disk
- Note: directories will be created.
- Remove --dry-run to write files to the disk."
        );
    }

    let source = canonicalize(source)?;

    eprintln!("Checking source dir at {}", source.display());

    if !source.exists() {
        eprintln!("- Source directory does not exist");
        return Ok(());
    }

    if !source.is_dir() {
        eprintln!("- Source is not a directory");
        return Ok(());
    }

    let dest = Path::new(&dest);
    let dest = simplified(dest); // canonicalize expects existing dir

    eprintln!("Creating destination directory at {}", dest.display());

    fs::create_dir_all(&dest)?;

    for entry in WalkDir::new(&source) {
        let entry = entry?;

        let path = entry.path();
        let path = canonicalize(path)?;

        if path.is_dir() && skip_empty_directories {
            continue;
        }

        let entry_relative = path.strip_prefix(&source)?;
        eprintln!("Processing {}", entry_relative.display());

        if path.is_dir() {
            eprintln!("- Creating directory {}", path.display());

            if !dry_run {
                fs::create_dir_all(dest.join(path.strip_prefix(&source)?))?;
            }

            continue;
        }

        if !path.is_file() {
            eprintln!("Skipping non-file {}", path.display());
            continue;
        }

        let dest_path = dest.join(entry_relative);
        let dest_parent = dest_path.parent().ok_or(anyhow::anyhow!("No parent"))?;
        fs::create_dir_all(dest_parent)?;

        let dest_parent = canonicalize(dest_parent)?;
        let dest_path = dest_parent.join(
            entry_relative
                .file_name()
                .ok_or(anyhow::anyhow!("No file name"))?,
        );

        let (ext, action): (_, fn(_, _) -> Result<()>) = match path
            .extension()
            .ok_or(anyhow::anyhow!("No extension"))?
            .to_str()
            .ok_or(anyhow::anyhow!("Invalid extension"))?
        {
            "wav" => ("mp3", |src, dest| {
                let (header, samples) = audio::read_wav(src)?;
                audio::save_mp3(dest, header, samples)
            }),

            // binding syntax just for clarity
            ext @ _ => match unsupported_files_action {
                UnsupportedFilesAction::Ignore => {
                    eprintln!("- Unsupported file, ignoring");
                    continue;
                }
                UnsupportedFilesAction::Copy => {
                    eprintln!("- Unsupported file, copying");
                    (ext, |src, dest| {
                        fs::copy(src, dest)?;
                        Ok(())
                    })
                }
                UnsupportedFilesAction::Abort => {
                    return Err(anyhow::anyhow!("Unsupported file, aborting"));
                }
            },
        };

        let dest_path = dest_path.with_extension(ext);

        eprintln!(
            "Processing file {} to {}",
            path.display(),
            dest_path.display()
        );

        if dest_path.exists() && !force {
            eprintln!("- Destination file exists, skipping");
            continue;
        }

        if dry_run {
            eprintln!("- Dry run, skipping");
            continue;
        }

        action(&path, &dest_path)?;
    }

    Ok(())
}

mod audio {
    use anyhow::{Context, Result};
    use std::{fs::File, io::Write, path::Path};

    pub struct Header {
        channels: u32,
        sample_rate: u32,
    }

    pub(crate) fn read_wav(path: &Path) -> Result<(Header, Vec<i16>)> {
        let mut file =
            File::open(path).with_context(|| format!("Cannot open wav at {}", path.display()))?;

        let (header, samples) = wav::read(&mut file)
            .with_context(|| format!("Cannot read wav at {}", path.display()))?;

        let header = Header {
            channels: header.channel_count as u32,
            sample_rate: header.sampling_rate as u32,
        };

        Ok((
            header,
            samples
                .try_into_sixteen()
                .expect("Only 16-bit samples are supported."),
        ))
    }

    pub(crate) fn save_mp3(path: &Path, header: Header, samples: Vec<i16>) -> Result<()> {
        use mp3lame_encoder::{Builder, FlushNoGap, InterleavedPcm, Quality};

        let path = path.with_extension("mp3");

        let mut encoder =
            Builder::new().with_context(|| anyhow::anyhow!("Failed to allocate encoder"))?;

        encoder
            .set_num_channels(header.channels as u8)
            .map_err(|e| anyhow::anyhow!("Unsupported number of channels: {:?}", e))?;

        encoder
            .set_sample_rate(header.sample_rate as u32)
            .map_err(|e| anyhow::anyhow!("Unsupported sample rate: {:?}", e))?;

        encoder
            .set_quality(Quality::Decent)
            .map_err(|e| anyhow::anyhow!("Unsupported quality: {:?}", e))?;

        let mut encoder = encoder
            .build()
            .map_err(|e| anyhow::anyhow!("Failed to build encoder: {:?}", e))?;

        let data = InterleavedPcm(&samples);

        let mut mp3 = Vec::with_capacity(mp3lame_encoder::max_required_buffer_size(samples.len()));
        let encoded_size = encoder
            .encode(data, mp3.spare_capacity_mut())
            .map_err(|e| anyhow::anyhow!("Failed to encode samples: {:?}", e))?;

        unsafe {
            mp3.set_len(mp3.len().wrapping_add(encoded_size));
        }

        let encoded_size = encoder
            .flush::<FlushNoGap>(mp3.spare_capacity_mut())
            .map_err(|_| anyhow::anyhow!("Failed to flush mp3."))?;

        unsafe {
            mp3.set_len(mp3.len().wrapping_add(encoded_size));
        }

        let mut file = File::create(&path)
            .with_context(|| format!("Cannot create mp3 at {}", path.display()))?;

        file.write_all(&mp3)
            .with_context(|| format!("Cannot write mp3 at {}", path.display()))?;

        Ok(())
    }
}
