//! Built-in song catalog.
//!
//! `build.rs` scans `assets/preload/data/songs/*/*-metadata.json` at compile
//! time and emits a static table, so freep lay can be built without a runtime
//! file-system walk (which also works on Android where `assets/` lives inside
//! the APK).

use bevy::prelude::*;
use serde::{Deserialize, Serialize};

/// Difficulty ids match the keys used by chart and metadata files.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Difficulty {
    Easy,
    Normal,
    Hard,
}

impl Difficulty {
    pub const ALL: [Difficulty; 3] = [Self::Easy, Self::Normal, Self::Hard];

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Easy => "easy",
            Self::Normal => "normal",
            Self::Hard => "hard",
        }
    }

    pub fn display_name(self) -> &'static str {
        match self {
            Self::Easy => "EASY",
            Self::Normal => "NORMAL",
            Self::Hard => "HARD",
        }
    }

    pub fn cycle(self, delta: i32) -> Self {
        let index = Self::ALL.iter().position(|d| *d == self).unwrap_or(1) as i32;
        let next = (index + delta).rem_euclid(Self::ALL.len() as i32) as usize;
        Self::ALL[next]
    }

    #[allow(clippy::should_implement_trait)]
    pub fn from_str(value: &str) -> Option<Self> {
        match value {
            "easy" => Some(Self::Easy),
            "normal" => Some(Self::Normal),
            "hard" => Some(Self::Hard),
            _ => None,
        }
    }
}

/// One built-in song, available for freeplay.
#[derive(Debug, Clone, Copy)]
pub struct SongEntry {
    pub id: &'static str,
    pub song_name: &'static str,
    pub artist: &'static str,
    /// Difficulties that the base chart actually contains.
    pub difficulties: &'static [&'static str],
    /// Extra chart variations (`erect`, `pico`, ...).
    pub variations: &'static [&'static str],
    pub bpm: f32,
    pub stage: &'static str,
    pub album: &'static str,
}

impl SongEntry {
    /// Path of the base chart JSON relative to the asset root.
    pub fn chart_path(&self) -> String {
        format!("preload/data/songs/{id}/{id}-chart.json", id = self.id)
    }

    /// Path of an extra chart variation (`erect`, `pico`, ...).
    pub fn variation_chart_path(&self, variation: &str) -> String {
        format!(
            "preload/data/songs/{id}/{id}-chart-{variation}.json",
            id = self.id
        )
    }

    /// Instrumental path relative to the asset root.
    pub fn instrumental_path(&self) -> String {
        format!("songs/{}/Inst.ogg", self.id)
    }

    /// Player vocals path, if the song ships them.
    pub fn player_voices_path(&self) -> String {
        format!("songs/{}/Voices-bf.ogg", self.id)
    }

    pub fn supports(&self, difficulty: Difficulty) -> bool {
        self.difficulties
            .iter()
            .any(|name| *name == difficulty.as_str())
    }

    /// Pick the requested difficulty, falling back to the first available one.
    pub fn closest_difficulty(&self, wanted: Difficulty) -> Difficulty {
        if self.supports(wanted) {
            return wanted;
        }
        self.difficulties
            .iter()
            .find_map(|name| Difficulty::from_str(name))
            .unwrap_or(Difficulty::Normal)
    }
}

include!(concat!(env!("OUT_DIR"), "/builtin_songs.rs"));

pub fn song(id: &str) -> Option<&'static SongEntry> {
    BUILTIN_SONGS.iter().find(|entry| entry.id == id)
}

pub fn song_count() -> usize {
    BUILTIN_SONGS.len()
}

/// Register the chart asset and its loader.
pub struct OxideDataPlugin;

impl Plugin for OxideDataPlugin {
    fn build(&self, app: &mut App) {
        app.init_asset::<crate::chart::SongChartAsset>()
            .init_asset_loader::<crate::chart::SongChartLoader>();
        info!("Loaded {} built-in songs", song_count());
    }
}
