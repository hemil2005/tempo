use std::fs::File;
use std::path::Path;

use bytes::Bytes;
use fdk_aac::enc::{
    AudioObjectType as FdkAot, BitRate as FdkBitRate, ChannelMode as FdkChannels, Encoder,
    EncoderParams, Transport,
};
use mp4::{
    AacConfig, AudioObjectType as Mp4Aot, ChannelConfig as Mp4Chan, MediaConfig, Mp4Config,
    Mp4Sample, Mp4Writer, SampleFreqIndex, TrackConfig, TrackType,
};

use crate::encode::{f32_to_i16, guess_quality_kbps, resample, Prepared, Progress, Result};
use crate::error::ConvertError;
use crate::format::Quality;

fn read_desc_len(buf: &[u8], pos: &mut usize) -> Option<(u8, u32, usize)> {
    if *pos >= buf.len() {
        return None;
    }
    let tag = buf[*pos];
    *pos += 1;
    let len_start = *pos;
    let mut size: u32 = 0;
    for _ in 0..4 {
        if *pos >= buf.len() {
            return None;
        }
        let b = buf[*pos];
        *pos += 1;
        size = (size << 7) | u32::from(b & 0x7f);
        if b & 0x80 == 0 {
            break;
        }
    }
    Some((tag, size, len_start))
}

fn scan_descriptors(data: &mut Vec<u8>, mut pos: usize, end: usize) {
    while pos < end {
        let (tag, len, len_start) = match read_desc_len(data, &mut pos) {
            Some(t) => t,
            None => break,
        };
        let content_end = (pos + len as usize).min(end);
        if tag == 0x06 {
            if len as usize != end - pos && end - pos < 128 {
                data[len_start] = (end - pos) as u8;
            }
            return;
        }
        if tag == 0x03 && pos + 3 <= content_end {
            scan_descriptors(data, pos + 3, content_end);
            pos = content_end;
            continue;
        }
        if tag == 0x04 && pos + 13 <= content_end {
            scan_descriptors(data, pos + 13, content_end);
            pos = content_end;
            continue;
        }
        pos = content_end;
    }
}

fn fix_esds_sl_len(data: &mut Vec<u8>) {
    let mut i = 0usize;
    while i + 8 <= data.len() {
        if &data[i + 4..i + 8] == b"esds" {
            let size =
                u32::from_be_bytes([data[i], data[i + 1], data[i + 2], data[i + 3]]) as usize;
            if size >= 8 && i + size <= data.len() {
                let end = i + size;
                scan_descriptors(data, i + 8 + 4, end);
            }
            i += size.max(8);
        } else {
            i += 1;
        }
    }
}

fn freq_index(rate: u32) -> Option<SampleFreqIndex> {
    Some(match rate {
        96000 => SampleFreqIndex::Freq96000,
        88200 => SampleFreqIndex::Freq88200,
        64000 => SampleFreqIndex::Freq64000,
        48000 => SampleFreqIndex::Freq48000,
        44100 => SampleFreqIndex::Freq44100,
        32000 => SampleFreqIndex::Freq32000,
        24000 => SampleFreqIndex::Freq24000,
        22050 => SampleFreqIndex::Freq22050,
        16000 => SampleFreqIndex::Freq16000,
        12000 => SampleFreqIndex::Freq12000,
        11025 => SampleFreqIndex::Freq11025,
        8000 => SampleFreqIndex::Freq8000,
        7350 => SampleFreqIndex::Freq7350,
        _ => return None,
    })
}

