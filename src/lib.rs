//! FNF: Oxide Engine — a Rust + Bevy reimplementation of Friday Night Funkin'.
//!
//! The crate is split into four workspace members:
//!
//! * [`oxide_core`] — app states, settings/save data, assets, audio, input and
//!   the chart conductor.
//! * [`oxide_data`] — chart/metadata formats plus the compile-time song
//!   catalog.
//! * [`oxide_game`] — the playable song session (notes, judgement, HUD).
//! * [`oxide_ui`] — title, main menu, freeplay, options and results screens.
//!
//! Desktop starts through `src/main.rs`. Android starts through the
//! `android_main` generated below by `#[bevy_main]`; iOS builds the normal
//! binary.

pub mod app;

pub use app::run;

#[cfg(target_os = "android")]
#[bevy::prelude::bevy_main]
fn main() {
    run();
}
