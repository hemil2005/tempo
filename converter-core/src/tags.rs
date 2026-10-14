use std::path::Path;

use symphonia::core::meta::StandardTag;

use crate::error::{ConvertError, Result};
use crate::format::OutputFormat;

#[derive(Debug, Clone, PartialEq)]
pub struct Picture {
    pub mime_type: String,
    pub data: Vec<u8>,
    pub description: Option<String>,
}

impl Picture {
    pub fn to_metadata_block_picture(&self) -> String {
        use base64::Engine;
        let mut block = Vec::new();
        block.extend_from_slice(&3u32.to_be_bytes());
        block.extend_from_slice(&(self.mime_type.len() as u32).to_be_bytes());
        block.extend_from_slice(self.mime_type.as_bytes());
        let desc = self.description.clone().unwrap_or_default();
        block.extend_from_slice(&(desc.len() as u32).to_be_bytes());
        block.extend_from_slice(desc.as_bytes());
        block.extend_from_slice(&0u32.to_be_bytes());
        block.extend_from_slice(&0u32.to_be_bytes());
        block.extend_from_slice(&0u32.to_be_bytes());
        block.extend_from_slice(&0u32.to_be_bytes());
        block.extend_from_slice(&(self.data.len() as u32).to_be_bytes());
        block.extend_from_slice(&self.data);
        base64::engine::general_purpose::STANDARD.encode(&block)
    }
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct Tags {
    pub title: Option<String>,
    pub artist: Option<String>,
    pub album: Option<String>,
    pub album_artist: Option<String>,
    pub genre: Option<String>,
    pub year: Option<String>,
    pub date: Option<String>,
    pub track_number: Option<String>,
    pub track_total: Option<String>,
    pub disc_number: Option<String>,
    pub composer: Option<String>,
    pub comment: Option<String>,
    pub lyrics: Option<String>,
    pub encoder: Option<String>,
    pub picture: Option<Picture>,
}

impl Tags {
    pub fn from_standard(std: &StandardTag) -> Self {
        let mut tags = Tags::default();
        match std {
            StandardTag::TrackTitle(v) => tags.title = Some(v.to_string()),
            StandardTag::Artist(v) => tags.artist = Some(v.to_string()),
            StandardTag::Album(v) => tags.album = Some(v.to_string()),
            StandardTag::AlbumArtist(v) => tags.album_artist = Some(v.to_string()),
            StandardTag::Genre(v) => tags.genre = Some(v.to_string()),
            StandardTag::Composer(v) => tags.composer = Some(v.to_string()),
            StandardTag::Comment(v) => tags.comment = Some(v.to_string()),
            StandardTag::Lyrics(v) => tags.lyrics = Some(v.to_string()),
            StandardTag::Encoder(v) => tags.encoder = Some(v.to_string()),
            StandardTag::TrackNumber(v) => tags.track_number = Some(v.to_string()),
            StandardTag::TrackTotal(v) => tags.track_total = Some(v.to_string()),
            StandardTag::DiscNumber(v) => tags.disc_number = Some(v.to_string()),
            StandardTag::RecordingYear(v) => tags.year = Some(v.to_string()),
            StandardTag::ReleaseYear(v) => tags.year = Some(v.to_string()),
            StandardTag::RecordingDate(v) => tags.date = Some(v.to_string()),
            StandardTag::ReleaseDate(v) => tags.date = Some(v.to_string()),
            _ => {}
        }
        tags
    }

    pub fn merge(&mut self, other: &Tags) {
        macro_rules! take {
            ($field:ident) => {
                if self.$field.is_none() {
                    self.$field = other.$field.clone();
                }
            };
        }
        take!(title);
        take!(artist);
        take!(album);
        take!(album_artist);
        take!(genre);
        take!(year);
        take!(date);
        take!(track_number);
        take!(track_total);
        take!(disc_number);
        take!(composer);
        take!(comment);
        take!(lyrics);
        take!(encoder);
        take!(picture);
    }

