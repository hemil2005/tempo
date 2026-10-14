use std::fmt;

use converter_core::{EncodeSettings, OutputFormat, Quality};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Mp3Mode {
    Cbr,
    Vbr,
}

impl Mp3Mode {
    pub const ALL: [Mp3Mode; 2] = [Mp3Mode::Cbr, Mp3Mode::Vbr];

    pub fn label(&self) -> &'static str {
        match self {
            Mp3Mode::Cbr => "CBR",
            Mp3Mode::Vbr => "VBR",
        }
    }
}

impl fmt::Display for Mp3Mode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.label())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Preset {
    HighQuality,
    Balanced,
    SpaceSaver,
    Discord10Mb,
    Discord25Mb,
    YouTube,
    IPhone,
    Custom,
}

impl Preset {
    pub const ALL: [Preset; 8] = [
        Preset::HighQuality,
        Preset::Balanced,
        Preset::SpaceSaver,
        Preset::Discord10Mb,
        Preset::Discord25Mb,
        Preset::YouTube,
        Preset::IPhone,
        Preset::Custom,
    ];

    pub fn label(&self) -> &'static str {
        match self {
            Preset::HighQuality => "Highest Quality (320k / Best)",
            Preset::Balanced => "Balanced (Standard 192k)",
            Preset::SpaceSaver => "Smallest File (64k-96k Opus)",
            Preset::Discord10Mb => "Discord < 10 MB (Auto-bitrate)",
            Preset::Discord25Mb => "Discord < 25 MB (Nitro)",
            Preset::YouTube => "YouTube Music (256k AAC / 48kHz)",
            Preset::IPhone => "iPhone / Apple (256k AAC .m4a)",
            Preset::Custom => "Custom",
        }
    }
}

impl fmt::Display for Preset {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.label())
    }
}

#[derive(Debug, Clone)]
pub struct GuiSettings {
    pub format: OutputFormat,
    pub preset: Preset,
    pub mp3_mode: Mp3Mode,
    pub mp3_bitrate: u32,
    pub mp3_vbr: u8,
    pub opus_bitrate: u32,
    pub aac_bitrate: u32,
    pub vorbis_q: u8,
    pub sample_rate: u32,
    pub channels: u16,
    pub output_dir: String,
    pub overwrite: bool,
}

impl Default for GuiSettings {
    fn default() -> Self {
        GuiSettings {
            format: OutputFormat::Mp3,
            preset: Preset::Balanced,
            mp3_mode: Mp3Mode::Cbr,
            mp3_bitrate: 192,
            mp3_vbr: 2,
            opus_bitrate: 128,
            aac_bitrate: 192,
            vorbis_q: 6,
            sample_rate: 0,
            channels: 0,
            output_dir: String::new(),
            overwrite: false,
        }
    }
}

impl GuiSettings {
    pub fn apply_preset(&mut self, preset: Preset) {
        self.preset = preset;
        match preset {
            Preset::HighQuality => {
                self.mp3_bitrate = 320;
                self.mp3_mode = Mp3Mode::Cbr;
                self.aac_bitrate = 256;
                self.opus_bitrate = 192;
                self.vorbis_q = 8;
            }
            Preset::Balanced => {
                self.mp3_bitrate = 192;
                self.mp3_mode = Mp3Mode::Cbr;
                self.aac_bitrate = 192;
                self.opus_bitrate = 128;
                self.vorbis_q = 6;
            }
            Preset::SpaceSaver => {
                self.format = OutputFormat::Opus;
                self.mp3_bitrate = 96;
                self.mp3_mode = Mp3Mode::Cbr;
                self.aac_bitrate = 96;
                self.opus_bitrate = 64;
                self.vorbis_q = 3;
            }
            Preset::Discord10Mb => {
                self.format = OutputFormat::Opus;
                self.opus_bitrate = 128;
            }
            Preset::Discord25Mb => {
                self.format = OutputFormat::Opus;
                self.opus_bitrate = 160;
            }
            Preset::YouTube => {
                self.format = OutputFormat::M4a;
                self.aac_bitrate = 256;
                self.sample_rate = 48000;
                self.channels = 2;
            }
            Preset::IPhone => {
                self.format = OutputFormat::M4a;
                self.aac_bitrate = 256;
                self.sample_rate = 44100;
                self.channels = 2;
            }
            Preset::Custom => {}
        }
    }

