use std::path::PathBuf;

use clap::{Parser, ValueEnum};

use converter_core::{convert, decode_file, AudioFormat, EncodeSettings, OutputFormat, Quality};

#[derive(Debug, Clone, Copy, ValueEnum)]
enum OutFmt {
    Mp3,
    Opus,
    Ogg,
    Wav,
    Flac,
    M4a,
    Aiff,
}

impl From<OutFmt> for OutputFormat {
    fn from(value: OutFmt) -> Self {
        match value {
            OutFmt::Mp3 => OutputFormat::Mp3,
            OutFmt::Opus => OutputFormat::Opus,
            OutFmt::Ogg => OutputFormat::OggVorbis,
            OutFmt::Wav => OutputFormat::Wav,
            OutFmt::Flac => OutputFormat::Flac,
            OutFmt::M4a => OutputFormat::M4a,
            OutFmt::Aiff => OutputFormat::Aiff,
        }
    }
}

#[derive(Parser)]
#[command(name = "converter", about = "Audio format converter")]
struct Args {
    input: PathBuf,
    output: PathBuf,
    #[arg(short, long, value_enum, default_value = "mp3")]
    format: OutFmt,
    #[arg(short, long, default_value_t = 0)]
    bitrate: u32,
    #[arg(long, default_value_t = 0)]
    sample_rate: u32,
    #[arg(long, default_value_t = 0)]
    channels: u16,
}

fn main() {
    env_logger::init();
    let args = Args::parse();
    let in_fmt = AudioFormat::from_path(&args.input);
    if !in_fmt.is_decodable() {
        eprintln!("unsupported input format");
        std::process::exit(2);
    }

    let decoded = decode_file(&args.input).expect("decode failed");
    println!(
        "decoded: {} frames, {} ch, {} Hz",
        decoded.frames(),
        decoded.channels,
        decoded.sample_rate
    );

    let format: OutputFormat = args.format.into();
    let mut settings = EncodeSettings::new(format);
    if args.bitrate != 0 {
        settings.quality = Quality::Cbr { kbps: args.bitrate };
    }
    if args.sample_rate != 0 {
        settings.sample_rate = args.sample_rate;
    }
    if args.channels != 0 {
        settings.channels = args.channels;
    }

    convert(&decoded, &settings, &args.output, &|p| {
        if ((p * 100.0) as u32).is_multiple_of(10) {
            eprintln!("progress: {:.0}%", p * 100.0);
        }
    })
    .expect("convert failed");
    println!("wrote {}", args.output.display());
}
