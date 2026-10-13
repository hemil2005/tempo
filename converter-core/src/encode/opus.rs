use std::fs::File;
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};

use audiopus::coder::Encoder as OpusEncoder;
use audiopus::{Application, Bitrate as OpusBitrate, Channels as OpusChannels, SampleRate};
use ogg::{PacketWriteEndInfo, PacketWriter};

use crate::encode::{guess_quality_kbps, resample, Prepared, Progress, Result};
use crate::error::ConvertError;
use crate::format::Quality;

const FRAME_MS: u32 = 20;
const OPUS_RATE: u32 = 48_000;
const PRE_SKIP: u16 = 312;

fn random_serial() -> u32 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| (d.subsec_nanos() ^ (d.as_secs() as u32)) | 1)
        .unwrap_or(0xC0FFEE01)
}

fn opus_head(channels: u8) -> Vec<u8> {
    let mut v = Vec::with_capacity(19);
    v.extend_from_slice(b"OpusHead");
    v.push(1);
    v.push(channels);
    v.extend_from_slice(&PRE_SKIP.to_le_bytes());
    v.extend_from_slice(&OPUS_RATE.to_le_bytes());
    v.extend_from_slice(&0i16.to_le_bytes());
    v.push(0);
    v
}

fn opus_tags(tags: &crate::tags::Tags) -> Vec<u8> {
    let mut v = Vec::new();
    v.extend_from_slice(b"OpusTags");
    let vendor = b"Tempo";
    v.extend_from_slice(&(vendor.len() as u32).to_le_bytes());
    v.extend_from_slice(vendor);
    let comments = tags.vorbis_comments();
    let user_count: u32 = comments.iter().filter(|(k, _)| k != "ENCODER").count() as u32;
    v.extend_from_slice(&user_count.to_le_bytes());
    for (k, val) in comments.iter().filter(|(k, _)| k != "ENCODER") {
        let entry = format!("{}={}", k, val);
        v.extend_from_slice(&(entry.len() as u32).to_le_bytes());
        v.extend_from_slice(entry.as_bytes());
    }
    v
}

pub fn write(
    prepared: &Prepared,
    quality: &Quality,
    tags: &crate::tags::Tags,
    out_path: &Path,
    progress: &Progress,
) -> Result<()> {
    let opus_channels = match prepared.channels {
        1 => OpusChannels::Mono,
        2 => OpusChannels::Stereo,
        _ => {
            return Err(ConvertError::Encode(
                "opus supports only mono and stereo".into(),
            ))
        }
    };

    let samples: Vec<f32> = if prepared.sample_rate == OPUS_RATE {
        prepared.samples.clone()
    } else {
        resample(
            &prepared.samples,
            prepared.channels as usize,
            prepared.sample_rate,
            OPUS_RATE,
        )?
    };
    let frames = samples.len() / prepared.channels as usize;

    let mut encoder = OpusEncoder::new(SampleRate::Hz48000, opus_channels, Application::Audio)
        .map_err(|e| ConvertError::Encode(e.to_string()))?;
    let kbps = guess_quality_kbps(*quality, 128);
    encoder
        .set_bitrate(OpusBitrate::BitsPerSecond((kbps * 1000) as i32))
        .map_err(|e| ConvertError::Encode(e.to_string()))?;

    let file = File::create(out_path)?;
    let mut writer = PacketWriter::new(file);
    let serial = random_serial();

    let channels_u8 = prepared.channels as u8;
    writer
        .write_packet(
            opus_head(channels_u8),
            serial,
            PacketWriteEndInfo::EndPage,
            0,
        )
        .map_err(|e| ConvertError::Encode(e.to_string()))?;
    writer
        .write_packet(opus_tags(tags), serial, PacketWriteEndInfo::NormalPacket, 0)
        .map_err(|e| ConvertError::Encode(e.to_string()))?;

    let frame_len = (OPUS_RATE / 1000 * FRAME_MS) as usize;
    let chan = prepared.channels as usize;
    let mut packet_buf = vec![0u8; 8192];
    let mut granule: u64 = 0;
    let mut consumed = 0usize;

    while consumed < frames {
        let mut frame = vec![0.0f32; frame_len * chan];
        let available = (frames - consumed).min(frame_len);
        frame[..available * chan]
            .copy_from_slice(&samples[consumed * chan..consumed * chan + available * chan]);
        consumed += available;
        granule += available as u64;

        let encoded = encoder
            .encode_float(&frame, &mut packet_buf)
            .map_err(|e| ConvertError::Encode(e.to_string()))?;
        let is_last = consumed >= frames;
        let packet: Vec<u8> = packet_buf[..encoded].to_vec();
        writer
            .write_packet(
                packet,
                serial,
                if is_last {
                    PacketWriteEndInfo::EndStream
                } else {
                    PacketWriteEndInfo::NormalPacket
                },
                granule,
            )
            .map_err(|e| ConvertError::Encode(e.to_string()))?;
        progress(0.05 + 0.9 * (consumed as f32 / frames as f32));
    }

    Ok(())
}
