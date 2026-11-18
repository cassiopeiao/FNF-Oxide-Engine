//! Application assembly shared by desktop, Android and iOS.

use bevy::asset::io::AssetSourceBuilder;
use bevy::asset::{AssetApp, AssetPlugin};
use bevy::prelude::*;
use bevy::render::view::screenshot::{Screenshot, save_to_disk};
use bevy::window::{PresentMode, WindowResolution};

use oxide_core::mods::{ModReader, ModRegistry, ModsPlugin, mods_dir};
use oxide_core::theme::BG;
use oxide_core::{AppState, CorePlugin};
use oxide_data::OxideDataPlugin;
use oxide_game::GamePlugin;
use oxide_mod_runtime::ModPlugin;
use oxide_ui::UiPlugin;

/// Resolve the asset directory at runtime.
///
/// No build-machine path may be baked into the binary: `CARGO_MANIFEST_DIR` is
/// an absolute path from the machine that compiled the crate, and a friend's
/// PC (or a phone) has no such directory. Instead we look for the assets where
/// they can actually be, relative to the executable or the working directory:
///
/// 1. `FNF_OXIDE_ASSETS` — explicit override for tests and portable builds.
/// 2. `<exe dir>/assets`, then up to five ancestors — this covers
///    `target/debug/OxideEngine` (walks up to the crate root) as well as a
///    packaged build that ships `assets/` next to the binary.
/// 3. `./assets` — relative to the current directory (`cargo run` from the
///    crate root).
/// 4. `assets` — Bevy's default: next to the executable on desktop, through
///    `AAssetManager` on Android and the app bundle on iOS.
fn asset_root() -> String {
    if let Some(path) = std::env::var_os("FNF_OXIDE_ASSETS") {
        return path.to_string_lossy().into_owned();
    }

    #[cfg(not(any(target_os = "android", target_os = "ios")))]
    {
        use std::path::Path;

        fn has_assets(dir: &Path) -> bool {
            // A single cheap marker is enough; the whole tree is 1.3 GB.
            dir.join("fonts").join("vcr.ttf").is_file()
        }

        if let Ok(exe) = std::env::current_exe() {
            let mut dir = exe.parent().map(Path::to_path_buf);
            while let Some(current) = dir {
                let candidate = current.join("assets");
                if has_assets(&candidate) {
                    return candidate.to_string_lossy().into_owned();
                }
                dir = current.parent().map(Path::to_path_buf);
            }
        }

        if has_assets(Path::new("assets")) {
            return "assets".to_owned();
        }
    }

    "assets".to_owned()
}

/// The resolved asset directory, logged once `LogPlugin` is up (an `info!`
/// before `App::new` would be dropped: tracing is not initialised yet).
#[derive(Resource, Clone)]
struct AssetRoot(String);

fn log_asset_root(root: Res<AssetRoot>) {
    info!("asset root: {}", root.0);
}

pub fn run() {
    let assets = asset_root();

    let registry = ModRegistry::scan(&mods_dir());
    info!(
        "mod source: {} package(s) under {}",
        registry.len(),
        mods_dir().display()
    );

    let source_registry = registry.clone();
    let mut app = App::new();
    // Must be registered before `AssetPlugin::build` creates the sources.
    app.register_asset_source(
        "mod",
        AssetSourceBuilder::new(move || Box::new(ModReader::new(source_registry.clone()))),
    );

    app.insert_resource(AssetRoot(assets.clone()))
        .add_plugins(
            DefaultPlugins
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        title: "Friday Night Funkin': Oxide Engine".to_owned(),
                        resolution: WindowResolution::new(1280, 720),
                        present_mode: PresentMode::AutoVsync,
                        resizable: true,
                        ..default()
                    }),
                    ..default()
                })
                .set(AssetPlugin {
                    file_path: assets,
                    ..default()
                })
                .set(ImagePlugin::default_nearest()),
        )
        .insert_resource(ClearColor(BG))
        .init_state::<AppState>()
        .add_plugins((
            CorePlugin,
            OxideDataPlugin,
            GamePlugin,
            UiPlugin,
            ModPlugin,
            ModsPlugin { registry },
        ))
        .add_systems(Startup, log_asset_root)
        .add_systems(Update, (debug_start_state, debug_screenshot))
        .run();
}

/// Development helper: `FNF_OXIDE_START=freeplay` jumps straight to a screen and
/// `FNF_OXIDE_SCREENSHOT=/tmp/x.png` saves one frame before the app exits.
fn debug_start_state(
    mut next: ResMut<NextState<AppState>>,
    time: Res<Time>,
    mut done: Local<bool>,
) {
    if *done {
        return;
    }
    if time.elapsed_secs() < 1.5 {
        return;
    }
    *done = true;
    let Ok(target) = std::env::var("FNF_OXIDE_START") else {
        return;
    };
    let state = match target.as_str() {
        "title" => AppState::Title,
        "mainmenu" => AppState::MainMenu,
        "storymenu" => AppState::StoryMenu,
        "freeplay" => AppState::Freeplay,
        "options" => AppState::Options,
        "play" => AppState::Play,
        "results" => AppState::Results,
        _ => return,
    };
    next.set(state);
}

fn debug_screenshot(mut commands: Commands, time: Res<Time>, mut done: Local<bool>) {
    if *done {
        return;
    }
    let Ok(path) = std::env::var("FNF_OXIDE_SCREENSHOT") else {
        return;
    };
    let delay = std::env::var("FNF_OXIDE_SHOT_DELAY")
        .ok()
        .and_then(|value| value.parse::<f32>().ok())
        .unwrap_or(4.0);
    if time.elapsed_secs() < delay {
        return;
    }
    *done = true;
    commands
        .spawn(Screenshot::primary_window())
        .observe(save_to_disk(path));
}
