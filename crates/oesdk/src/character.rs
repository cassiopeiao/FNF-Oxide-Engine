//! OOP proxy for a character. Every method is a host call in disguise.

use crate::cast::CharacterId;
use crate::ctx::Ctx;
use crate::events::Vec2;

/// Borrowed handle to one on-stage character.
///
/// The proxy is cheap to create and never caches engine state, so it is always
/// safe to re-acquire it inside a callback.
pub struct Character<'a, 'h> {
    ctx: &'a mut Ctx<'h>,
    id: CharacterId,
}

impl<'a, 'h> Character<'a, 'h> {
    pub(crate) fn new(ctx: &'a mut Ctx<'h>, id: CharacterId) -> Self {
        Self { ctx, id }
    }

    pub fn id(&self) -> CharacterId {
        self.id
    }

    /// Plays a named animation ("singLEFT", "idle", "hey", ...).
    pub fn play_anim(&mut self, anim: &str) {
        self.ctx.host.play_anim(self.id, anim);
    }

    /// Restarts the idle/dance loop.
    pub fn dance(&mut self) {
        self.ctx.host.play_anim(self.id, "idle");
    }

    pub fn position(&mut self) -> Vec2 {
        self.ctx.host.character_position(self.id)
    }

    pub fn set_position(&mut self, pos: Vec2) {
        self.ctx.host.set_character_position(self.id, pos);
    }

    pub fn set_opacity(&mut self, alpha: f32) {
        self.ctx.host.set_character_opacity(self.id, alpha);
    }

    pub fn set_flip_x(&mut self, flip: bool) {
        self.ctx.host.set_character_flip(self.id, flip);
    }
}
