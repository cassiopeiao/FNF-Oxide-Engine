//! Wires the core resources and the shared 2D camera into the app.

use bevy::camera::ScalingMode;
use bevy::prelude::*;
use bevy::window::PrimaryWindow;

use crate::animate::AnimateLoader;
use crate::assets::GameAssets;
use crate::config::{DESIGN_HEIGHT, DESIGN_WIDTH};
use crate::save::SaveStore;
use crate::sparrow::{SparrowAtlas, SparrowAtlasLoader};

pub struct CorePlugin;

impl Plugin for CorePlugin {
    fn build(&self, app: &mut App) {
        // Register the custom atlas asset before `GameAssets` creates handles
        // for it. Handles cannot be allocated for uninitialised asset types.
        app.init_asset::<SparrowAtlas>()
            .init_asset_loader::<SparrowAtlasLoader>()
            .init_asset_loader::<AnimateLoader>()
            .init_asset::<crate::animate_asset::AnimateAtlasAsset>()
            .init_asset_loader::<crate::animate_asset::AnimateAtlasLoader>()
            .init_resource::<crate::animate_asset::AnimateMeshCache>()
            .init_asset::<crate::raw_json::RawJson>()
            .init_asset_loader::<crate::raw_json::RawJsonLoader>();

        // Resources must exist before the initial `OnEnter(Boot)` run, so load
        // them eagerly at plugin-build time instead of in `PreStartup`.
        let store = SaveStore::new();
        let settings = store.load();
        let assets = {
            let asset_server = app.world().resource::<AssetServer>();
            GameAssets::load(asset_server)
        };
        info!(
            "FNF: Oxide Engine — settings file: {}",
            store.settings_path().display()
        );

        app.insert_resource(store)
            .insert_resource(settings)
            .insert_resource(assets)
            .add_systems(Startup, setup_camera)
            .add_systems(Update, (sync_ui_scale, apply_msaa));
    }
}

fn setup_camera(mut commands: Commands, settings: Res<crate::Settings>) {
    commands.spawn((
        Name::new("Main Camera"),
        Camera2d,
        settings.msaa_value(),
        Projection::Orthographic(OrthographicProjection {
            scaling_mode: ScalingMode::FixedVertical {
                viewport_height: DESIGN_HEIGHT,
            },
            ..OrthographicProjection::default_2d()
        }),
    ));
}

/// Push the anti-aliasing option onto the 2D camera whenever it changes.
fn apply_msaa(settings: Res<crate::Settings>, mut cameras: Query<&mut Msaa, With<Camera2d>>) {
    if !settings.is_changed() {
        return;
    }
    let value = settings.msaa_value();
    for mut msaa in &mut cameras {
        if *msaa != value {
            *msaa = value;
        }
    }
}

/// Keep the 1280x720 UI authored design proportional on every window size.
fn sync_ui_scale(
    windows: Query<&Window, With<PrimaryWindow>>,
    mut ui_scale: ResMut<UiScale>,
    mut last: Local<f32>,
) {
    let Ok(window) = windows.single() else {
        return;
    };
    let scale = (window.width() / DESIGN_WIDTH)
        .min(window.height() / DESIGN_HEIGHT)
        .max(0.35);
    if (scale - *last).abs() > 0.005 {
        ui_scale.0 = scale;
        *last = scale;
    }
}
