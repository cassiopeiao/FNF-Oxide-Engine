//! User settings, persisted as JSON next to the save data.

use bevy::prelude::*;
use serde::{Deserialize, Serialize};

/// Persisted options. Every field has a sane default so older save files keep
/// working after new options are added.
#[derive(Resource, Debug, Clone, Serialize, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct Settings {
    /// 0.0..=1.0 master gain.
    pub master_volume: f32,
    /// 0.0..=1.0 music bus gain (instrumental + voices).
    pub music_volume: f32,
    /// 0.0..=1.0 sound effect bus gain.
    pub sfx_volume: f32,
    /// Receptors at the bottom, notes fall downwards.
    pub downscroll: bool,
    /// Multiplier applied on top of the chart's scroll speed.
    pub note_speed: f32,
    /// Allow pressing with no note without breaking combo.
    pub ghost_tapping: bool,
    /// Audio latency compensation in milliseconds, applied to judgement.
    pub offset_ms: f32,
    /// Multisample anti-aliasing samples (0 = off, then 2/4/8).
    pub msaa: u32,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            master_volume: 0.9,
            music_volume: 1.0,
            sfx_volume: 1.0,
            downscroll: false,
            note_speed: 1.0,
            ghost_tapping: true,
            offset_ms: 0.0,
            msaa: 0,
        }
    }
}

impl Settings {
    /// Keep every value inside a safe range. Called after loading and after
    /// every UI edit.
    pub fn clamp(&mut self) {
        self.master_volume = self.master_volume.clamp(0.0, 1.0);
        self.music_volume = self.music_volume.clamp(0.0, 1.0);
        self.sfx_volume = self.sfx_volume.clamp(0.0, 1.0);
        self.note_speed = self.note_speed.clamp(0.5, 2.5);
        self.offset_ms = self.offset_ms.clamp(-200.0, 200.0);
        self.msaa = match self.msaa {
            2 | 4 | 8 => self.msaa,
            _ => 0,
        };
    }

    /// Final gain for instrumental/voices.
    pub fn music_gain(&self) -> f32 {
        (self.master_volume * self.music_volume).clamp(0.0, 1.0)
    }

    /// Final gain for UI/game sound effects.
    pub fn sfx_gain(&self) -> f32 {
        (self.master_volume * self.sfx_volume).clamp(0.0, 1.0)
    }

    /// The Bevy `Msaa` value matching this setting.
    pub fn msaa_value(&self) -> Msaa {
        match self.msaa {
            2 => Msaa::Sample2,
            4 => Msaa::Sample4,
            8 => Msaa::Sample8,
            _ => Msaa::Off,
        }
    }

    /// Human-readable label used by the options row.
    pub fn msaa_label(&self) -> &'static str {
        match self.msaa {
            2 => "2X",
            4 => "4X",
            8 => "8X",
            _ => "OFF",
        }
    }

    /// Cycle Off -> 2X -> 4X -> 8X -> Off.
    pub fn cycle_msaa(&mut self, delta: i32) {
        const STEPS: [u32; 4] = [0, 2, 4, 8];
        let index = STEPS.iter().position(|v| *v == self.msaa).unwrap_or(0) as i32;
        let next = (index + delta).rem_euclid(STEPS.len() as i32) as usize;
        self.msaa = STEPS[next];
    }
}
