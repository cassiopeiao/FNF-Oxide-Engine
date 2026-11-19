//! Callback dispatch shared by the generated guest glue and by headless tests.

use crate::abi;
use crate::ctx::Ctx;
use crate::events::{Hit, Note};
use crate::mod_trait::Mod;

/// Dynamic payload for a dispatch call.
#[derive(Debug, Clone, Copy)]
pub enum Payload<'a> {
    None,
    Update(f32),
    Beat(u32),
    NoteHit(&'a Hit),
    NoteMiss(&'a Note),
}

/// Routes a named callback into a mod instance.
pub fn dispatch(m: &mut dyn Mod, callback: &str, ctx: &mut Ctx<'_>, payload: Payload<'_>) {
    match (callback, payload) {
        (abi::CALLBACK_CREATE, _) => m.on_create(ctx),
        (abi::CALLBACK_UPDATE, Payload::Update(dt)) => m.on_update(ctx, dt),
        (abi::CALLBACK_SONG_START, _) => m.on_song_start(ctx),
        (abi::CALLBACK_BEAT, Payload::Beat(beat)) => m.on_beat(ctx, beat),
        (abi::CALLBACK_NOTE_HIT, Payload::NoteHit(hit)) => m.on_note_hit(ctx, hit),
        (abi::CALLBACK_NOTE_MISS, Payload::NoteMiss(note)) => m.on_note_miss(ctx, note),
        (abi::CALLBACK_SONG_END, _) => m.on_song_end(ctx),
        // Unknown callbacks are ignored on purpose: a newer host must not crash
        // an older mod (and vice versa).
        _ => {}
    }
}