    pub fn target_size_mb(&self) -> Option<f64> {
        match self.preset {
            Preset::Discord10Mb => Some(9.5),
            Preset::Discord25Mb => Some(24.0),
            _ => None,
        }
    }
    pub fn to_encode_settings(&self) -> EncodeSettings {
        let quality = match self.format {
            OutputFormat::Mp3 => match self.mp3_mode {
                Mp3Mode::Cbr => Quality::Cbr {
                    kbps: self.mp3_bitrate,
                },
                Mp3Mode::Vbr => Quality::Vbr {
                    level: self.mp3_vbr,
                },
            },
            OutputFormat::Opus => Quality::Cbr {
                kbps: self.opus_bitrate,
            },
            OutputFormat::M4a => Quality::Cbr {
                kbps: self.aac_bitrate,
            },
            OutputFormat::OggVorbis => Quality::Scale {
                value: self.vorbis_q as f32 / 10.0,
            },
            _ => Quality::Lossless,
        };
        EncodeSettings {
            format: self.format,
            quality,
            sample_rate: self.sample_rate,
            channels: self.channels,
        }
    }

    pub fn quality_summary(&self) -> String {
        match self.format {
            OutputFormat::Mp3 => match self.mp3_mode {
                Mp3Mode::Cbr => format!("{} kbps CBR", self.mp3_bitrate),
                Mp3Mode::Vbr => format!("VBR V{}", self.mp3_vbr),
            },
            OutputFormat::Opus => format!("{} kbps", self.opus_bitrate),
            OutputFormat::M4a => format!("{} kbps", self.aac_bitrate),
            OutputFormat::OggVorbis => format!("Q{:.1}", self.vorbis_q as f32 / 10.0),
            _ => "lossless".to_string(),
        }
    }
}

macro_rules! option_wrapper {
    ($name:ident, $inner:ty, $fmt:expr) => {
        #[derive(Debug, Clone, Copy, PartialEq, Eq)]
        pub struct $name(pub $inner);

        impl fmt::Display for $name {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                write!(f, $fmt, self.0)
            }
        }
    };
}

option_wrapper!(Kbps, u32, "{} kbps");
option_wrapper!(VbrLevel, u8, "V{}");
option_wrapper!(VorbisQ, u8, "Q{}");

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Rate(pub u32);

impl fmt::Display for Rate {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.0 {
            0 => write!(f, "Keep original"),
            r => write!(f, "{} Hz", r),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Chans(pub u16);

impl fmt::Display for Chans {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.0 {
            0 => write!(f, "Keep original"),
            1 => write!(f, "Mono"),
            2 => write!(f, "Stereo"),
            c => write!(f, "{} ch", c),
        }
    }
}

pub const MP3_RATES: [Kbps; 8] = [
    Kbps(64),
    Kbps(96),
    Kbps(128),
    Kbps(160),
    Kbps(192),
    Kbps(224),
    Kbps(256),
    Kbps(320),
];
pub const OPUS_RATES: [Kbps; 8] = [
    Kbps(32),
    Kbps(48),
    Kbps(64),
    Kbps(96),
    Kbps(128),
    Kbps(160),
    Kbps(192),
    Kbps(256),
];
pub const AAC_RATES: [Kbps; 7] = [
    Kbps(64),
    Kbps(96),
    Kbps(128),
    Kbps(160),
    Kbps(192),
    Kbps(224),
    Kbps(256),
];
pub const VBR_LEVELS: [VbrLevel; 10] = [
    VbrLevel(0),
    VbrLevel(1),
    VbrLevel(2),
    VbrLevel(3),
    VbrLevel(4),
    VbrLevel(5),
    VbrLevel(6),
    VbrLevel(7),
    VbrLevel(8),
    VbrLevel(9),
];
pub const VORBIS_QS: [VorbisQ; 11] = [
    VorbisQ(0),
    VorbisQ(1),
    VorbisQ(2),
    VorbisQ(3),
    VorbisQ(4),
    VorbisQ(5),
    VorbisQ(6),
    VorbisQ(7),
    VorbisQ(8),
    VorbisQ(9),
    VorbisQ(10),
];
pub const RATE_OPTS: [Rate; 3] = [Rate(0), Rate(44100), Rate(48000)];
pub const CHAN_OPTS: [Chans; 3] = [Chans(0), Chans(1), Chans(2)];
