//! Chart model for Friday Night Funkin' v2 chart files.
//!
//! A chart is one JSON file per song *variation* (base, `erect`, `pico`, ...)
//! that contains note data for every difficulty plus a list of stage events.
//! Times are milliseconds, note directions are `0..=7` where `>= 4` selects the
//! player's side, and `l` is the hold length in milliseconds (absent for taps).

use std::collections::HashMap;

use bevy::asset::io::Reader;
use bevy::asset::{AssetLoader, LoadContext};
use bevy::prelude::*;
use serde::Deserialize;

use crate::catalog::Difficulty;

/// A single note or the head of a hold.
#[derive(Debug, Clone, Copy, Deserialize)]
pub struct Note {
    /// Time in milliseconds.
    pub t: f64,
    /// Direction `0..=7`. `d % 4` picks the lane, `d >= 4` means player side.
    pub d: u8,
    /// Hold length in milliseconds. `0.0` for a tap note.
    #[serde(default)]
    pub l: f64,
}

impl Note {
    pub fn lane(&self) -> usize {
        usize::from(self.d % 4)
    }

    pub fn is_player(&self) -> bool {
        // `SongNoteData.getStrumlineIndex()` is `floor(data / 4)`: strumline 0
        // (the player) owns data 0..3, strumline 1 (the opponent) owns 4..7.
        self.d < 4
    }

    pub fn is_hold(&self) -> bool {
        self.l > 0.0
    }

    pub fn end_ms(&self) -> f64 {
        self.t + self.l
    }
}

/// A stage event such as `FocusCamera` or `PlayAnimation`.
#[derive(Debug, Clone, Deserialize)]
pub struct ChartEvent {
    pub t: f64,
    pub e: String,
    #[serde(default)]
    pub v: serde_json::Value,
}

/// Raw chart file. Use [`SongChart::player_notes`] to get only the notes the
/// local player has to hit.
#[derive(Debug, Clone, Default, Deserialize)]
pub struct SongChart {
    #[serde(default)]
    pub version: String,
    #[serde(default, rename = "scrollSpeed")]
    pub scroll_speed: HashMap<String, f32>,
    #[serde(default)]
    pub events: Vec<ChartEvent>,
    #[serde(default)]
    pub notes: HashMap<String, Vec<Note>>,
}

impl SongChart {
    /// Every note of a difficulty, both sides, sorted by time.
    pub fn notes_for(&self, difficulty: Difficulty) -> Vec<Note> {
        let mut notes = self
            .notes
            .get(difficulty.as_str())
            .cloned()
            .unwrap_or_default();
        notes.sort_by(|a, b| a.t.total_cmp(&b.t));
        notes
    }

    /// Only the player's notes (directions 4..=7).
    pub fn player_notes(&self, difficulty: Difficulty) -> Vec<Note> {
        self.notes_for(difficulty)
            .into_iter()
            .filter(Note::is_player)
            .collect()
    }

    /// Chart-provided scroll speed for a difficulty, if present.
    pub fn scroll_speed_for(&self, difficulty: Difficulty) -> Option<f32> {
        self.scroll_speed.get(difficulty.as_str()).copied()
    }

    /// Last moment in the chart, including hold tails.
    pub fn duration_ms(&self) -> f64 {
        self.notes
            .values()
            .flatten()
            .map(Note::end_ms)
            .fold(0.0_f64, f64::max)
            .max(self.events.iter().map(|e| e.t).fold(0.0, f64::max))
    }
}

/// Asset wrapper so charts can be streamed by Bevy's asset server (which is the
/// only way to read files packed inside an Android APK).
#[derive(Asset, TypePath, Debug, Clone)]
pub struct SongChartAsset(pub SongChart);

#[derive(Default, TypePath)]
pub struct SongChartLoader;

impl AssetLoader for SongChartLoader {
    type Asset = SongChartAsset;
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
        let chart: SongChart = serde_json::from_slice(&bytes)
            .map_err(|error| std::io::Error::new(std::io::ErrorKind::InvalidData, error))?;
        Ok(SongChartAsset(chart))
    }

    fn extensions(&self) -> &[&str] {
        &["json"]
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::catalog::Difficulty;

    #[test]
    fn hold_end_is_start_plus_length() {
        let note = Note {
            t: 600.0,
            d: 7,
            l: 525.0,
        };
        assert_eq!(note.lane(), 3);
        assert!(note.is_player());
        assert!(note.is_hold());
        assert_eq!(note.end_ms(), 1125.0);
    }

    #[test]
    fn chart_selects_notes_by_difficulty() {
        let json = r#"{
            "version": "2.0.0",
            "scrollSpeed": { "easy": 1.2, "normal": 1.3 },
            "events": [ { "t": 4200, "e": "FocusCamera", "v": 1 } ],
            "notes": {
                "easy": [ { "t": 0, "d": 0 } ],
                "normal": [ { "t": 0, "d": 6 }, { "t": 600, "d": 7, "l": 525 } ],
                "hard": []
            }
        }"#;
        let chart: SongChart = serde_json::from_str(json).unwrap();
        assert_eq!(chart.notes_for(Difficulty::Normal).len(), 2);
        assert_eq!(chart.notes_for(Difficulty::Hard).len(), 0);
        assert_eq!(chart.scroll_speed_for(Difficulty::Normal), Some(1.3));
        assert_eq!(chart.duration_ms(), 4200.0);
        assert_eq!(chart.events[0].e, "FocusCamera");
    }
}
