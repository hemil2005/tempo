# Rust Audio Converter

[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)
[![Rust: 2021](https://img.shields.io/badge/Rust-2021%20Edition-orange.svg)](https://www.rust-lang.org/)
[![Platform](https://img.shields.io/badge/Platform-Windows%20%7C%20Linux%20%7C%20macOS-lightgrey.svg)]()

> A fast, modern, privacy-first desktop audio converter built with Rust. 100% free and open-source — no cloud uploads, no ads, no file size limits.

![Audio Converter GUI](assets/gui-preview.png)

---

## ✨ Features

- **🔒 100% Offline & Private**: All conversions happen directly on your CPU. No accounts, no internet required, and zero telemetry.
- **⚡ Blazingly Fast**: Native performance powered by Rust, multi-threaded worker pools, and Link-Time Optimization (LTO).
- **🖥️ Clean Desktop GUI**: Built with [Iced](https://github.com/iced-rs/iced) — drag & drop audio files, track real-time progress, and batch convert full albums with one click.
- **💻 Power-User CLI**: Includes a dedicated, scriptable command-line interface (`converter-cli`) with rich argument parsing.
- **🏷️ Metadata & Tag Preservation**: Transfers title, artist, album, genre, year, and track numbers between formats seamlessly.
- **🎛️ Audio Customization**: Fine-tune CBR/VBR bitrates, sample rates (powered by the high-quality [Rubato](https://github.com/HEnquist/rubato) resampler), and channel configurations (Mono/Stereo).

---

## 🎧 Supported Formats

| Format | Decoding (Input) | Encoding (Output) | Encoder Engine |
| :--- | :---: | :---: | :--- |
| **MP3** (`.mp3`) | ✅ | ✅ | LAME (`mp3lame-encoder`) |
| **FLAC** (`.flac`) | ✅ | ✅ | `flacenc` (Lossless) |
| **WAV** (`.wav`, `.wave`) | ✅ | ✅ | `hound` (Lossless PCM) |
| **M4A / AAC** (`.m4a`, `.mp4`, `.aac`) | ✅ | ✅ | Fraunhofer FDK-AAC (`fdk-aac`) |
| **Opus** (`.opus`) | ✅ | ✅ | libopus (`audiopus`) |
| **Ogg Vorbis** (`.ogg`, `.oga`) | ✅ | ✅ | `vorbis_rs` |
| **AIFF** (`.aiff`, `.aif`) | ✅ | ✅ | `hound` (Lossless PCM) |

---

## 🚀 Getting Started

### 1. Graphical Desktop App

Launch the application:
- On Windows, double-click `converter-app.exe` (or run `Start-Process converter-app.exe`).

**Usage:**
1. Click **Add files** or drag & drop audio tracks directly into the window.
2. Select your target **Format** (e.g. MP3, FLAC, M4A) and adjust bitrate/quality options.
3. (Optional) Choose a destination folder (defaults to the source directory).
4. Click **Start** to convert.

---

### 2. Command-Line Interface (CLI)

The CLI tool allows quick single-file conversions or integration into automated scripts:

```powershell
# Basic conversion: WAV to MP3
converter-cli -f mp3 "song.wav" "song.mp3"

# High-quality FLAC to AAC (M4A) with custom bitrate
converter-cli -f m4a -b 256 "track.flac" "track.m4a"

# Convert to Opus with custom sample rate and mono channels
converter-cli -f opus -b 96 --sample-rate 48000 --channels 1 "input.wav" "output.opus"

# View all options
converter-cli --help
```

---

## 🛠️ Building From Source

### Prerequisites

- [Rust & Cargo](https://rustup.rs/) (Stable 1.80+)
- **Windows**: MinGW 64-bit GCC toolchain (required for C-based audio codecs: LAME, FDK-AAC, Opus).
- **Linux**: Standard build essentials (`gcc`, `g++`, `libopus-dev`, `pkg-config`).
- **macOS**: Xcode Command Line Tools.

### Build Commands

```bash
# Clone the repository
git clone https://github.com/your-username/rust-audio-converter.git
cd rust-audio-converter

# Build development binaries
cargo build

# Build optimized release binaries (LTO enabled)
# On Windows, use the helper script to configure MinGW PATH:
powershell -ExecutionPolicy Bypass -File .\build.ps1 build --release

# Run automated tests
powershell -ExecutionPolicy Bypass -File .\build.ps1 test
```

Binaries will be placed in:
- `target/release/converter-app.exe` (Desktop GUI)
- `target/release/converter-cli.exe` (Command-Line Tool)

---

## 📂 Project Architecture

This repository is structured as a Cargo workspace with three focused crates:

```text
rust-audio-converter/
├── converter-core/     # Pure conversion library (decoding, resampling, encoding, metadata tags)
├── converter-cli/      # Command-line binary interface using clap
├── converter-app/      # Cross-platform desktop GUI built on Iced
├── assets/             # Project screenshots and branding icons
└── testdata/           # Test audio clips and verification suites
```

---

## 🗺️ Roadmap

- [x] Multi-format decode and encode core engine
- [x] Multi-threaded batch conversion worker pool
- [x] Desktop GUI with drag-and-drop support
- [ ] Embedded album art / cover art preservation
- [ ] Folder-level recursive batch conversion
- [ ] Beginner-friendly preset profiles ("High Quality", "Podcast", "Lossless Archival")
- [ ] Automated GitHub Actions release binaries for Windows, macOS, and Linux
- [ ] Distribution via `winget` and package managers

---

## 📄 License & Attribution

This project is licensed under the [MIT License](LICENSE).  
For third-party codec licenses and attribution (Symphonia, LAME, Opus, FDK-AAC, Iced), see [THIRDPARTY.md](THIRDPARTY.md).
