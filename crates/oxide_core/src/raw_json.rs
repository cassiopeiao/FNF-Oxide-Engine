//! A tiny asset wrapper that loads a `.json` file as a raw string.
//!
//! Stage and character definitions are plain data; parsing them lazily in the
//! game layer keeps this crate free of gameplay types.

use bevy::asset::io::Reader;
use bevy::asset::{AssetLoader, LoadContext};
use bevy::prelude::*;

#[derive(Asset, TypePath, Clone)]
pub struct RawJson(pub String);

#[derive(Default, TypePath)]
pub struct RawJsonLoader;

impl AssetLoader for RawJsonLoader {
    type Asset = RawJson;
    type Settings = ();
    type Error = std::io::Error;

    async fn load(
        &self,
        reader: &mut dyn Reader,
        _settings: &Self::Settings,
        _load_context: &mut LoadContext<'_>,
    ) -> Result<Self::Asset, Self::Error> {
        let mut bytes = Vec::new();
        reader.read_to_end(&mut bytes).await?;
        Ok(RawJson(String::from_utf8_lossy(&bytes).into_owned()))
    }

    fn extensions(&self) -> &[&str] {
        &["json"]
    }
}