    pub fn write_to_file(&self, path: &Path, format: OutputFormat) -> Result<()> {
        if self.is_empty() {
            return Ok(());
        }
        match format {
            OutputFormat::Mp3 => self.write_id3(path),
            OutputFormat::Flac => self.write_flac(path),
            OutputFormat::M4a => self.write_m4a(path),
            _ => Ok(()),
        }
    }

    fn write_id3(&self, path: &Path) -> Result<()> {
        use id3::{Tag as Id3Tag, TagLike, Version};
        let mut tag = Id3Tag::new();
        if let Some(v) = &self.title {
            tag.set_title(v.clone());
        }
        if let Some(v) = &self.artist {
            tag.set_artist(v.clone());
        }
        if let Some(v) = &self.album {
            tag.set_album(v.clone());
        }
        if let Some(v) = &self.album_artist {
            tag.set_album_artist(v.clone());
        }
        if let Some(v) = &self.genre {
            tag.set_genre(v.clone());
        }
        if let Some(v) = &self.composer {
            tag.set_text("TCOM", v.clone());
        }
        if let Some(v) = &self.comment {
            tag.set_text("COMM", v.clone());
        }
        if let Some(v) = &self.year {
            tag.set_text("TYER", v.clone());
        }
        if let Some(v) = &self.track_number {
            if let Ok(n) = v.parse::<u32>() {
                tag.set_track(n);
            } else {
                tag.set_text("TRCK", v.clone());
            }
        }
        if let Some(v) = &self.disc_number {
            if let Ok(n) = v.parse::<u32>() {
                tag.set_disc(n);
            }
        }
        if let Some(pic) = &self.picture {
            tag.add_frame(id3::frame::Picture {
                mime_type: pic.mime_type.clone(),
                picture_type: id3::frame::PictureType::CoverFront,
                description: pic.description.clone().unwrap_or_default(),
                data: pic.data.clone(),
            });
        }
        tag.write_to_path(path, Version::Id3v24)
            .map_err(|e| ConvertError::Tags(e.to_string()))?;
        Ok(())
    }

    fn write_flac(&self, path: &Path) -> Result<()> {
        let mut tag =
            metaflac::Tag::read_from_path(path).map_err(|e| ConvertError::Tags(e.to_string()))?;
        if let Some(v) = &self.title {
            tag.set_vorbis("TITLE", vec![v.clone()]);
        }
        if let Some(v) = &self.artist {
            tag.set_vorbis("ARTIST", vec![v.clone()]);
        }
        if let Some(v) = &self.album {
            tag.set_vorbis("ALBUM", vec![v.clone()]);
        }
        if let Some(v) = &self.album_artist {
            tag.set_vorbis("ALBUMARTIST", vec![v.clone()]);
        }
        if let Some(v) = &self.genre {
            tag.set_vorbis("GENRE", vec![v.clone()]);
        }
        if let Some(v) = &self.year {
            tag.set_vorbis("DATE", vec![v.clone()]);
        }
        if let Some(v) = &self.date {
            tag.set_vorbis("DATE", vec![v.clone()]);
        }
        if let Some(v) = &self.track_number {
            tag.set_vorbis("TRACKNUMBER", vec![v.clone()]);
        }
        if let Some(v) = &self.track_total {
            tag.set_vorbis("TRACKTOTAL", vec![v.clone()]);
        }
        if let Some(v) = &self.disc_number {
            tag.set_vorbis("DISCNUMBER", vec![v.clone()]);
        }
        if let Some(v) = &self.composer {
            tag.set_vorbis("COMPOSER", vec![v.clone()]);
        }
        if let Some(v) = &self.comment {
            tag.set_vorbis("COMMENT", vec![v.clone()]);
        }
        if let Some(v) = &self.lyrics {
            tag.set_vorbis("LYRICS", vec![v.clone()]);
        }
        if let Some(pic) = &self.picture {
            tag.add_picture(
                pic.mime_type.clone(),
                metaflac::block::PictureType::CoverFront,
                pic.data.clone(),
            );
        }
        tag.save().map_err(|e| ConvertError::Tags(e.to_string()))?;
        Ok(())
    }

