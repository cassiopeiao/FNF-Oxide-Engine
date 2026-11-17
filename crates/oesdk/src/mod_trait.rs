//! The trait a mod implements. All callbacks default to no-ops, so a mod only
//! writes the ones it cares about.

use crate::ctx::Ctx;
use crate::events::{Hit, Note};

pub trait Mod {
    fn on_create(&mut self, _ctx: &mut Ctx<'_>) {}
    fn on_update(&mut self, _ctx: &mut Ctx<'_>, _dt: f32) {}
    fn on_song_start(&mut self, _ctx: &mut Ctx<'_>) {}
    fn on_beat(&mut self, _ctx: &mut Ctx<'_>, _beat: u32) {}
    fn on_note_hit(&mut self, _ctx: &mut Ctx<'_>, _hit: &Hit) {}
    fn on_note_miss(&mut self, _ctx: &mut Ctx<'_>, _note: &Note) {}
    fn on_song_end(&mut self, _ctx: &mut Ctx<'_>) {}
}
