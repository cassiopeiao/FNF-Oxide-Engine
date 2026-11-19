//! Host state handed to every mod instance.
//!
//! The typed imports (`core`, `camera`, `sprites`, `audio`, `rng`) will be
//! generated from `oesdk/wit/oesdk.wit` with `wasmtime::component::bindgen!`
//! and implemented on this struct. Until that glue lands, `HostState` only
//! carries the limits and a command queue so the rest of the runtime can be
//! developed and tested against mocks.

use oesdk::prelude::{CharacterInfo, PlayState, Vec2};

/// Commands buffered by a mod and applied by the host at a safe point.
#[derive(Debug, Clone)]
pub enum ModCommand {
    CameraShake {
        intensity: f32,
        seconds: f32,
    },
    CameraFlash {
        rgba: u32,
        seconds: f32,
    },
    PlayAnim {
        character: String,
        anim: String,
    },
    SpawnSprite {
        asset: String,
        pos: Vec2,
        layer: i32,
        alpha: f32,
    },
    PlaySound {
        asset: String,
        volume: f32,
    },
    MoveCharacter {
        id: u32,
        x: f32,
        y: f32,
    },
    SetAlpha {
        id: u32,
        alpha: f32,
    },
    SetFlip {
        id: u32,
        flip: bool,
    },
    PlayAnimById {
        id: u32,
        anim: String,
    },
    DespawnSprite {
        id: u32,
    },
    MoveSprite {
        id: u32,
        x: f32,
        y: f32,
    },
    StopSound {
        id: u32,
    },
}

#[derive(Debug, Default)]
pub struct HostState {
    pub commands: Vec<ModCommand>,
    pub characters: Vec<CharacterInfo>,
    pub play: PlayState,
    pub rng_state: u64,
    pub next_sprite: u32,
    pub next_sound: u32,
}

impl HostState {
    pub fn new() -> Self {
        Self {
            rng_state: 0x9E37_79B9_7F4A_7C15,
            ..Default::default()
        }
    }

    /// xorshift64*; deterministic so replays and networked sessions agree.
    pub fn next_rng(&mut self) -> f32 {
        let mut x = self.rng_state;
        x ^= x >> 12;
        x ^= x << 25;
        x ^= x >> 27;
        self.rng_state = x;
        ((x.wrapping_mul(0x2545_F491_4F6C_DD1D) >> 40) as f32) / (1u32 << 24) as f32
    }
}
