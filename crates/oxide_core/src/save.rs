//! Platform-aware settings storage.
//!
//! * Desktop: the OS data directory (`~/.local/share/fnf-oxide` on Linux).
//! * Android: the app's internal files directory, supplied by `AndroidApp`.
//! * iOS: `$HOME/Documents/fnf-oxide` inside the app sandbox.
//!
//! `FNF_OXIDE_DATA_DIR` overrides everything, which is handy for tests and for
//! running portable builds from a USB stick.

use std::fs;
use std::path::{Path, PathBuf};

use bevy::prelude::*;

use crate::settings::Settings;

const SETTINGS_FILE: &str = "settings.json";

/// Location of the writable game data directory.
#[derive(Resource, Debug, Clone)]
pub struct SaveStore {
    pub dir: PathBuf,
}

impl Default for SaveStore {
    fn default() -> Self {
        Self::new()
    }
}

impl SaveStore {
    pub fn new() -> Self {
        let dir = data_dir();
        Self { dir }
    }

    pub fn settings_path(&self) -> PathBuf {
        self.dir.join(SETTINGS_FILE)
    }

    /// Load settings, falling back to defaults (and logging) on any error.
    pub fn load(&self) -> Settings {
        let path = self.settings_path();
        match fs::read_to_string(&path) {
            Ok(raw) => match serde_json::from_str::<Settings>(&raw) {
                Ok(mut settings) => {
                    settings.clamp();
                    settings
                }
                Err(error) => {
                    warn!("Could not parse {}: {error}", path.display());
                    Settings::default()
                }
            },
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => Settings::default(),
            Err(error) => {
                warn!("Could not read {}: {error}", path.display());
                Settings::default()
            }
        }
    }

    /// Persist settings. Errors are logged instead of crashing the game.
    pub fn save(&self, settings: &Settings) {
        if let Err(error) = self.try_save(settings) {
            warn!("Could not write settings: {error}");
        }
    }

    pub fn try_save(&self, settings: &Settings) -> std::io::Result<()> {
        fs::create_dir_all(&self.dir)?;
        let encoded = serde_json::to_string_pretty(settings)
            .map_err(|error| std::io::Error::new(std::io::ErrorKind::InvalidData, error))?;
        fs::write(self.settings_path(), encoded)
    }
}

fn data_dir() -> PathBuf {
    if let Some(overridden) = std::env::var_os("FNF_OXIDE_DATA_DIR") {
        return PathBuf::from(overridden);
    }

    if let Some(dir) = platform_data_dir() {
        return dir.join("fnf-oxide");
    }

    // Last resort: keep the game running even on an exotic platform.
    Path::new(".").join("fnf-oxide-data")
}

#[cfg(target_os = "android")]
fn platform_data_dir() -> Option<PathBuf> {
    bevy::android::ANDROID_APP
        .get()
        .and_then(|app| app.internal_data_path())
}

#[cfg(target_os = "ios")]
fn platform_data_dir() -> Option<PathBuf> {
    std::env::var_os("HOME").map(|home| PathBuf::from(home).join("Documents"))
}

#[cfg(not(any(target_os = "android", target_os = "ios")))]
fn platform_data_dir() -> Option<PathBuf> {
    if let Some(dir) = std::env::var_os("XDG_DATA_HOME") {
        return Some(PathBuf::from(dir));
    }

    #[cfg(target_os = "windows")]
    if let Some(dir) = std::env::var_os("APPDATA") {
        return Some(PathBuf::from(dir));
    }

    #[cfg(target_os = "macos")]
    if let Some(home) = std::env::var_os("HOME") {
        return Some(PathBuf::from(home).join("Library/Application Support"));
    }

    std::env::var_os("HOME").map(|home| PathBuf::from(home).join(".local/share"))
}
