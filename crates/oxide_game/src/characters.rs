//! Animated stage characters.
//!
//! All three cast members (BF, Dad, GF) are rendered from Adobe Animate atlases
//! through the vendored `bevy_animate_atlas` parser: every animation frame is
//! baked into a single mesh, so there is no giant pre-rendered sheet and the
//! native art size/flip is preserved.

use bevy::prelude::*;
use oxide_core::{AnimateAtlasAsset, AnimateVisual};

use crate::PlaySession;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CharacterSide {
    Player,
    Opponent,
    Girlfriend,
}

#[derive(Component)]
pub struct MeshCharacter {
    pub side: CharacterSide,
    pub material: Handle<ColorMaterial>,
    pub anim: &'static str,
}

/// Spawn one character; `atlas` is the Animation.json of its Animate sheet.
#[allow(clippy::too_many_arguments)]
pub fn spawn_mesh_character(
    commands: &mut Commands,
    materials: &mut Assets<ColorMaterial>,
    side: CharacterSide,
    atlas: Handle<AnimateAtlasAsset>,
    x: f32,
    foot_y: f32,
    scale: f32,
    flip: bool,
    z: f32,
) {
    let material = materials.add(ColorMaterial {
        color: Color::WHITE,
        texture: None,
        ..default()
    });
    commands.spawn((
        Name::new("Animate Character"),
        Mesh2d(Handle::default()),
        MeshMaterial2d(material.clone()),
        Transform::from_xyz(x, foot_y, z).with_scale(Vec3::splat(scale)),
        crate::play::PlayObject,
        MeshCharacter {
            side,
            material,
            anim: "Idle",
        },
        AnimateVisual {
            atlas,
            anim: "Idle".to_string(),
            frame: 0,
            elapsed: 0.0,
            hold_last: false,
            flip,
        },
    ));
}

fn sing_label(lane: usize) -> &'static str {
    match lane {
        0 => "Left",
        1 => "Down",
        2 => "Up",
        _ => "Right",
    }
}

/// Count down the one-shot sing / miss poses so the cast falls back to idle.
pub fn tick_sing_timers(time: Res<Time>, mut session: ResMut<PlaySession>) {
    let dt = time.delta_secs();
    if let Some((_, remaining)) = &mut session.player_sing {
        *remaining -= dt;
        if *remaining <= 0.0 {
            session.player_sing = None;
        }
    }
    if let Some((_, remaining)) = &mut session.player_miss {
        *remaining -= dt;
        if *remaining <= 0.0 {
            session.player_miss = None;
        }
    }
    if let Some((_, remaining)) = &mut session.opponent_sing {
        *remaining -= dt;
        if *remaining <= 0.0 {
            session.opponent_sing = None;
        }
    }
}

/// Pick the animation the session asks for, falling back to the idle loop.
#[allow(clippy::collapsible_if)]
pub fn drive_mesh_characters(
    atlases: Res<Assets<AnimateAtlasAsset>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    session: Res<PlaySession>,
    mut characters: Query<(&mut MeshCharacter, &mut AnimateVisual)>,
) {
    for (mut character, mut visual) in &mut characters {
        if !character.material.is_strong() {
            continue;
        }
        if let Some(asset) = atlases.get(&visual.atlas) {
            // The texture is shared by every animation; set it once.
            if let Some(mut material) = materials.get_mut(&character.material) {
                if material.texture.is_none() {
                    material.texture = Some(asset.texture.clone());
                }
            }
        }

        let wanted: &'static str = match character.side {
            CharacterSide::Player => {
                if let Some((lane, _)) = session.player_miss {
                    match lane {
                        0 => "Left Miss",
                        1 => "Down Miss",
                        2 => "Up Miss",
                        _ => "Right Miss",
                    }
                } else if let Some((lane, _)) = session.player_sing {
                    sing_label(lane)
                } else {
                    "Idle"
                }
            }
            CharacterSide::Opponent => session
                .opponent_sing
                .map(|(lane, _)| sing_label(lane))
                .unwrap_or("Idle"),
            CharacterSide::Girlfriend => "Idle",
        };

        if wanted != character.anim {
            character.anim = wanted;
            visual.anim = wanted.to_string();
            visual.frame = 0;
            visual.elapsed = 0.0;
        }
    }
}