    fn write_m4a(&self, path: &Path) -> Result<()> {
        let mut tag = mp4ameta::Tag::default();
        if let Some(v) = &self.title {
            tag.set_title(v.clone());
        }
        if let Some(v) = &self.artist {
            tag.set_artist(v.clone());
        }
        if let Some(v) = &self.album {
            tag.set_album(v.clone());
        }
        if let Some(v) = &self.album_artist {
            tag.set_album_artist(v.clone());
        }
        if let Some(v) = &self.genre {
            tag.set_genre(v.clone());
        }
        if let Some(v) = &self.year {
            tag.set_year(v.clone());
        }
        if let Some(v) = &self.track_number {
            if let Ok(n) = v.parse::<u16>() {
                tag.set_track_number(n);
            }
        }
        if let Some(v) = &self.track_total {
            if let Ok(n) = v.parse::<u16>() {
                tag.set_total_tracks(n);
            }
        }
        if let Some(v) = &self.composer {
            tag.set_composer(v.clone());
        }
        if let Some(v) = &self.comment {
            tag.set_comment(v.clone());
        }
        if let Some(pic) = &self.picture {
            let img = if pic.mime_type.contains("png") {
                mp4ameta::Img::png(pic.data.clone())
            } else {
                mp4ameta::Img::jpeg(pic.data.clone())
            };
            tag.set_artwork(img);
        }
        tag.write_to_path(path)
            .map_err(|e| ConvertError::Tags(e.to_string()))?;
        Ok(())
    }

    pub fn vorbis_comments(&self) -> Vec<(String, String)> {
        let mut out = Vec::new();
        if let Some(v) = &self.title {
            out.push(("TITLE".to_string(), v.clone()));
        }
        if let Some(v) = &self.artist {
            out.push(("ARTIST".to_string(), v.clone()));
        }
        if let Some(v) = &self.album {
            out.push(("ALBUM".to_string(), v.clone()));
        }
        if let Some(v) = &self.album_artist {
            out.push(("ALBUMARTIST".to_string(), v.clone()));
        }
        if let Some(v) = &self.genre {
            out.push(("GENRE".to_string(), v.clone()));
        }
        if let Some(v) = &self.year {
            out.push(("DATE".to_string(), v.clone()));
        }
        if let Some(v) = &self.date {
            out.push(("DATE".to_string(), v.clone()));
        }
        if let Some(v) = &self.track_number {
            out.push(("TRACKNUMBER".to_string(), v.clone()));
        }
        if let Some(v) = &self.track_total {
            out.push(("TRACKTOTAL".to_string(), v.clone()));
        }
        if let Some(v) = &self.disc_number {
            out.push(("DISCNUMBER".to_string(), v.clone()));
        }
        if let Some(v) = &self.composer {
            out.push(("COMPOSER".to_string(), v.clone()));
        }
        if let Some(v) = &self.comment {
            out.push(("COMMENT".to_string(), v.clone()));
        }
        if let Some(v) = &self.lyrics {
            out.push(("LYRICS".to_string(), v.clone()));
        }
        out.push(("ENCODER".to_string(), "Tempo".to_string()));
        if let Some(pic) = &self.picture {
            out.push((
                "METADATA_BLOCK_PICTURE".to_string(),
                pic.to_metadata_block_picture(),
            ));
        }
        out
    }

