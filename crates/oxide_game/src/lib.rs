//! Rhythm gameplay: chart playback, input, judgement, HUD and results data.

pub mod characters;
pub mod hud;
pub mod play;

use bevy::prelude::*;
use oxide_core::AppState;
use oxide_data::{BUILTIN_SONGS, Difficulty, SongEntry};

pub use play::LaneState;

/// What the freeplay screen has selected.
#[derive(Resource, Debug, Clone, Copy)]
pub struct SongSelection {
    pub song_index: usize,
    pub difficulty: Difficulty,
}

impl Default for SongSelection {
    fn default() -> Self {
        let song_index = BUILTIN_SONGS
            .iter()
            .position(|entry| entry.id == "bopeebo")
            .unwrap_or(0);
        Self {
            song_index,
            difficulty: Difficulty::Normal,
        }
    }
}

impl SongSelection {
    pub fn entry(&self) -> &'static SongEntry {
        &BUILTIN_SONGS[self.song_index.min(BUILTIN_SONGS.len().saturating_sub(1))]
    }
}

/// One judgement result. Hold tails are not judged in this skeleton.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Judgement {
    Sick,
    Good,
    Bad,
    Shit,
    Miss,
}

impl Judgement {
    pub fn label(self) -> &'static str {
        match self {
            Self::Sick => "SICK!!",
            Self::Good => "GOOD",
            Self::Bad => "BAD",
            Self::Shit => "SHIT",
            Self::Miss => "MISS",
        }
    }

    /// Accuracy weight in `0.0..=1.0`.
    pub fn weight(self) -> f32 {
        match self {
            Self::Sick => 1.0,
            Self::Good => 0.75,
            Self::Bad => 0.4,
            Self::Shit => 0.2,
            Self::Miss => 0.0,
        }
    }

    pub fn points(self) -> u32 {
        match self {
            Self::Sick => 350,
            Self::Good => 200,
            Self::Bad => 100,
            Self::Shit => 50,
            Self::Miss => 0,
        }
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct JudgementCounts {
    pub sick: u32,
    pub good: u32,
    pub bad: u32,
    pub shit: u32,
    pub miss: u32,
}

impl JudgementCounts {
    pub fn record(&mut self, judgement: Judgement) {
        match judgement {
            Judgement::Sick => self.sick += 1,
            Judgement::Good => self.good += 1,
            Judgement::Bad => self.bad += 1,
            Judgement::Shit => self.shit += 1,
            Judgement::Miss => self.miss += 1,
        }
    }

    pub fn total(self) -> u32 {
        self.sick + self.good + self.bad + self.shit + self.miss
    }

    pub fn weighted(self) -> f32 {
        self.sick as f32 * Judgement::Sick.weight()
            + self.good as f32 * Judgement::Good.weight()
            + self.bad as f32 * Judgement::Bad.weight()
            + self.shit as f32 * Judgement::Shit.weight()
    }
}

/// Data handed to the results screen.
#[derive(Resource, Debug, Clone)]
pub struct LastResult {
    pub song_name: String,
    pub artist: String,
    pub difficulty: Difficulty,
    pub score: u32,
    pub accuracy: f32,
    pub max_combo: u32,
    pub counts: JudgementCounts,
    pub rank: &'static str,
    pub failed: bool,
}

impl Default for LastResult {
    fn default() -> Self {
        Self {
            song_name: "NO SONG".to_owned(),
            artist: "OXIDE ENGINE".to_owned(),
            difficulty: Difficulty::Normal,
            score: 0,
            accuracy: 0.0,
            max_combo: 0,
            counts: JudgementCounts::default(),
            rank: "F",
            failed: false,
        }
    }
}

impl LastResult {
    pub fn total_notes(&self) -> u32 {
        self.counts.total()
    }
}

/// FNF-style letter rank. `failed` always yields F.
pub fn rank_for(accuracy: f32, failed: bool) -> &'static str {
    if failed {
        return "F";
    }
    match accuracy {
        a if a >= 95.0 => "S+",
        a if a >= 90.0 => "S",
        a if a >= 80.0 => "A",
        a if a >= 70.0 => "B",
        a if a >= 60.0 => "C",
        _ => "D",
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PlayPhase {
    Loading,
    Countdown,
    Playing,
    Paused,
    Finished,
}

/// Everything that describes the current attempt.
#[derive(Resource)]
pub struct PlaySession {
    pub song: &'static SongEntry,
    pub difficulty: Difficulty,
    pub chart: Handle<oxide_data::SongChartAsset>,
    pub instrumental: Handle<AudioSource>,
    /// Player vocal track, mixed with the instrumental during play.
    pub vocals: Vec<Handle<AudioSource>>,
    pub phase: PlayPhase,
    pub paused_from: Option<PlayPhase>,
    pub conductor: oxide_core::Conductor,
    pub scroll_speed: f32,
    pub duration_ms: f64,
    pub total_notes: u32,
    pub score: u32,
    pub combo: u32,
    pub max_combo: u32,
    pub counts: JudgementCounts,
    pub health: f32,
    pub audio: Option<Entity>,
    pub popup: Option<(Judgement, f32)>,
    pub failed: bool,
    pub finished: bool,
    /// Lane and remaining seconds for the player's sing animation.
    pub player_sing: Option<(usize, f32)>,
    /// Lane and remaining seconds for the player's miss animation.
    pub player_miss: Option<(usize, f32)>,
    /// Lane and remaining seconds for the opponent's sing animation.
    pub opponent_sing: Option<(usize, f32)>,
    /// Opponent notes, used to drive Dad's sing poses.
    pub opponent_notes: Vec<oxide_data::Note>,
    pub next_opponent: usize,
    /// Stage definition for this song.
    pub stage: Handle<oxide_core::RawJson>,
    /// Whether the stage props have been spawned.
    pub stage_spawned: bool,
    /// Song metadata (characters, stage) for this chart.
    pub metadata: Handle<oxide_core::RawJson>,
    /// Whether the cast has been spawned from the metadata.
    pub cast_spawned: bool,
}

impl PlaySession {
    pub fn accuracy(&self) -> f32 {
        let total = self.counts.total();
        if total == 0 {
            return 100.0;
        }
        // Divide by the number of notes judged so far, but include misses so
        // accuracy does not inflate while the song is still running.
        let weighted = self.counts.weighted();
        (weighted / total as f32 * 100.0).clamp(0.0, 100.0)
    }
}

pub struct GamePlugin;

impl Plugin for GamePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<SongSelection>()
            .init_resource::<LastResult>()
            .init_resource::<LaneState>()
            .add_systems(OnEnter(AppState::Play), play::setup_play)
            .add_systems(OnExit(AppState::Play), play::cleanup_play)
            .add_systems(
                Update,
                (
                    play::toggle_pause,
                    play::begin_when_loaded,
                    play::tick_play,
                    play::read_input,
                    play::judge_notes,
                    play::miss_notes,
                    play::move_notes,
                    play::prepare_notes,
                    play::update_receptors,
                    play::update_splashes,
                    characters::tick_sing_timers,
                    characters::drive_mesh_characters,
                    play::spawn_stage,
                    play::layout_stage_props,
                    oxide_core::animate_asset::update_animate_visuals,
                    hud::update_hud,
                    hud::bob_health_icons,
                )
                    .chain()
                    .run_if(in_state(AppState::Play)),
            );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rank_thresholds() {
        assert_eq!(rank_for(100.0, false), "S+");
        assert_eq!(rank_for(95.0, false), "S+");
        assert_eq!(rank_for(94.9, false), "S");
        assert_eq!(rank_for(80.0, false), "A");
        assert_eq!(rank_for(59.9, false), "D");
        assert_eq!(rank_for(100.0, true), "F");
    }

    #[test]
    fn judgement_points_and_accuracy() {
        let mut counts = JudgementCounts::default();
        counts.record(Judgement::Sick);
        counts.record(Judgement::Good);
        assert_eq!(counts.total(), 2);
        assert!((counts.weighted() - 1.75).abs() < f32::EPSILON);
        assert_eq!(Judgement::Sick.points(), 350);
    }
}
