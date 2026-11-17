//! Adobe Animate (Sparrow v2) character atlases.
//!
//! The heavy lifting lives in the vendored `bevy_animate_atlas` crate; this
//! module only teaches the Bevy asset server how to load an `Animation.json`
//! together with its sibling `spritemap1.json` / `spritemap1.png`, and how to
//! turn the per-frame draw parts into a single textured mesh.

use std::collections::HashMap;

use bevy::asset::io::Reader;
use bevy::asset::{AssetId, AssetLoader, LoadContext, RenderAssetUsages};
use bevy::mesh::{Indices, Mesh, PrimitiveTopology};
use bevy::prelude::*;
use bevy_animate_atlas::animate_atlas::DrawPart;
use bevy_animate_atlas::{AnimateAtlas, parse_animate_atlas};

/// A parsed Animate atlas plus the texture it samples from.
#[derive(Asset, TypePath)]
pub struct AnimateAtlasAsset {
    pub atlas: AnimateAtlas,
    pub texture: Handle<Image>,
}

#[derive(Default, TypePath)]
pub struct AnimateAtlasLoader;

impl AssetLoader for AnimateAtlasLoader {
    type Asset = AnimateAtlasAsset;
    type Settings = ();
    type Error = std::io::Error;

    async fn load(
        &self,
        reader: &mut dyn Reader,
        _settings: &Self::Settings,
        load_context: &mut LoadContext<'_>,
    ) -> Result<Self::Asset, Self::Error> {
        let mut bytes = Vec::new();
        reader.read_to_end(&mut bytes).await?;
        let animation_json = String::from_utf8_lossy(&bytes).into_owned();

        let dir = load_context
            .path()
            .path()
            .parent()
            .map(|p| p.to_path_buf())
            .unwrap_or_default();

        let spritemap_bytes = load_context
            .read_asset_bytes(dir.join("spritemap1.json"))
            .await
            .map_err(std::io::Error::other)?;
        let spritemap_json = String::from_utf8_lossy(&spritemap_bytes).into_owned();
        let texture = load_context.load(dir.join("spritemap1.png"));

        let atlas = parse_animate_atlas(&spritemap_json, &animation_json);
        Ok(AnimateAtlasAsset { atlas, texture })
    }

    fn extensions(&self) -> &[&str] {
        &["json"]
    }
}

/// Marks an entity whose mesh is rebuilt from an Animate atlas every frame.
#[derive(Component)]
pub struct AnimateVisual {
    pub atlas: Handle<AnimateAtlasAsset>,
    pub anim: String,
    pub frame: usize,
    pub elapsed: f32,
    /// When set, the animation stops on its last frame instead of looping.
    pub hold_last: bool,
    /// Mirror the mesh horizontally (characters like BF are authored flipped).
    pub flip: bool,
}

/// Cache of already-built frame meshes, keyed by atlas + animation + frame.
#[derive(Resource, Default)]
pub struct AnimateMeshCache {
    meshes: HashMap<(AssetId<AnimateAtlasAsset>, String, usize, bool), Handle<Mesh>>,
    /// Feet-centre pivot per atlas, derived from the idle frame.
    pivots: HashMap<AssetId<AnimateAtlasAsset>, Vec2>,
}

/// Bounds of every draw part of a frame, in mesh space (y up).
pub fn frame_bounds(atlas: &AnimateAtlas, parts: &[DrawPart]) -> (Vec2, Vec2) {
    let mut min = Vec2::splat(f32::MAX);
    let mut max = Vec2::splat(f32::MIN);
    for part in parts {
        let Some(sprite) = atlas.sprites.get(&part.sprite_name) else {
            continue;
        };
        let [a, b, c, d, tx, ty] = part.matrix;
        let (quad_w, quad_h) = if sprite.rotated {
            (sprite.h, sprite.w)
        } else {
            (sprite.w, sprite.h)
        };
        for (lx, ly) in [(0.0, 0.0), (quad_w, 0.0), (quad_w, quad_h), (0.0, quad_h)] {
            let x = a * lx + c * ly + tx;
            let y = -(b * lx + d * ly + ty);
            min = min.min(Vec2::new(x, y));
            max = max.max(Vec2::new(x, y));
        }
    }
    (min, max)
}

