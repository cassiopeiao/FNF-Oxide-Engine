//! Core services shared by every screen of FNF: Oxide Engine.

pub mod animate;
pub mod animate_asset;
pub mod assets;
pub mod conductor;
pub mod config;
pub mod mods;
pub mod plugin;
pub mod raw_json;
pub mod save;
pub mod settings;
pub mod sparrow;
pub mod states;
pub mod theme;

pub use animate::AnimateLoader;
pub use animate_asset::{AnimateAtlasAsset, AnimateAtlasLoader, AnimateVisual};
pub use assets::GameAssets;
pub use conductor::Conductor;
pub use config::{DESIGN_HEIGHT, DESIGN_WIDTH};
pub use plugin::CorePlugin;
pub use raw_json::{RawJson, RawJsonLoader};
pub use save::SaveStore;
pub use settings::Settings;
pub use sparrow::{SparrowAtlas, SparrowAtlasLoader, SparrowFrame};
pub use states::AppState;

use bevy::prelude::*;

/// Marker for a root entity that owns an entire screen.
///
/// Screens spawn their UI under a single [`ScreenRoot`] entity and remove it
/// with [`despawn_screen`] on state exit.
#[derive(Component, Debug, Default)]
pub struct ScreenRoot;

/// Remove every entity marked with `T`, including descendants.
pub fn despawn_screen<T: Component>(mut commands: Commands, roots: Query<Entity, With<T>>) {
    for entity in &roots {
        commands.entity(entity).despawn();
    }
}

/// Shared colors, timings and layout constants.
pub mod prelude {
    pub use crate::assets::GameAssets;
    pub use crate::conductor::Conductor;
    pub use crate::config::*;
    pub use crate::save::SaveStore;
    pub use crate::settings::Settings;
    pub use crate::sparrow::{SparrowAtlas, SparrowAtlasLoader, SparrowFrame};
    pub use crate::states::AppState;
    pub use crate::theme::*;
    pub use crate::{ScreenRoot, despawn_screen};
}
