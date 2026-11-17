//! Application assembly shared by desktop, Android and iOS.

use bevy::asset::AssetPlugin;
use bevy::prelude::*;
use bevy::render::view::screenshot::{Screenshot, save_to_disk};
use bevy::window::{PresentMode, WindowResolution};

use oxide_core::theme::BG;
use oxide_core::{AppState, CorePlugin};
use oxide_data::OxideDataPlugin;
use oxide_game::GamePlugin;
use oxide_mod_runtime::ModPlugin;
use oxide_ui::UiPlugin;

/// Bevy resolves `assets/` next to the executable by default, which is wrong
/// for `cargo run` (the binary lives in `target/<profile>/`). In development we
/// therefore point at the crate's own asset directory; packaged builds keep the
/// platform default so Android/iOS read the files bundled with the app.
fn asset_root() -> String {
    #[cfg(not(any(target_os = "android", target_os = "ios")))]
    {
        if let Ok(path) = std::env::var("FNF_OXIDE_ASSETS") {
            return path;
        }
        concat!(env!("CARGO_MANIFEST_DIR"), "/assets").to_owned()
    }
    #[cfg(any(target_os = "android", target_os = "ios"))]
    {
        "assets".to_owned()
    }
}

pub fn run() {
    App::new()
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
                    file_path: asset_root(),
                    ..default()
                })
                .set(ImagePlugin::default_nearest()),
        )
        .insert_resource(ClearColor(BG))
        .init_state::<AppState>()
        .add_plugins((CorePlugin, OxideDataPlugin, GamePlugin, UiPlugin, ModPlugin))
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
