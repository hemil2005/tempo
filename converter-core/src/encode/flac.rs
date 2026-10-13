use std::fs::File;
use std::io::Write;
use std::path::Path;

use flacenc::bitsink::MemSink;
use flacenc::component::BitRepr;
use flacenc::error::Verify;

use crate::encode::{Prepared, Progress, Result};
use crate::error::ConvertError;

pub fn write(prepared: &Prepared, out_path: &Path, progress: &Progress) -> Result<()> {
    let channels = prepared.channels as usize;
    if channels > 8 || channels == 0 {
        return Err(ConvertError::Encode("flac supports 1-8 channels".into()));
    }
    let frames = prepared.samples.len() / channels;
    progress(0.05);

    let i32_samples: Vec<i32> = prepared
        .samples
        .iter()
        .map(|s| (s.clamp(-1.0, 1.0) * 8_388_607.0).round() as i32)
        .collect();
    progress(0.2);

    let config = flacenc::config::Encoder::default()
        .into_verified()
        .map_err(|(_, e)| ConvertError::Encode(format!("{:?}", e)))?;
    let block_size = config.block_size;
    let source = flacenc::source::MemSource::from_samples(
        &i32_samples,
        channels,
        24,
        prepared.sample_rate as usize,
    );
    let mut stream = flacenc::encode_with_fixed_block_size(&config, source, block_size)
        .map_err(|e| ConvertError::Encode(e.to_string()))?;
    progress(0.85);

    stream.stream_info_mut().set_total_samples(frames);
    if frames > block_size {
        stream
            .stream_info_mut()
            .set_block_sizes(block_size, block_size)
            .map_err(|e| ConvertError::Encode(format!("{:?}", e)))?;
    }

    let mut sink = MemSink::<u8>::new();
    stream
        .write(&mut sink)
        .map_err(|e| ConvertError::Encode(e.to_string()))?;
    progress(0.92);

    let mut f = File::create(out_path)?;
    f.write_all(sink.as_slice())?;
    f.flush()?;
    Ok(())
}
