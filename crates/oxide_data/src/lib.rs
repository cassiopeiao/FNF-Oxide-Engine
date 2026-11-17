//! FNF chart / song metadata model, compiled-in song catalog and loader
//! registration.

pub mod catalog;
pub mod chart;

pub use catalog::{BUILTIN_SONGS, Difficulty, OxideDataPlugin, SongEntry, song, song_count};
pub use chart::{ChartEvent, Note, SongChart, SongChartAsset, SongChartLoader};
