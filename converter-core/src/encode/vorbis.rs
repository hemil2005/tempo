use std::fs::File;
use std::num::{NonZeroU32, NonZeroU8};
use std::path::Path;

use vorbis_rs::{VorbisBitrateManagementStrategy, VorbisEncoderBuilder};

use crate::encode::{Prepared, Progress, Result};
use crate::error::ConvertError;
use crate::format::Quality;

pub fn write(
    prepared: &Prepared,
    quality: &Quality,
    tags: &crate::tags::Tags,
    out_path: &Path,
    progress: &Progress,
) -> Result<()> {
    if prepared.channels == 0 || prepared.channels > 8 {
        return Err(ConvertError::Encode("vorbis supports 1-8 channels".into()));
    }
    let channels = prepared.channels as usize;
    let chan_u8 = NonZeroU8::new(channels as u8)
        .ok_or_else(|| ConvertError::Encode("bad channels".into()))?;
    let freq = NonZeroU32::new(prepared.sample_rate)
        .ok_or_else(|| ConvertError::Encode("bad sample rate".into()))?;

    let strategy = match quality {
        Quality::Scale { value } => VorbisBitrateManagementStrategy::QualityVbr {
            target_quality: (-0.2 + value.clamp(0.0, 1.0) * 1.2).clamp(-0.2, 1.0),
        },
        Quality::Vbr { level } => VorbisBitrateManagementStrategy::QualityVbr {
            target_quality: (*level as f32 / 9.0 * 1.2 - 0.2).clamp(-0.2, 1.0),
        },
        Quality::Cbr { kbps } => VorbisBitrateManagementStrategy::Vbr {
            target_bitrate: NonZeroU32::new(kbps.max(&8) * 1000)
                .ok_or_else(|| ConvertError::Encode("bad bitrate".into()))?,
        },
        Quality::Lossless => VorbisBitrateManagementStrategy::Vbr {
            target_bitrate: NonZeroU32::new(160 * 1000)
                .ok_or_else(|| ConvertError::Encode("bad bitrate".into()))?,
        },
    };

    let file = File::create(out_path)?;
    let mut builder = VorbisEncoderBuilder::new(freq, chan_u8, file)
        .map_err(|e| ConvertError::Encode(e.to_string()))?;
    builder.bitrate_management_strategy(strategy);
    for (k, v) in tags.vorbis_comments() {
        builder
            .comment_tag(k, v)
            .map_err(|e| ConvertError::Encode(e.to_string()))?;
    }
    let mut encoder = builder
        .build()
        .map_err(|e| ConvertError::Encode(e.to_string()))?;

    let frames = prepared.samples.len() / channels;
    let mut planar: Vec<Vec<f32>> = (0..channels)
        .map(|c| {
            prepared
                .samples
                .iter()
                .skip(c)
                .step_by(channels)
                .copied()
                .collect()
        })
        .collect();

    let block = 4096usize;
    let mut offset = 0usize;
    while offset < frames {
        let end = (offset + block).min(frames);
        let view: Vec<&[f32]> = planar.iter().map(|ch| &ch[offset..end]).collect();
        encoder
            .encode_audio_block(&view)
            .map_err(|e| ConvertError::Encode(e.to_string()))?;
        offset = end;
        progress(0.05 + 0.9 * (offset as f32 / frames as f32));
    }
    let _ = &mut planar;
    encoder
        .finish()
        .map_err(|e| ConvertError::Encode(e.to_string()))?;
    Ok(())
}
