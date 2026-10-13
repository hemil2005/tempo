use std::path::Path;

use crate::encode::{f32_to_i16, Prepared, Progress, Result};

pub fn write(prepared: &Prepared, out_path: &Path, progress: &Progress) -> Result<()> {
    let spec = hound::WavSpec {
        channels: prepared.channels,
        sample_rate: prepared.sample_rate,
        bits_per_sample: 16,
        sample_format: hound::SampleFormat::Int,
    };
    let mut writer = hound::WavWriter::create(out_path, spec)
        .map_err(|e| crate::error::ConvertError::Encode(format!("wav writer: {}", e)))?;

    let samples = f32_to_i16(&prepared.samples);
    let total = samples.len();
    for (i, &s) in samples.iter().enumerate() {
        writer
            .write_sample(s)
            .map_err(|e| crate::error::ConvertError::Encode(format!("wav write: {}", e)))?;
        if i % 4096 == 0 {
            progress(0.05 + 0.9 * (i as f32 / total as f32));
        }
    }
    writer
        .finalize()
        .map_err(|e| crate::error::ConvertError::Encode(format!("wav finalize: {}", e)))?;
    Ok(())
}
