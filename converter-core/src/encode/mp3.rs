use std::fs::File;
use std::io::Write;
use std::path::Path;

use mp3lame_encoder::{Bitrate, Builder, FlushNoGap, InterleavedPcm, Mode, MonoPcm, VbrMode};

use crate::encode::{f32_to_i16, guess_quality_kbps, Prepared, Progress, Result};
use crate::error::ConvertError;
use crate::format::Quality;

fn nearest_bitrate(kbps: u32) -> Bitrate {
    match kbps {
        0..=8 => Bitrate::Kbps8,
        9..=16 => Bitrate::Kbps16,
        17..=24 => Bitrate::Kbps24,
        25..=32 => Bitrate::Kbps32,
        33..=40 => Bitrate::Kbps40,
        41..=48 => Bitrate::Kbps48,
        49..=64 => Bitrate::Kbps64,
        65..=80 => Bitrate::Kbps80,
        81..=96 => Bitrate::Kbps96,
        97..=112 => Bitrate::Kbps112,
        113..=128 => Bitrate::Kbps128,
        129..=160 => Bitrate::Kbps160,
        161..=192 => Bitrate::Kbps192,
        193..=224 => Bitrate::Kbps224,
        225..=256 => Bitrate::Kbps256,
        _ => Bitrate::Kbps320,
    }
}

pub fn write(
    prepared: &Prepared,
    quality: &Quality,
    out_path: &Path,
    progress: &Progress,
) -> Result<()> {
    if prepared.channels != 1 && prepared.channels != 2 {
        return Err(ConvertError::Encode(
            "mp3 supports only mono and stereo".into(),
        ));
    }

    let mut builder = Builder::new().ok_or_else(|| ConvertError::Encode("lame init".into()))?;
    builder
        .set_num_channels(prepared.channels as u8)
        .map_err(|e| ConvertError::Encode(e.to_string()))?;
    builder
        .set_sample_rate(prepared.sample_rate)
        .map_err(|e| ConvertError::Encode(e.to_string()))?;
    match quality {
        Quality::Cbr { kbps } => {
            builder
                .set_brate(nearest_bitrate(*kbps))
                .map_err(|e| ConvertError::Encode(e.to_string()))?;
            builder
                .set_mode(if prepared.channels == 1 {
                    Mode::Mono
                } else {
                    Mode::JointStereo
                })
                .map_err(|e| ConvertError::Encode(e.to_string()))?;
        }
        Quality::Vbr { level } => {
            builder
                .set_vbr_mode(VbrMode::Mtrh)
                .map_err(|e| ConvertError::Encode(e.to_string()))?;
            let q: mp3lame_encoder::Quality = match (*level).clamp(0, 9) {
                0 => mp3lame_encoder::Quality::Best,
                1 => mp3lame_encoder::Quality::SecondBest,
                2 => mp3lame_encoder::Quality::NearBest,
                3 => mp3lame_encoder::Quality::VeryNice,
                4 => mp3lame_encoder::Quality::Nice,
                5 => mp3lame_encoder::Quality::Good,
                6 => mp3lame_encoder::Quality::Decent,
                7 => mp3lame_encoder::Quality::Ok,
                8 => mp3lame_encoder::Quality::SecondWorst,
                _ => mp3lame_encoder::Quality::Worst,
            };
            builder
                .set_vbr_quality(q)
                .map_err(|e| ConvertError::Encode(e.to_string()))?;
        }
        Quality::Scale { .. } | Quality::Lossless => {
            let kbps = guess_quality_kbps(*quality, 192);
            builder
                .set_brate(nearest_bitrate(kbps))
                .map_err(|e| ConvertError::Encode(e.to_string()))?;
        }
    }

    let mut encoder = builder
        .build()
        .map_err(|e| ConvertError::Encode(e.to_string()))?;

    let pcm = f32_to_i16(&prepared.samples);
    let mut out = Vec::new();

    let frames = prepared.samples.len() / prepared.channels as usize;
    let chunk_frames = 1152usize * 8;
    let mut offset = 0usize;
    while offset < frames {
        let end = (offset + chunk_frames).min(frames);
        out.reserve(mp3lame_encoder::max_required_buffer_size(
            (end - offset) * prepared.channels as usize,
        ));
        if prepared.channels == 1 {
            let seg = MonoPcm(&pcm[offset..end]);
            encoder
                .encode_to_vec(seg, &mut out)
                .map_err(|e| ConvertError::Encode(e.to_string()))?;
        } else {
            let seg = InterleavedPcm(&pcm[offset * 2..end * 2]);
            encoder
                .encode_to_vec(seg, &mut out)
                .map_err(|e| ConvertError::Encode(e.to_string()))?;
        }
        offset = end;
        progress(0.05 + 0.9 * (offset as f32 / frames as f32));
    }
    out.reserve(8192);
    encoder
        .flush_to_vec::<FlushNoGap>(&mut out)
        .map_err(|e| ConvertError::Encode(e.to_string()))?;

    let mut f = File::create(out_path)?;
    f.write_all(&out)?;
    f.flush()?;
    Ok(())
}
