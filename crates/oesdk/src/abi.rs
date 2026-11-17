//! Names that cross the guest/host boundary.
//!
//! Keep this list in sync with `wit/oesdk.wit`; the runtime resolves callbacks
//! by these exact strings, so changing one is a breaking change.

/// Semantic version of the *mod ABI* (not of the engine).
pub const ABI_VERSION: &str = "0.1.0";

pub const CALLBACK_CREATE: &str = "on-create";
pub const CALLBACK_UPDATE: &str = "on-update";
pub const CALLBACK_SONG_START: &str = "on-song-start";
pub const CALLBACK_BEAT: &str = "on-beat";
pub const CALLBACK_NOTE_HIT: &str = "on-note-hit";
pub const CALLBACK_NOTE_MISS: &str = "on-note-miss";
pub const CALLBACK_SONG_END: &str = "on-song-end";

/// Every callback the host may call, in dispatch order.
pub const CALLBACKS: &[&str] = &[
    CALLBACK_CREATE,
    CALLBACK_UPDATE,
    CALLBACK_SONG_START,
    CALLBACK_BEAT,
    CALLBACK_NOTE_HIT,
    CALLBACK_NOTE_MISS,
    CALLBACK_SONG_END,
];

/// Returns true when `name` is a callback this ABI version knows about.
pub fn is_known_callback(name: &str) -> bool {
    CALLBACKS.contains(&name)
}
