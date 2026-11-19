//! Spawnable sprites. Builder-style so the call site reads like Flixel code.

use crate::ctx::Ctx;
use crate::events::Vec2;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct SpriteId(pub u32);

/// Builder returned by [`crate::ctx::Ctx::spawn_sprite`].
pub struct SpriteBuilder<'a, 'h> {
    ctx: &'a mut Ctx<'h>,
    asset: String,
    pos: Vec2,
    layer: i32,
    alpha: f32,
}

impl<'a, 'h> SpriteBuilder<'a, 'h> {
    pub(crate) fn new(ctx: &'a mut Ctx<'h>, asset: impl Into<String>) -> Self {
        Self {
            ctx,
            asset: asset.into(),
            pos: Vec2::default(),
            layer: 0,
            alpha: 1.0,
        }
    }

    pub fn at(mut self, pos: Vec2) -> Self {
        self.pos = pos;
        self
    }

    pub fn layer(mut self, layer: i32) -> Self {
        self.layer = layer;
        self
    }

    pub fn alpha(mut self, alpha: f32) -> Self {
        self.alpha = alpha;
        self
    }

    pub fn build(self) -> SpriteId {
        let asset = self.asset;
        let (pos, layer, alpha) = (self.pos, self.layer, self.alpha);
        self.ctx.host.spawn_sprite(&asset, pos, layer, alpha)
    }
}

/// Handle to a spawned sprite.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct SpriteHandle {
    pub id: SpriteId,
}

impl SpriteHandle {
    pub fn despawn(self, ctx: &mut Ctx<'_>) {
        ctx.host.despawn(self.id);
    }

    pub fn set_position(self, ctx: &mut Ctx<'_>, pos: Vec2) {
        ctx.host.set_sprite_position(self.id, pos);
    }
}
