use std::fmt;
use std::path::Path;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum AudioFormat {
    Opus,
    Ogg,
    Mp3,
    Wav,
    Flac,
    M4a,
    Aac,
    Aiff,
    Aif,
    Unknown,
}

impl AudioFormat {
    pub fn from_path(path: &Path) -> Self {
        match path
            .extension()
            .and_then(|e| e.to_str())
            .map(|e| e.to_ascii_lowercase())
            .as_deref()
        {
            Some("opus") => AudioFormat::Opus,
            Some("ogg") | Some("oga") => AudioFormat::Ogg,
            Some("mp3") => AudioFormat::Mp3,
            Some("wav") | Some("wave") => AudioFormat::Wav,
            Some("flac") => AudioFormat::Flac,
            Some("m4a") | Some("mp4") | Some("m4b") => AudioFormat::M4a,
            Some("aac") => AudioFormat::Aac,
            Some("aiff") => AudioFormat::Aiff,
            Some("aif") | Some("aifc") => AudioFormat::Aif,
            _ => AudioFormat::Unknown,
        }
    }

    pub fn extension(&self) -> &'static str {
        match self {
            AudioFormat::Opus => "opus",
            AudioFormat::Ogg => "ogg",
            AudioFormat::Mp3 => "mp3",
            AudioFormat::Wav => "wav",
            AudioFormat::Flac => "flac",
            AudioFormat::M4a => "m4a",
            AudioFormat::Aac => "aac",
            AudioFormat::Aiff => "aiff",
            AudioFormat::Aif => "aif",
            AudioFormat::Unknown => "bin",
        }
    }

    pub fn label(&self) -> &'static str {
        match self {
            AudioFormat::Opus => "Opus",
            AudioFormat::Ogg => "Ogg Vorbis",
            AudioFormat::Mp3 => "MP3",
            AudioFormat::Wav => "WAV",
            AudioFormat::Flac => "FLAC",
            AudioFormat::M4a => "M4A (AAC)",
            AudioFormat::Aac => "AAC",
            AudioFormat::Aiff => "AIFF",
            AudioFormat::Aif => "AIFF",
            AudioFormat::Unknown => "Unknown",
        }
    }

    pub fn is_decodable(&self) -> bool {
        !matches!(self, AudioFormat::Unknown)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum OutputFormat {
    Mp3,
    Opus,
    OggVorbis,
    Wav,
    Flac,
    M4a,
    Aiff,
}

impl OutputFormat {
    pub const ALL: [OutputFormat; 7] = [
        OutputFormat::Mp3,
        OutputFormat::Opus,
        OutputFormat::OggVorbis,
        OutputFormat::Wav,
        OutputFormat::Flac,
        OutputFormat::M4a,
        OutputFormat::Aiff,
    ];

    pub fn extension(&self) -> &'static str {
        match self {
            OutputFormat::Mp3 => "mp3",
            OutputFormat::Opus => "opus",
            OutputFormat::OggVorbis => "ogg",
            OutputFormat::Wav => "wav",
            OutputFormat::Flac => "flac",
            OutputFormat::M4a => "m4a",
            OutputFormat::Aiff => "aiff",
        }
    }

    pub fn label(&self) -> &'static str {
        match self {
            OutputFormat::Mp3 => "MP3",
            OutputFormat::Opus => "Opus",
            OutputFormat::OggVorbis => "Ogg Vorbis",
            OutputFormat::Wav => "WAV",
            OutputFormat::Flac => "FLAC",
            OutputFormat::M4a => "M4A (AAC)",
            OutputFormat::Aiff => "AIFF",
        }
    }

    pub fn is_lossless(&self) -> bool {
        matches!(
            self,
            OutputFormat::Wav | OutputFormat::Flac | OutputFormat::Aiff
        )
    }
}

impl fmt::Display for OutputFormat {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.label())
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Quality {
    Cbr { kbps: u32 },
    Vbr { level: u8 },
    Scale { value: f32 },
    Lossless,
}

impl Default for Quality {
    fn default() -> Self {
        Quality::Cbr { kbps: 192 }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct EncodeSettings {
    pub format: OutputFormat,
    pub quality: Quality,
    pub sample_rate: u32,
    pub channels: u16,
}

impl EncodeSettings {
    pub fn new(format: OutputFormat) -> Self {
        let quality = match format {
            OutputFormat::Opus => Quality::Cbr { kbps: 128 },
            OutputFormat::OggVorbis => Quality::Scale { value: 0.6 },
            OutputFormat::M4a => Quality::Cbr { kbps: 192 },
            OutputFormat::Wav | OutputFormat::Flac | OutputFormat::Aiff => Quality::Lossless,
            OutputFormat::Mp3 => Quality::Cbr { kbps: 192 },
        };
        EncodeSettings {
            format,
            quality,
            sample_rate: 0,
            channels: 0,
        }
    }
}
