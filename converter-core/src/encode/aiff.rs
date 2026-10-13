use std::fs::File;
use std::io::Write;
use std::path::Path;

use crate::encode::{f32_to_i16, Prepared, Progress, Result};
use crate::error::ConvertError;

fn extended80(rate: u32) -> [u8; 10] {
    if rate == 0 {
        return [0; 10];
    }
    let v = rate as f64;
    let exp2 = v.log2().floor() as i32;
    let mantissa = v / 2f64.powi(exp2);
    let frac64 = (mantissa * 9223372036854775808.0).round() as u64;
    let biased = (exp2 + 16383) as u16;
    let mut out = [0u8; 10];
    out[..2].copy_from_slice(&biased.to_be_bytes());
    out[2..].copy_from_slice(&frac64.to_be_bytes());
    out
}

fn write_chunk(file: &mut File, id: &[u8; 4], data: &[u8]) -> std::io::Result<()> {
    file.write_all(id)?;
    file.write_all(&(data.len() as u32).to_be_bytes())?;
    file.write_all(data)?;
    if data.len() % 2 == 1 {
        file.write_all(&[0])?;
    }
    Ok(())
}

pub fn write(prepared: &Prepared, out_path: &Path, progress: &Progress) -> Result<()> {
    if prepared.channels == 0 || prepared.channels > 2 {
        return Err(ConvertError::Encode(
            "aiff supports only mono and stereo".into(),
        ));
    }
    let frames = prepared.samples.len() / prepared.channels as usize;
    let pcm = f32_to_i16(&prepared.samples);
    progress(0.1);

    let mut comm = Vec::with_capacity(18);
    comm.extend_from_slice(&(prepared.channels as i16).to_be_bytes());
    comm.extend_from_slice(&(frames as u32).to_be_bytes());
    comm.extend_from_slice(&16i16.to_be_bytes());
    comm.extend_from_slice(&extended80(prepared.sample_rate));

    let mut ssnd = Vec::with_capacity(8 + pcm.len() * 2);
    ssnd.extend_from_slice(&0u32.to_be_bytes());
    ssnd.extend_from_slice(&0u32.to_be_bytes());
    for &s in &pcm {
        ssnd.extend_from_slice(&s.to_be_bytes());
    }
    progress(0.5);

    let form_size = 4 + (8 + comm.len() as u32) + (8 + (ssnd.len() + (ssnd.len() % 2)) as u32);
    let mut file = File::create(out_path)?;
    file.write_all(b"FORM")?;
    file.write_all(&form_size.to_be_bytes())?;
    file.write_all(b"AIFF")?;
    write_chunk(&mut file, b"COMM", &comm)?;
    write_chunk(&mut file, b"SSND", &ssnd)?;
    file.flush()?;
    progress(0.95);
    Ok(())
}
