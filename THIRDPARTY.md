# Third-Party Notices and Licenses

This project incorporates open-source components and libraries. Below is an overview of the key components, their purpose, and their respective licenses.

| Component | Purpose | License | Upstream Project |
| :--- | :--- | :--- | :--- |
| **Symphonia** | Pure Rust audio decoding framework | [MPL-2.0](https://www.mozilla.org/en-US/MPL/2.0/) | [github.com/pdeljanov/Symphonia](https://github.com/pdeljanov/Symphonia) |
| **Iced** | Cross-platform GUI framework | [MIT](https://opensource.org/licenses/MIT) | [github.com/iced-rs/iced](https://github.com/iced-rs/iced) |
| **LAME (mp3lame-encoder)** | MP3 encoding library | [LGPL-2.0](https://www.gnu.org/licenses/old-licenses/lgpl-2.0.html) | [lame.sourceforge.io](https://lame.sourceforge.io/) |
| **libopus / audiopus** | Opus audio codec | [BSD-3-Clause](https://opensource.org/licenses/BSD-3-Clause) | [opus-codec.org](https://opus-codec.org/) |
| **libvorbis (vorbis_rs)** | Ogg Vorbis audio encoding | [BSD-style](https://xiph.org/licenses/bsd/) | [xiph.org/vorbis](https://xiph.org/vorbis/) |
| **FDK-AAC (fdk-aac)** | High-quality AAC encoding | [Fraunhofer FDK AAC License](https://github.com/mstorsjo/fdk-aac) | [github.com/mstorsjo/fdk-aac](https://github.com/mstorsjo/fdk-aac) |
| **flacenc** | FLAC lossless encoder | [Apache-2.0](https://www.apache.org/licenses/LICENSE-2.0) | [github.com/google/flacenc-rs](https://github.com/google/flacenc-rs) |
| **Hound** | WAV audio reading and writing | [Apache-2.0](https://www.apache.org/licenses/LICENSE-2.0) | [github.com/ruuda/hound](https://github.com/ruuda/hound) |
| **Rubato** | Fast audio sample rate conversion | [MIT](https://opensource.org/licenses/MIT) | [github.com/HEnquist/rubato](https://github.com/HEnquist/rubato) |
| **id3 / metaflac / mp4ameta** | Audio metadata & tag readers/writers | [MIT](https://opensource.org/licenses/MIT) | Various community crates |
| **rfd** | Native system file dialogs | [MIT](https://opensource.org/licenses/MIT) | [github.com/PolyMeilex/rfd](https://github.com/PolyMeilex/rfd) |
| **base64** | Base64 encoding (cover-art embedding) | [MIT / Apache-2.0](https://opensource.org/licenses/MIT) | [github.com/marshallpierce/rust-base64](https://github.com/marshallpierce/rust-base64) |

---

### Distribution Notes
When distributing binaries of this software, ensure compliance with LGPL requirements for dynamically or statically linked LAME objects and Fraunhofer terms for FDK AAC where applicable.