pub fn write(
    prepared: &Prepared,
    quality: &Quality,
    out_path: &Path,
    progress: &Progress,
) -> Result<()> {
    let (channels_mode, chan_conf) = match prepared.channels {
        1 => (FdkChannels::Mono, Mp4Chan::Mono),
        2 => (FdkChannels::Stereo, Mp4Chan::Stereo),
        _ => {
            return Err(ConvertError::Encode(
                "aac supports only mono and stereo".into(),
            ))
        }
    };

    let (rate, pcm_f32): (u32, Vec<f32>) = if freq_index(prepared.sample_rate).is_some() {
        (prepared.sample_rate, prepared.samples.clone())
    } else {
        let target = if prepared.sample_rate >= 44000 {
            48000
        } else {
            44100
        };
        let rs = resample(
            &prepared.samples,
            prepared.channels as usize,
            prepared.sample_rate,
            target,
        )?;
        (target, rs)
    };
    let pcm = f32_to_i16(&pcm_f32);

    let kbps = guess_quality_kbps(*quality, 192);
    let encoder = Encoder::new(EncoderParams {
        bit_rate: FdkBitRate::Cbr(kbps * 1000),
        sample_rate: rate,
        transport: Transport::Raw,
        channels: channels_mode,
        audio_object_type: FdkAot::Mpeg4LowComplexity,
    })
    .map_err(|e| ConvertError::Encode(format!("{:?}", e)))?;

    let mut packets: Vec<Vec<u8>> = Vec::new();
    let mut out_buf = vec![0u8; 8192];
    let chan = prepared.channels as usize;
    let frame_len = 1024 * chan;

    let mut padded = pcm.clone();
    while !padded.len().is_multiple_of(frame_len) {
        padded.push(0);
    }

    let mut pos = 0usize;
    while pos < padded.len() {
        let info = encoder
            .encode(&padded[pos..], &mut out_buf)
            .map_err(|e| ConvertError::Encode(format!("{:?}", e)))?;
        if info.input_consumed == 0 && info.output_size == 0 {
            break;
        }
        pos += info.input_consumed;
        if info.output_size > 0 {
            packets.push(out_buf[..info.output_size].to_vec());
        }
        progress(0.05 + 0.5 * (pos.min(padded.len()) as f32 / padded.len() as f32));
    }

    let zeros = vec![0i16; frame_len];
    let input_chunks = padded.len() / chan / 1024;
    let expected_out = input_chunks + 1;
    for _ in 0..16 {
        if packets.len() >= expected_out {
            break;
        }
        let info = encoder
            .encode(&zeros, &mut out_buf)
            .map_err(|e| ConvertError::Encode(format!("{:?}", e)))?;
        if info.output_size == 0 {
            break;
        }
        packets.push(out_buf[..info.output_size].to_vec());
    }
    let _ = chan;

    let file = File::create(out_path)?;
    let config = Mp4Config {
        major_brand: "M4A "
            .parse()
            .map_err(|e| ConvertError::Encode(format!("{:?}", e)))?,
        minor_version: 0,
        compatible_brands: vec![
            "isom"
                .parse()
                .map_err(|e| ConvertError::Encode(format!("{:?}", e)))?,
            "M4A "
                .parse()
                .map_err(|e| ConvertError::Encode(format!("{:?}", e)))?,
        ],
        timescale: rate,
    };
    let mut writer =
        Mp4Writer::write_start(file, &config).map_err(|e| ConvertError::Encode(e.to_string()))?;
    let track = TrackConfig {
        track_type: TrackType::Audio,
        timescale: rate,
        language: "und".to_string(),
        media_conf: MediaConfig::AacConfig(AacConfig {
            bitrate: kbps * 1000,
            profile: Mp4Aot::AacLowComplexity,
            freq_index: freq_index(rate).ok_or_else(|| ConvertError::Encode("bad rate".into()))?,
            chan_conf,
        }),
    };
    writer
        .add_track(&track)
        .map_err(|e| ConvertError::Encode(e.to_string()))?;

    let mut start_time: u64 = 0;
    let total = packets.len();
    for (i, packet) in packets.iter().enumerate() {
        let sample = Mp4Sample {
            start_time,
            duration: 1024,
            rendering_offset: 0,
            is_sync: true,
            bytes: Bytes::copy_from_slice(packet),
        };
        writer
            .write_sample(1, &sample)
            .map_err(|e| ConvertError::Encode(e.to_string()))?;
        start_time += 1024;
        progress(0.55 + 0.4 * (i as f32 / total.max(1) as f32));
    }
    writer
        .write_end()
        .map_err(|e| ConvertError::Encode(e.to_string()))?;
    drop(writer);
    progress(0.97);

    let mut data = std::fs::read(out_path)?;
    fix_esds_sl_len(&mut data);
    std::fs::write(out_path, &data)?;
    Ok(())
}
