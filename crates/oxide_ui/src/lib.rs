//! Menus, HUD-adjacent screens and shared widgets for FNF: Oxide Engine.

pub mod debug;
pub mod freeplay;
pub mod menu;
pub mod options;
pub mod results;
pub mod story;
pub mod title;
pub mod widgets;

use bevy::prelude::*;
use oxide_core::{AppState, ScreenRoot, despawn_screen};

pub struct UiPlugin;

impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<freeplay::FreeplayCursor>()
            .add_systems(Startup, debug::spawn_debug_overlay)
            .add_systems(Update, debug::update_debug_overlay)
            .add_systems(OnEnter(AppState::Boot), title::spawn_boot)
            .add_systems(Update, title::boot_tick.run_if(in_state(AppState::Boot)))
            .add_systems(OnExit(AppState::Boot), despawn_screen::<ScreenRoot>)
            .add_systems(OnEnter(AppState::Title), title::spawn_title)
            .add_systems(
                Update,
                (
                    title::title_input,
                    title::title_advance,
                    title::animate_title,
                )
                    .run_if(in_state(AppState::Title)),
            )
            .add_systems(OnExit(AppState::Title), despawn_screen::<ScreenRoot>)
            .add_systems(OnEnter(AppState::MainMenu), menu::spawn_main_menu)
            .add_systems(
                Update,
                (
                    menu::main_menu_input,
                    menu::animate_menu_buttons,
                    menu::tick_menu_gate,
                )
                    .run_if(in_state(AppState::MainMenu)),
            )
            .add_systems(OnExit(AppState::MainMenu), despawn_screen::<ScreenRoot>)
            .add_systems(OnEnter(AppState::StoryMenu), story::spawn_story)
            .add_systems(
                Update,
                (
                    story::story_input,
                    story::refresh_story,
                    story::animate_story,
                )
                    .run_if(in_state(AppState::StoryMenu)),
            )
            .add_systems(OnExit(AppState::StoryMenu), despawn_screen::<ScreenRoot>)
            .add_systems(OnEnter(AppState::Freeplay), freeplay::spawn_freeplay)
            .add_systems(
                Update,
                (
                    freeplay::freeplay_input,
                    freeplay::scroll_capsules,
                    freeplay::style_capsules,
                    freeplay::refresh_difficulty,
                )
                    .run_if(in_state(AppState::Freeplay)),
            )
            .add_systems(OnExit(AppState::Freeplay), despawn_screen::<ScreenRoot>)
            .add_systems(OnEnter(AppState::Options), options::spawn_options)
            .add_systems(
                Update,
                (options::options_keys, options::refresh_values)
                    .run_if(in_state(AppState::Options)),
            )
            .add_systems(OnExit(AppState::Options), despawn_screen::<ScreenRoot>)
            .add_systems(OnEnter(AppState::Results), results::spawn_results)
            .add_systems(
                Update,
                results::results_keys.run_if(in_state(AppState::Results)),
            )
            .add_systems(OnExit(AppState::Results), despawn_screen::<ScreenRoot>)
            // One global style pass for every standard menu button.
            .add_systems(Update, (widgets::button_style, widgets::image_button_style));
    }
}
