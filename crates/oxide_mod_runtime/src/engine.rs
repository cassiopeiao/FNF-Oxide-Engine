//! Engine + loaded module bookkeeping.

use std::collections::HashMap;

use bevy::prelude::Resource;
use wasmtime::component::Component;
use wasmtime::{Config, Engine};

use crate::manifest::ModManifest;

#[derive(Debug)]
pub enum ModError {
    Engine(wasmtime::Error),
    Deserialize(String),
    Manifest(String),
    Duplicate(String),
    InvalidApi { found: String, expected: String },
}

impl std::fmt::Display for ModError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Engine(e) => write!(f, "wasmtime: {e}"),
            Self::Deserialize(e) => write!(f, "component: {e}"),
            Self::Manifest(e) => write!(f, "manifest: {e}"),
            Self::Duplicate(id) => write!(f, "mod `{id}` is already loaded"),
            Self::InvalidApi { found, expected } => {
                write!(f, "api `{found}` does not match runtime ABI `{expected}`")
            }
        }
    }
}

impl std::error::Error for ModError {}

impl From<wasmtime::Error> for ModError {
    fn from(value: wasmtime::Error) -> Self {
        Self::Engine(value)
    }
}

pub struct LoadedMod {
    pub manifest: ModManifest,
    pub component: Component,
    /// Bytes the component came from, kept so the runtime can re-instantiate a
    /// pool without another disk read.
    pub wasm_len: usize,
}

#[derive(Resource)]
pub struct ModRuntime {
    engine: Engine,
    mods: HashMap<String, LoadedMod>,
    /// Epoch ticks once per `Update`; instances get a deadline of 1 tick.
    epoch: u64,
}

impl ModRuntime {
    pub fn new() -> Result<Self, ModError> {
        let mut config = Config::new();
        config.consume_fuel(true);
        config.epoch_interruption(true);
        let engine = Engine::new(&config)?;
        Ok(Self {
            engine,
            mods: HashMap::new(),
            epoch: 0,
        })
    }

    pub fn abi_version(&self) -> &'static str {
        oesdk::abi::ABI_VERSION
    }

    pub fn engine(&self) -> &Engine {
        &self.engine
    }

    pub fn mods(&self) -> impl Iterator<Item = &LoadedMod> {
        self.mods.values()
    }

    pub fn get(&self, id: &str) -> Option<&LoadedMod> {
        self.mods.get(id)
    }

    /// Loads a component from bytes. The manifest must already be validated.
    pub fn load(&mut self, manifest: ModManifest, wasm: &[u8]) -> Result<(), ModError> {
        if self.mods.contains_key(&manifest.id) {
            return Err(ModError::Duplicate(manifest.id));
        }
        if manifest.api != oesdk::abi::ABI_VERSION {
            return Err(ModError::InvalidApi {
                found: manifest.api,
                expected: oesdk::abi::ABI_VERSION.to_string(),
            });
        }
        let component =
            Component::new(&self.engine, wasm).map_err(|e| ModError::Deserialize(e.to_string()))?;
        let id = manifest.id.clone();
        self.mods.insert(
            id,
            LoadedMod {
                manifest,
                component,
                wasm_len: wasm.len(),
            },
        );
        Ok(())
    }

    /// Advances the deadline clock used by epoch interruption.
    pub fn tick_epoch(&mut self) {
        self.epoch = self.epoch.wrapping_add(1);
        self.engine.increment_epoch();
    }

    pub fn unload(&mut self, id: &str) -> bool {
        self.mods.remove(id).is_some()
    }
}
