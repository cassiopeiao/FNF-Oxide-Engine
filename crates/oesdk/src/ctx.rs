//! The single door between a mod and the engine.
//!
//! `Ctx` is the *only* thing a mod receives. It deliberately contains no Bevy
//! type, no `Entity`, no `Handle`: everything the mod can touch is a method
//! call that the host translates into an ECS operation at a safe point.

use crate::cast::{CharacterId, CharacterInfo};
use crate::character::Character;
use crate::events::{Side, Vec2};
use crate::sprite::{SpriteBuilder, SpriteId};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LogLevel {
    Trace,
    Info,
    Warn,
    Error,
}

/// Immutable snapshot of the current song, copied across the boundary.
#[derive(Debug, Clone, PartialEq)]
pub struct PlayState {
    pub song_name: String,
    pub difficulty: String,
    pub bpm: f32,
    pub score: u32,
    pub combo: u32,
    pub max_combo: u32,
    pub health: f32,
    pub accuracy: f32,
    pub total_notes: u32,
    /// Number of player-side characters on stage (>= 1).
    pub player_count: u8,
}

impl Default for PlayState {
    fn default() -> Self {
        Self {
            song_name: String::new(),
            difficulty: String::new(),
            bpm: 0.0,
            score: 0,
            combo: 0,
            max_combo: 0,
            health: 1.0,
            accuracy: 1.0,
            total_notes: 0,
            player_count: 1,
        }
    }
}

/// Implemented by the host (real engine) and by mocks (tests, CLI dry-run).
///
/// Every method here maps 1:1 onto a WIT import in `wit/oesdk.wit`.
pub trait HostBridge {
    fn log(&mut self, level: LogLevel, msg: &str);
    fn play_state(&mut self) -> PlayState;

    fn characters(&mut self) -> Vec<CharacterInfo>;
    fn character(&mut self, name: &str) -> Option<CharacterId>;
    fn character_position(&mut self, id: CharacterId) -> Vec2;
    fn set_character_position(&mut self, id: CharacterId, pos: Vec2);
    fn set_character_opacity(&mut self, id: CharacterId, alpha: f32);
    fn set_character_flip(&mut self, id: CharacterId, flip: bool);
    fn play_anim(&mut self, id: CharacterId, anim: &str);

    fn camera_shake(&mut self, intensity: f32, seconds: f32);
    fn camera_flash(&mut self, rgba: u32, seconds: f32);

    fn spawn_sprite(&mut self, asset: &str, pos: Vec2, layer: i32, alpha: f32) -> SpriteId;
    fn despawn(&mut self, id: SpriteId);
    fn set_sprite_position(&mut self, id: SpriteId, pos: Vec2);

    fn rng(&mut self) -> f32;
}

/// Per-callback context handed to a mod.
pub struct Ctx<'h> {
    pub(crate) host: &'h mut dyn HostBridge,
    frame: u64,
    dt: f32,
}

impl<'h> Ctx<'h> {
    pub fn new(host: &'h mut dyn HostBridge) -> Self {
        Self {
            host,
            frame: 0,
            dt: 0.0,
        }
    }

    /// Starts a new frame; the host calls this before dispatching `on-update`.
    pub fn begin_frame(&mut self, dt: f32) {
        self.frame = self.frame.wrapping_add(1);
        self.dt = dt;
    }

    pub fn frame(&self) -> u64 {
        self.frame
    }

    pub fn dt(&self) -> f32 {
        self.dt
    }

    pub fn log(&mut self, level: LogLevel, msg: &str) {
        self.host.log(level, msg);
    }

    pub fn play(&mut self) -> PlayState {
        self.host.play_state()
    }

    /// Every character currently on stage, in spawn order.
    pub fn characters(&mut self) -> Vec<CharacterInfo> {
        self.host.characters()
    }

    pub fn character(&mut self, name: &str) -> Option<Character<'_, 'h>> {
        let id = self.host.character(name)?;
        Some(Character::new(self, id))
    }

    /// N-th character on a given side, for multi-character stages.
    pub fn cast(&mut self, side: Side, slot: u8) -> Option<Character<'_, 'h>> {
        let id = self
            .host
            .characters()
            .into_iter()
            .find(|c| c.side == side && c.slot == slot)?
            .id;
        Some(Character::new(self, id))
    }

    pub fn camera(&mut self) -> Camera<'_, 'h> {
        Camera { ctx: self }
    }

    pub fn rng(&mut self) -> Rng<'_, 'h> {
        Rng { ctx: self }
    }

    pub fn spawn_sprite(&mut self, asset: impl Into<String>) -> SpriteBuilder<'_, 'h> {
        SpriteBuilder::new(self, asset)
    }
}

pub struct Camera<'a, 'h> {
    ctx: &'a mut Ctx<'h>,
}

impl Camera<'_, '_> {
    pub fn shake(&mut self, intensity: f32, seconds: f32) {
        self.ctx.host.camera_shake(intensity, seconds);
    }

    pub fn flash(&mut self, rgba: u32, seconds: f32) {
        self.ctx.host.camera_flash(rgba, seconds);
    }
}

pub struct Rng<'a, 'h> {
    ctx: &'a mut Ctx<'h>,
}

impl Rng<'_, '_> {
    /// Uniform in `0.0..1.0`.
    pub fn next_f32(&mut self) -> f32 {
        self.ctx.host.rng()
    }

    pub fn range(&mut self, lo: f32, hi: f32) -> f32 {
        lo + (hi - lo) * self.next_f32()
    }

    pub fn chance(&mut self, p: f32) -> bool {
        self.next_f32() < p
    }
}
