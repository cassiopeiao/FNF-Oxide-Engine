//! # oesdk
//!
//! SDK for building **FNF: Oxide Engine** mods as WebAssembly components.
//!
//! Two halves live in this crate:
//!
//! * the **guest API** (`prelude`) that a mod links against — an OOP facade that
//!   deliberately mirrors the shapes modders know from v-slice (`Character`,
//!   `Camera`, `PlayState`) while never exposing an ECS type;
//! * the **CLI** (`oesdk` binary) that turns a mod project into a signed
//!   `.fnfm` package.
//!
//! The ABI itself is defined once, in `wit/oesdk.wit`. Rust is the first-class
//! language; C/C++/TypeScript shells are generated from the same WIT file, so
//! the contract lives in one place.
//!
//! Licensed under MIT OR Apache-2.0.

pub mod content;
pub mod pack;

pub mod manifest;
pub mod mod_trait;
pub mod prelude;
pub mod registry;

pub use mod_trait::Mod;

// ABI surface lives in `oesdk_api`; re-export it so mods only depend on `oesdk`.
pub use oesdk_api::{
    ABI_VERSION, CALLBACKS, Camera, Character, CharacterId, CharacterInfo, Ctx, Hit, HostBridge,
    Judgement, Note, PlayState, Rng, Side, SpriteBuilder, SpriteHandle, SpriteId, Vec2, abi, cast,
    character, ctx, events, sprite,
};
pub use oesdk_macro::register_mod;
