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

pub mod abi;
pub mod cast;
pub mod character;
pub mod ctx;
pub mod events;
pub mod manifest;
pub mod mod_trait;
pub mod prelude;
pub mod registry;
pub mod sprite;

pub use ctx::{Ctx, HostBridge, LogLevel, PlayState, Rng};
pub use events::{Hit, Judgement, Note, Side, Vec2};
pub use mod_trait::Mod;