    pub fn is_empty(&self) -> bool {
        self.title.is_none()
            && self.artist.is_none()
            && self.album.is_none()
            && self.album_artist.is_none()
            && self.genre.is_none()
            && self.year.is_none()
            && self.date.is_none()
            && self.track_number.is_none()
            && self.track_total.is_none()
            && self.disc_number.is_none()
            && self.composer.is_none()
            && self.comment.is_none()
            && self.lyrics.is_none()
            && self.picture.is_none()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn picture_handling_and_empty_check() {
        let mut tags = Tags::default();
        assert!(tags.is_empty());

        let pic = Picture {
            mime_type: "image/png".to_string(),
            data: vec![0x89, 0x50, 0x4E, 0x47],
            description: Some("Front Cover".to_string()),
        };
        tags.picture = Some(pic.clone());
        assert!(!tags.is_empty());

        let mut other = Tags::default();
        other.merge(&tags);
        assert!(other.picture.is_some());
        assert_eq!(other.picture.as_ref().unwrap().mime_type, "image/png");
        assert_eq!(other.picture.as_ref().unwrap().data, pic.data);
    }

    #[test]
    fn vorbis_comments_generation() {
        let tags = Tags {
            title: Some("Test Title".to_string()),
            artist: Some("Test Artist".to_string()),
            ..Default::default()
        };
        let comments = tags.vorbis_comments();
        assert!(comments
            .iter()
            .any(|(k, v)| k == "TITLE" && v == "Test Title"));
        assert!(comments
            .iter()
            .any(|(k, v)| k == "ARTIST" && v == "Test Artist"));
        assert!(comments.iter().any(|(k, v)| k == "ENCODER" && v == "Tempo"));
    }

    #[test]
    fn id3_tag_and_artwork_write_and_read() {
        use id3::TagLike;
        let temp_file = std::env::temp_dir().join("tempo_test_art.mp3");
        let _ = std::fs::remove_file(&temp_file);
        std::fs::write(&temp_file, b"").unwrap();

        let pic_data = vec![0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A];
        let tags = Tags {
            title: Some("Song With Art".to_string()),
            artist: Some("Hemil Shah".to_string()),
            album: Some("Tempo Album".to_string()),
            picture: Some(Picture {
                mime_type: "image/png".to_string(),
                data: pic_data.clone(),
                description: Some("Cover Art".to_string()),
            }),
            ..Default::default()
        };

        tags.write_to_file(&temp_file, OutputFormat::Mp3).unwrap();

        let read_tag = id3::Tag::read_from_path(&temp_file).unwrap();
        assert_eq!(read_tag.title(), Some("Song With Art"));
        assert_eq!(read_tag.artist(), Some("Hemil Shah"));
        assert_eq!(read_tag.album(), Some("Tempo Album"));

        let pic = read_tag.pictures().next().expect("picture should exist");
        assert_eq!(pic.mime_type, "image/png");
        assert_eq!(pic.data, pic_data);

        let _ = std::fs::remove_file(&temp_file);
    }

    #[test]
    fn metadata_block_picture_round_trip() {
        use base64::Engine;
        let pic_data = vec![0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A];
        let pic = Picture {
            mime_type: "image/png".to_string(),
            data: pic_data.clone(),
            description: Some("Cover".to_string()),
        };
        let encoded = pic.to_metadata_block_picture();
        let raw = base64::engine::general_purpose::STANDARD
            .decode(&encoded)
            .unwrap();
        assert_eq!(&raw[0..4], &3u32.to_be_bytes());
        let mime_len = u32::from_be_bytes([raw[4], raw[5], raw[6], raw[7]]) as usize;
        assert_eq!(&raw[8..8 + mime_len], b"image/png");
        assert!(raw.ends_with(&pic_data));

        let tags = Tags {
            picture: Some(pic),
            ..Default::default()
        };
        let comments = tags.vorbis_comments();
        let mbp = comments
            .iter()
            .find(|(k, _)| k == "METADATA_BLOCK_PICTURE")
            .expect("picture comment present");
        assert_eq!(mbp.1, encoded);
    }
}
