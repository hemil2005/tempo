use std::path::PathBuf;

use converter_core::{
    convert, decode_file, Decoded, EncodeSettings, OutputFormat, Picture, Quality, Tags,
};

fn fixture(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("workspace root")
        .join("testdata")
        .join(name)
}

fn tagged_decoded() -> Decoded {
    let mut decoded = decode_file(&fixture("tone.wav")).expect("decode fixture");
    decoded.tags.picture = Some(Picture {
        mime_type: "image/png".to_string(),
        data: vec![0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A, 0x00],
        description: Some("Cover".to_string()),
    });
    decoded.tags.title = Some("Art Test".to_string());
    decoded
}

fn convert_and_read_back(format: OutputFormat, name: &str) -> Tags {
    let out = std::env::temp_dir().join(name);
    let _ = std::fs::remove_file(&out);
    let decoded = tagged_decoded();
    let settings = EncodeSettings {
        format,
        quality: Quality::Cbr { kbps: 96 },
        sample_rate: 0,
        channels: 0,
    };
    convert(&decoded, &settings, &out, &|_| {}).expect("convert");
    assert!(out.exists());
    let back = decode_file(&out).expect("decode back");
    let _ = std::fs::remove_file(&out);
    back.tags
}

#[test]
fn opus_preserves_cover_art() {
    let tags = convert_and_read_back(OutputFormat::Opus, "tempo_rt_art.opus");
    assert_eq!(tags.title.as_deref(), Some("Art Test"));
    let pic = tags.picture.expect("opus picture preserved");
    assert_eq!(pic.mime_type, "image/png");
    assert_eq!(
        pic.data,
        vec![0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A, 0x00]
    );
}

#[test]
fn ogg_preserves_cover_art() {
    let tags = convert_and_read_back(OutputFormat::OggVorbis, "tempo_rt_art.ogg");
    assert_eq!(tags.title.as_deref(), Some("Art Test"));
    let pic = tags.picture.expect("ogg picture preserved");
    assert_eq!(pic.mime_type, "image/png");
    assert_eq!(
        pic.data,
        vec![0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A, 0x00]
    );
}
