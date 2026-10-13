use std::fs::File;
use std::path::Path;

use symphonia::core::codecs::audio::AudioDecoderOptions;
use symphonia::core::codecs::registry::CodecRegistry;
use symphonia::core::errors::Error as SymphoniaError;
use symphonia::core::formats::probe::Hint;
use symphonia::core::formats::{FormatOptions, TrackType};
use symphonia::core::io::MediaSourceStream;
use symphonia::core::meta::MetadataOptions;
use symphonia_adapter_libopus::OpusDecoder;

use id3::TagLike;

use crate::error::{ConvertError, Result};
use crate::tags::Tags;

pub struct Decoded {
    pub samples: Vec<f32>,
    pub channels: usize,
    pub sample_rate: u32,
    pub bits_per_sample: Option<u32>,
    pub tags: Tags,
}

impl Decoded {
    pub fn frames(&self) -> usize {
        self.samples.len() / self.channels
    }
}

fn codec_registry() -> CodecRegistry {
    let mut registry = CodecRegistry::new();
    symphonia::default::register_enabled_codecs(&mut registry);
    registry.register_audio_decoder::<OpusDecoder>();
    registry
}

pub fn decode_file(path: &Path) -> Result<Decoded> {
    let src = File::open(path).map_err(ConvertError::Io)?;
    let mss = MediaSourceStream::new(Box::new(src), Default::default());

    let mut hint = Hint::new();
    if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
        hint.with_extension(ext);
    }

    let meta_opts: MetadataOptions = Default::default();
    let fmt_opts: FormatOptions = Default::default();
    let dec_opts: AudioDecoderOptions = Default::default();

    let mut format = symphonia::default::get_probe()
        .probe(&hint, mss, fmt_opts, meta_opts)
        .map_err(|e| ConvertError::Decode(e.to_string()))?;

    let (track_id, codec_params) = {
        let track = format
            .default_track(TrackType::Audio)
            .ok_or_else(|| ConvertError::Decode("no default audio track".into()))?;
        let params = track
            .codec_params
            .as_ref()
            .and_then(|cp| cp.audio())
            .cloned()
            .ok_or_else(|| ConvertError::Decode("audio codec parameters missing".into()))?;
        (track.id, params)
    };

    let codecs = codec_registry();
    let mut decoder = codecs
        .make_audio_decoder(&codec_params, &dec_opts)
        .map_err(|e| ConvertError::UnsupportedCodec(e.to_string()))?;

    let mut samples: Vec<f32> = Vec::new();
    let mut chunk: Vec<f32> = Vec::new();
    let mut rate: u32 = 0;
    let mut channels: usize = 0;
    let mut bits_per_sample: Option<u32> = None;
    let mut tags: Tags = Tags::default();

    loop {
        let packet = match format.next_packet() {
            Ok(Some(p)) => p,
            Ok(None) => break,
            Err(SymphoniaError::IoError(_)) | Err(SymphoniaError::DecodeError(_)) => continue,
            Err(e) => return Err(ConvertError::Decode(e.to_string())),
        };

        if packet.track_id != track_id {
            continue;
        }

        while !format.metadata().is_latest() {
            format.metadata().pop();
        }

        match decoder.decode(&packet) {
            Ok(decoded) => {
                let spec = decoded.spec();
                if rate == 0 {
                    rate = spec.rate();
                    channels = spec.channels().count();
                    bits_per_sample = codec_params.bits_per_sample;
                }
                decoded.copy_to_vec_interleaved(&mut chunk);
                samples.extend_from_slice(&chunk);
            }
            Err(SymphoniaError::IoError(_)) | Err(SymphoniaError::DecodeError(_)) => continue,
            Err(SymphoniaError::ResetRequired) => break,
            Err(e) => return Err(ConvertError::Decode(e.to_string())),
        }
    }

    let native_tags = read_native_tags(path);
    tags.merge(&native_tags);

    if let Some(rev) = format.metadata().skip_to_latest() {
        for tag in rev.media.tags.iter() {
            if let Some(std) = tag.std.as_ref() {
                let t = Tags::from_standard(std);
                tags.merge(&t);
            }
        }
        for visual in rev.media.visuals.iter() {
            if tags.picture.is_none() {
                let mime = visual
                    .media_type
                    .clone()
                    .unwrap_or_else(|| "image/jpeg".to_string());
                tags.picture = Some(crate::tags::Picture {
                    mime_type: mime,
                    data: visual.data.to_vec(),
                    description: None,
                });
            }
        }
    }

    if rate == 0 {
        return Err(ConvertError::Decode("no audio samples decoded".into()));
    }

    Ok(Decoded {
        samples,
        channels,
        sample_rate: rate,
        bits_per_sample,
        tags,
    })
}

