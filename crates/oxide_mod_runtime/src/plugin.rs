//! Bevy integration.

use bevy::prelude::*;

use crate::engine::ModRuntime;

pub struct ModPlugin;

impl Plugin for ModPlugin {
    fn build(&self, app: &mut App) {
        match ModRuntime::new() {
            Ok(runtime) => {
                info!(
                    "mod runtime ready (ABI {}, {} mods loaded)",
                    runtime.abi_version(),
                    runtime.mods().count()
                );
                app.insert_resource(runtime);
                app.add_systems(Last, tick_epoch);
            }
            Err(err) => {
                // A missing wasmtime backend must never take the game down:
                // mods are optional.
                warn!("mod runtime disabled: {err}");
            }
        }
    }
}

fn tick_epoch(mut runtime: ResMut<ModRuntime>) {
    runtime.tick_epoch();
}
