//! Top-level application states.
//!
//! Mirrors the original Haxe state flow closely enough that each screen can be
//! ported side by side:
//!
//! `Boot -> Title -> MainMenu -> Freeplay -> Play -> Results`

use bevy::prelude::*;

#[derive(States, Default, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum AppState {
    /// Load fonts/settings and show a tiny loading screen.
    #[default]
    Boot,
    /// "Press enter" splash.
    Title,
    /// Main menu with Freeplay / Options / Quit.
    MainMenu,
    /// Week selection with difficulty (original StoryMenuState).
    StoryMenu,
    /// Song grid with difficulty selection.
    Freeplay,
    /// Volume, scroll direction, speed, offset and ghost tapping.
    Options,
    /// The actual rhythm gameplay screen.
    Play,
    /// Score, accuracy, rank and retry.
    Results,
}

impl AppState {
    /// States that are menus (useful for shared escape/back handling).
    pub fn is_menu(self) -> bool {
        matches!(
            self,
            Self::Title
                | Self::MainMenu
                | Self::StoryMenu
                | Self::Freeplay
                | Self::Options
                | Self::Results
        )
    }
}