fn read_native_tags(path: &Path) -> Tags {
    let mut tags = Tags::default();
    let ext = path
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("")
        .to_ascii_lowercase();

    match ext.as_str() {
        "mp3" => {
            if let Ok(id3_tag) = id3::Tag::read_from_path(path) {
                tags.title = id3_tag.title().map(|s| s.to_string());
                tags.artist = id3_tag.artist().map(|s| s.to_string());
                tags.album = id3_tag.album().map(|s| s.to_string());
                tags.album_artist = id3_tag.album_artist().map(|s| s.to_string());
                tags.genre = id3_tag.genre().map(|s| s.to_string());
                tags.year = id3_tag.year().map(|y| y.to_string());
                tags.track_number = id3_tag.track().map(|t| t.to_string());
                tags.track_total = id3_tag.total_tracks().map(|t| t.to_string());
                tags.disc_number = id3_tag.disc().map(|d| d.to_string());
                if let Some(pic) = id3_tag.pictures().next() {
                    tags.picture = Some(crate::tags::Picture {
                        mime_type: pic.mime_type.clone(),
                        data: pic.data.clone(),
                        description: Some(pic.description.clone()),
                    });
                }
            }
        }
        "flac" => {
            if let Ok(flac_tag) = metaflac::Tag::read_from_path(path) {
                if let Some(vorbis) = flac_tag.vorbis_comments() {
                    tags.title = vorbis.title().and_then(|v| v.first().cloned());
                    tags.artist = vorbis.artist().and_then(|v| v.first().cloned());
                    tags.album = vorbis.album().and_then(|v| v.first().cloned());
                    tags.genre = vorbis.genre().and_then(|v| v.first().cloned());
                }
                if let Some(pic) = flac_tag.pictures().next() {
                    tags.picture = Some(crate::tags::Picture {
                        mime_type: pic.mime_type.clone(),
                        data: pic.data.clone(),
                        description: Some(pic.description.clone()),
                    });
                }
            }
        }
        "m4a" | "mp4" | "aac" => {
            if let Ok(m4a_tag) = mp4ameta::Tag::read_from_path(path) {
                tags.title = m4a_tag.title().map(|s| s.to_string());
                tags.artist = m4a_tag.artist().map(|s| s.to_string());
                tags.album = m4a_tag.album().map(|s| s.to_string());
                tags.album_artist = m4a_tag.album_artist().map(|s| s.to_string());
                tags.genre = m4a_tag.genre().map(|s| s.to_string());
                tags.year = m4a_tag.year().map(|s| s.to_string());
                tags.track_number = m4a_tag.track_number().map(|t| t.to_string());
                tags.track_total = m4a_tag.total_tracks().map(|t| t.to_string());
                if let Some(art) = m4a_tag.artwork() {
                    let mime = match art.fmt {
                        mp4ameta::ImgFmt::Png => "image/png",
                        _ => "image/jpeg",
                    };
                    tags.picture = Some(crate::tags::Picture {
                        mime_type: mime.to_string(),
                        data: art.data.to_vec(),
                        description: None,
                    });
                }
            }
        }
        _ => {}
    }
    tags
}
