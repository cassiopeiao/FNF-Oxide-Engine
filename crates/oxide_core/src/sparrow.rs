//! Sparrow texture-atlas loading.
//!
//! Friday Night Funkin' ships most of its hand-drawn characters as Starling
//! Sparrow atlases: one PNG plus an XML file describing named sub-rectangles.
//! Bevy's built-in texture-atlas loader does not read that XML, so this module
//! registers a small custom asset loader.  The parsed atlas is used both by the
//! gameplay stage and by the title-screen dancer.

use std::collections::HashMap;
use std::path::{Path, PathBuf};

use bevy::asset::io::Reader;
use bevy::asset::{AssetLoader, LoadContext};
use bevy::prelude::*;
use quick_xml::Reader as XmlReader;
use quick_xml::events::Event;

/// One named frame inside a Sparrow atlas.
#[derive(Debug, Clone)]
pub struct SparrowFrame {
    /// Pixel rectangle inside the atlas texture.
    pub rect: Rect,
    /// Visible size of the trimmed frame.
    pub size: Vec2,
    /// Offset of the trimmed frame inside the original untrimmed frame.
    pub offset: Vec2,
    /// Original untrimmed frame size.
    pub source_size: Vec2,
}

/// A Sparrow atlas plus its texture handle.
#[derive(Asset, TypePath, Debug)]
pub struct SparrowAtlas {
    pub texture: Handle<Image>,
    pub frames: HashMap<String, SparrowFrame>,
}

impl SparrowAtlas {
    pub fn frame(&self, name: &str) -> Option<&SparrowFrame> {
        self.frames.get(name)
    }

    /// Every frame whose name is `prefix` followed by digits, in numeric order.
    pub fn clip(&self, prefix: &str) -> Vec<SparrowFrame> {
        let mut names: Vec<&String> = self
            .frames
            .keys()
            .filter(|name| {
                name.starts_with(prefix)
                    && name[prefix.len()..]
                        .chars()
                        .all(|character| character.is_ascii_digit())
            })
            .collect();
        names.sort();
        names
            .into_iter()
            .filter_map(|name| self.frames.get(name).cloned())
            .collect()
    }
}

#[derive(Default, TypePath)]
pub struct SparrowAtlasLoader;

impl AssetLoader for SparrowAtlasLoader {
    type Asset = SparrowAtlas;
    type Settings = ();
    type Error = std::io::Error;

    async fn load(
        &self,
        reader: &mut dyn Reader,
        _settings: &Self::Settings,
        load_context: &mut LoadContext<'_>,
    ) -> Result<Self::Asset, Self::Error> {
        let mut bytes = Vec::new();
        reader.read_to_end(&mut bytes).await?;
        let (image_name, frames) = parse_sparrow(&bytes).map_err(|error| {
            std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                format!("invalid Sparrow atlas: {error}"),
            )
        })?;

        let image_path = resolve_image_path(load_context.path().path(), &image_name);
        let texture = load_context.load(image_path);

        Ok(SparrowAtlas { texture, frames })
    }

    fn extensions(&self) -> &[&str] {
        &["xml"]
    }
}

fn resolve_image_path(atlas_path: &Path, image_name: &str) -> PathBuf {
    let parent = atlas_path.parent().unwrap_or_else(|| Path::new(""));
    parent.join(image_name)
}

fn parse_sparrow(
    bytes: &[u8],
) -> Result<(String, HashMap<String, SparrowFrame>), Box<dyn std::error::Error + Send + Sync>> {
    let mut reader = XmlReader::from_reader(bytes);
    reader.config_mut().trim_text(true);
    let mut image_name = String::new();
    let mut frames = HashMap::new();

    loop {
        match reader.read_event()? {
            Event::Start(element) | Event::Empty(element) => {
                let name = element.name();
                if name.as_ref() == b"TextureAtlas" {
                    for attribute in element.attributes() {
                        let attribute = attribute?;
                        if attribute.key.as_ref() == b"imagePath" {
                            image_name = attribute.unescape_value()?.into_owned();
                        }
                    }
                } else if name.as_ref() == b"SubTexture" {
                    let mut frame_name = String::new();
                    let mut x = 0.0;
                    let mut y = 0.0;
                    let mut width = 0.0;
                    let mut height = 0.0;
                    let mut frame_x = 0.0;
                    let mut frame_y = 0.0;
                    let mut frame_width = 0.0;
                    let mut frame_height = 0.0;
                    for attribute in element.attributes() {
                        let attribute = attribute?;
                        let value = attribute.unescape_value()?;
                        match attribute.key.as_ref() {
                            b"name" => frame_name = value.into_owned(),
                            b"x" => x = value.parse()?,
                            b"y" => y = value.parse()?,
                            b"width" => width = value.parse()?,
                            b"height" => height = value.parse()?,
                            b"frameX" => frame_x = value.parse()?,
                            b"frameY" => frame_y = value.parse()?,
                            b"frameWidth" => frame_width = value.parse()?,
                            b"frameHeight" => frame_height = value.parse()?,
                            _ => {}
                        }
                    }
                    if frame_name.is_empty() || width <= 0.0 || height <= 0.0 {
                        continue;
                    }
                    if frame_width <= 0.0 {
                        frame_width = width;
                    }
                    if frame_height <= 0.0 {
                        frame_height = height;
                    }
                    frames.insert(
                        frame_name,
                        SparrowFrame {
                            rect: Rect::new(x, y, x + width, y + height),
                            size: Vec2::new(width, height),
                            offset: Vec2::new(frame_x, frame_y),
                            source_size: Vec2::new(frame_width, frame_height),
                        },
                    );
                }
            }
            Event::Eof => break,
            _ => {}
        }
    }

    if image_name.is_empty() {
        return Err("missing TextureAtlas imagePath".into());
    }
    Ok((image_name, frames))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_subtextures_and_trim_offsets() {
        let xml = br#"<?xml version="1.0" encoding="utf-8"?>
            <TextureAtlas imagePath="sheet.png">
                <SubTexture name="BF idle0000" x="10" y="20" width="30" height="40"
                    frameX="-5" frameY="-6" frameWidth="50" frameHeight="60"/>
                <SubTexture name="BF idle0001" x="50" y="20" width="30" height="40"/>
            </TextureAtlas>"#;
        let (image, frames) = parse_sparrow(xml).expect("valid atlas");
        assert_eq!(image, "sheet.png");
        assert_eq!(frames.len(), 2);

        let first = frames.get("BF idle0000").expect("first frame");
        assert_eq!(first.rect.min, Vec2::new(10.0, 20.0));
        assert_eq!(first.rect.max, Vec2::new(40.0, 60.0));
        assert_eq!(first.size, Vec2::new(30.0, 40.0));
        assert_eq!(first.offset, Vec2::new(-5.0, -6.0));
        assert_eq!(first.source_size, Vec2::new(50.0, 60.0));

        let second = frames.get("BF idle0001").expect("second frame");
        assert_eq!(second.source_size, Vec2::new(30.0, 40.0));
    }

    #[test]
    fn rejects_atlas_without_image_path() {
        let xml = br#"<TextureAtlas><SubTexture name="a" x="0" y="0" width="1" height="1"/></TextureAtlas>"#;
        assert!(parse_sparrow(xml).is_err());
    }
}