/// Build one mesh containing every draw part of a frame.
pub fn build_frame_mesh(atlas: &AnimateAtlas, parts: &[DrawPart], pivot: Vec2, flip: bool) -> Mesh {
    let mut positions: Vec<[f32; 3]> = Vec::new();
    let mut uvs: Vec<[f32; 2]> = Vec::new();
    let mut normals: Vec<[f32; 3]> = Vec::new();
    let mut indices: Vec<u32> = Vec::new();

    for part in parts {
        let Some(sprite) = atlas.sprites.get(&part.sprite_name) else {
            continue;
        };
        let [a, b, c, d, tx, ty] = part.matrix;
        let (quad_w, quad_h) = if sprite.rotated {
            (sprite.h, sprite.w)
        } else {
            (sprite.w, sprite.h)
        };
        let transform = |lx: f32, ly: f32| -> [f32; 3] {
            let px = a * lx + c * ly + tx - pivot.x;
            [
                if flip { -px } else { px },
                -(b * lx + d * ly + ty) - pivot.y,
                0.0,
            ]
        };
        let base = positions.len() as u32;
        positions.extend([
            transform(0.0, 0.0),
            transform(quad_w, 0.0),
            transform(quad_w, quad_h),
            transform(0.0, quad_h),
        ]);

        let (sw, sh) = (atlas.sheet_w, atlas.sheet_h);
        let (uv_tl, uv_tr, uv_br, uv_bl) = if sprite.rotated {
            (
                [(sprite.x + sprite.w) / sw, sprite.y / sh],
                [(sprite.x + sprite.w) / sw, (sprite.y + sprite.h) / sh],
                [sprite.x / sw, (sprite.y + sprite.h) / sh],
                [sprite.x / sw, sprite.y / sh],
            )
        } else {
            (
                [sprite.x / sw, sprite.y / sh],
                [(sprite.x + sprite.w) / sw, sprite.y / sh],
                [(sprite.x + sprite.w) / sw, (sprite.y + sprite.h) / sh],
                [sprite.x / sw, (sprite.y + sprite.h) / sh],
            )
        };
        uvs.extend([uv_tl, uv_tr, uv_br, uv_bl]);
        normals.extend([[0.0, 0.0, 1.0]; 4]);
        if flip {
            indices.extend([base, base + 2, base + 1, base, base + 3, base + 2]);
        } else {
            indices.extend([base, base + 1, base + 2, base, base + 2, base + 3]);
        }
    }

    let mut mesh = Mesh::new(
        PrimitiveTopology::TriangleList,
        RenderAssetUsages::RENDER_WORLD,
    );
    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, positions);
    mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, uvs);
    mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, normals);
    mesh.insert_indices(Indices::U32(indices));
    mesh
}

/// Advance the animation clock and swap in the right cached mesh.
pub fn update_animate_visuals(
    time: Res<Time>,
    atlases: Res<Assets<AnimateAtlasAsset>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut cache: ResMut<AnimateMeshCache>,
    mut visuals: Query<(&mut AnimateVisual, &mut Mesh2d)>,
) {
    for (mut visual, mut mesh2d) in &mut visuals {
        let Some(asset) = atlases.get(&visual.atlas) else {
            continue;
        };
        let Some((_, duration, _, _)) = asset.atlas.animations.get(&visual.anim) else {
            continue;
        };
        if *duration == 0 {
            continue;
        }
        visual.elapsed += time.delta_secs();
        // Accumulate time and only step when a whole 1/24s frame has passed;
        // resetting `elapsed` every update froze the animation above 24 fps.
        let advance = (visual.elapsed * 24.0) as usize;
        if advance == 0 {
            continue;
        }
        let frame = if visual.hold_last {
            (visual.frame + advance).min(duration - 1)
        } else {
            (visual.frame + advance) % duration
        };
        visual.frame = frame;
        visual.elapsed -= advance as f32 / 24.0;

        let atlas_id = visual.atlas.id();
        let pivot = if let Some(pivot) = cache.pivots.get(&atlas_id) {
            *pivot
        } else {
            let parts = bevy_animate_atlas::get_animate_parts(&asset.atlas, "Idle", 0);
            let (min, max) = frame_bounds(&asset.atlas, &parts);
            let pivot = Vec2::new((min.x + max.x) * 0.5, min.y);
            cache.pivots.insert(atlas_id, pivot);
            pivot
        };
        let key = (atlas_id, visual.anim.clone(), frame, visual.flip);
        let handle = if let Some(handle) = cache.meshes.get(&key) {
            handle.clone()
        } else {
            let parts = bevy_animate_atlas::get_animate_parts(&asset.atlas, &visual.anim, frame);
            let handle = meshes.add(build_frame_mesh(&asset.atlas, &parts, pivot, visual.flip));
            cache.meshes.insert(key, handle.clone());
            handle
        };
        mesh2d.0 = handle;
    }
}

/// Convenience: current frame mesh handle for a visual (used at spawn time).
pub fn frame_mesh(
    asset: &AnimateAtlasAsset,
    anim: &str,
    frame: usize,
    meshes: &mut Assets<Mesh>,
) -> Handle<Mesh> {
    let parts = bevy_animate_atlas::get_animate_parts(&asset.atlas, anim, frame);
    let (min, max) = frame_bounds(&asset.atlas, &parts);
    let pivot = Vec2::new((min.x + max.x) * 0.5, min.y);
    meshes.add(build_frame_mesh(&asset.atlas, &parts, pivot, false))
}
