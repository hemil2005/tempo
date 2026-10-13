pub mod aac;
pub mod aiff;
pub mod flac;
pub mod mp3;
pub mod opus;
pub mod vorbis;
pub mod wav;

use std::path::Path;

use audioadapter_buffers::owned::InterleavedOwned;
use rubato::{Fft, FixedSync, Resampler};

use crate::decode::Decoded;
use crate::error::{ConvertError, Result};
use crate::format::{EncodeSettings, OutputFormat, Quality};

pub struct Prepared {
    pub samples: Vec<f32>,
    pub channels: u16,
    pub sample_rate: u32,
}

pub type Progress = dyn Fn(f32) + Send + Sync;

pub fn convert(
    decoded: &Decoded,
    settings: &EncodeSettings,
    out_path: &Path,
    progress: &Progress,
) -> Result<()> {
    if out_path
        .extension()
        .and_then(|e| e.to_str())
        .map(|e| e.to_ascii_lowercase())
        .as_deref()
        != Some(settings.format.extension())
    {
        return Err(ConvertError::Encode(format!(
            "output path extension must be .{}",
            settings.format.extension()
        )));
    }

    let prepared = prepare(decoded, settings)?;
    progress(0.05);

    match settings.format {
        OutputFormat::Mp3 => mp3::write(&prepared, &settings.quality, out_path, progress)?,
        OutputFormat::Opus => opus::write(
            &prepared,
            &settings.quality,
            &decoded.tags,
            out_path,
            progress,
        )?,
        OutputFormat::OggVorbis => vorbis::write(
            &prepared,
            &settings.quality,
            &decoded.tags,
            out_path,
            progress,
        )?,
        OutputFormat::Wav => wav::write(&prepared, out_path, progress)?,
        OutputFormat::Flac => flac::write(&prepared, out_path, progress)?,
        OutputFormat::M4a => aac::write(&prepared, &settings.quality, out_path, progress)?,
        OutputFormat::Aiff => aiff::write(&prepared, out_path, progress)?,
    }

    decoded.tags.write_to_file(out_path, settings.format)?;

    progress(1.0);
    Ok(())
}

fn prepare(decoded: &Decoded, settings: &EncodeSettings) -> Result<Prepared> {
    let target_rate = if settings.sample_rate == 0 {
        decoded.sample_rate
    } else {
        settings.sample_rate
    };
    let target_channels = if settings.channels == 0 {
        decoded.channels as u16
    } else {
        settings.channels
    };

    let mut samples =
        convert_channels(&decoded.samples, decoded.channels, target_channels as usize);
    let mut rate = decoded.sample_rate;

    if target_rate != rate {
        samples = resample(&samples, target_channels as usize, rate, target_rate)?;
        rate = target_rate;
    }

    Ok(Prepared {
        samples,
        channels: target_channels,
        sample_rate: rate,
    })
}

fn convert_channels(input: &[f32], from: usize, to: usize) -> Vec<f32> {
    if from == to {
        return input.to_vec();
    }
    let frames = input.len() / from;
    let mut out = Vec::with_capacity(frames * to);
    match (from, to) {
        (1, 2) => {
            for s in input.iter().take(frames) {
                out.push(*s);
                out.push(*s);
            }
        }
        (2, 1) => {
            for f in 0..frames {
                out.push(0.5 * (input[f * 2] + input[f * 2 + 1]));
            }
        }
        _ => {
            for f in 0..frames {
                for c in 0..to {
                    out.push(input[f * from + (c % from)]);
                }
            }
        }
    }
    out
}

pub(crate) fn resample(
    input: &[f32],
    channels: usize,
    rate_in: u32,
    rate_out: u32,
) -> Result<Vec<f32>> {
    let frames = input.len() / channels;
    let mut resampler = Fft::<f32>::new(
        rate_in as usize,
        rate_out as usize,
        1024,
        channels,
        FixedSync::Both,
    )
    .map_err(|e| ConvertError::Resample(e.to_string()))?;

    let input_buf = InterleavedOwned::<f32>::new_from(input.to_vec(), channels, frames)
        .map_err(|e| ConvertError::Resample(e.to_string()))?;
    let needed = resampler.process_all_needed_output_len(frames);
    let mut output_buf = InterleavedOwned::<f32>::new(0.0, channels, needed);

    let (_, produced) = resampler
        .process_all_into_buffer(&input_buf, &mut output_buf, frames, None)
        .map_err(|e| ConvertError::Resample(e.to_string()))?;

    let data = output_buf.take_data();
    Ok(data.into_iter().take(produced * channels).collect())
}

pub fn f32_to_i16(samples: &[f32]) -> Vec<i16> {
    samples
        .iter()
        .map(|s| (s.clamp(-1.0, 1.0) * 32767.0).round() as i16)
        .collect()
}

pub(crate) fn guess_quality_kbps(quality: Quality, def: u32) -> u32 {
    match quality {
        Quality::Cbr { kbps } => kbps,
        Quality::Vbr { level } => match level.clamp(0, 9) {
            0 => 245,
            1 => 225,
            2 => 190,
            3 => 175,
            4 => 165,
            5 => 130,
            6 => 115,
            7 => 100,
            8 => 85,
            _ => 65,
        },
        Quality::Scale { value } => ((96.0 + value.clamp(0.0, 1.0) * 224.0) as u32).max(64),
        Quality::Lossless => def,
    }
}
