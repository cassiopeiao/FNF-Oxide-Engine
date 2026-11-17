//! # oxide_mod_runtime
//!
//! Host side of the Oxide mod system: loads `.fnfm` packages, verifies them and
//! runs their WebAssembly components through [`wasmtime`].
//!
//! The ABI contract lives in `oesdk/wit/oesdk.wit`; the guest-side types live in
//! the `oesdk` crate. This crate is the *only* place where Bevy and wasmtime
//! meet, which keeps the contract free of ECS types.
//!
//! Current state: scaffolding. Module loading, limits and the plugin lifecycle
//! are wired; the typed `bindgen!` glue for imports/exports is the next step.

pub mod engine;
pub mod host;
pub mod manifest;
pub mod plugin;

pub use engine::{LoadedMod, ModError, ModRuntime};
pub use manifest::ModManifest;
pub use plugin::ModPlugin;
