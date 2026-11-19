//! ABI types shared by mods and the engine. No engine dependency.
pub mod abi;
pub mod cast;
pub mod character;
pub mod ctx;
pub mod events;
pub mod sprite;

pub use abi::{ABI_VERSION, CALLBACKS};
pub use cast::{CharacterId, CharacterInfo};
pub use character::Character;
pub use ctx::{Camera, Ctx, HostBridge, PlayState, Rng};
pub use events::{Hit, Judgement, Note, Side, Vec2};
pub use sprite::{SpriteBuilder, SpriteHandle, SpriteId};
