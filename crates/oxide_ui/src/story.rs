//! Story-mode week selection, mirroring the original `StoryMenuState`: a solid
//! backdrop, the week's title art, a difficulty selector, the three character
//! silhouettes and left/right arrows to flip through weeks.

use bevy::input::touch::Touches;
use bevy::prelude::*;
use oxide_core::config::{DESIGN_HEIGHT, DESIGN_WIDTH, screen_to_world};
use oxide_core::sparrow::{SparrowAtlas, SparrowFrame};
use oxide_core::{AppState, GameAssets, ScreenRoot};
use oxide_data::{BUILTIN_SONGS, Difficulty};
use oxide_game::SongSelection;

use crate::widgets::lime_text;

/// One week of the story campaign, in display order.
struct StoryWeek {
    /// Index into `GameAssets::story_titles`.
    title: usize,
    /// First song id, selected when the player confirms.
    first_song: &'static str,
    /// Prop asset stems (`storymenu/props/<stem>.xml`), opponent/bf/gf order.
    props: [&'static str; 3],
    /// Background colour.
    bg: (u8, u8, u8),
}

const STORY_WEEKS: [StoryWeek; 10] = [
    StoryWeek {
        title: 0,
        first_song: "bopeebo",
        props: ["dad", "bf", "gf"],
        bg: (0xF9, 0xCF, 0x51),
    },
    StoryWeek {
        title: 1,
        first_song: "spookeez",
        props: ["spooky", "bf", "gf"],
        bg: (0xF9, 0xCF, 0x51),
    },
    StoryWeek {
        title: 2,
        first_song: "pico",
        props: ["pico", "bf", "gf"],
        bg: (0xF9, 0xCF, 0x51),
    },
    StoryWeek {
        title: 3,
        first_song: "satin-panties",
        props: ["mom", "bf", "gf"],
        bg: (0xF9, 0xCF, 0x51),
    },
    StoryWeek {
        title: 4,
        first_song: "cocoa",
        props: ["parents-xmas", "bf", "gf"],
        bg: (0xF9, 0xCF, 0x51),
    },
    StoryWeek {
        title: 5,
        first_song: "senpai",
        props: ["senpai", "bf", "gf"],
        bg: (0xF9, 0xCF, 0x51),
    },
    StoryWeek {
        title: 6,
        first_song: "ugh",
        props: ["tankman", "bf", "gf"],
        bg: (0xF9, 0xCF, 0x51),
    },
    StoryWeek {
        title: 7,
        first_song: "darnell",
        props: ["darnell", "pico-player", "nene"],
        bg: (0x41, 0x3C, 0xAE),
    },
    StoryWeek {
        title: 8,
        first_song: "tutorial",
        props: ["gf", "bf", "gf"],
        bg: (0xF9, 0xCF, 0x51),
    },
    StoryWeek {
        title: 9,
        first_song: "spaghetti",
        props: ["spaghetti", "bf", "gf"],
        bg: (0xF9, 0xCF, 0x51),
    },
];

/// Which week + difficulty the story screen has selected.
#[derive(Resource)]
pub struct StorySelection {
    pub week: usize,
    pub difficulty: usize,
}

impl Default for StorySelection {
    fn default() -> Self {
        Self {
            week: 0,
            difficulty: 1,
        }
    }
}

/// An animated sprite: a character silhouette, or a single-frame arrow.
#[derive(Component)]
pub struct StoryProp {
    atlas: Handle<SparrowAtlas>,
    foot: Vec2,
    scale: f32,
    /// Prop slot (0=opponent, 1=bf, 2=gf). 3 marks an arrow (single frame).
    slot: usize,
    frames: Vec<SparrowFrame>,
    index: usize,
    elapsed: f32,
    ready: bool,
}

/// The currently displayed week title.
#[derive(Component)]
pub struct StoryTitle;

#[derive(Component)]
pub struct StoryDifficultyChip;

#[derive(Component)]
pub struct StoryBackdrop;

pub fn spawn_story(
    mut commands: Commands,
    assets: Res<GameAssets>,
    asset_server: Res<AssetServer>,
) {
    commands.insert_resource(StorySelection::default());

    commands.spawn((
        Sprite::from_color(
            Color::srgb_u8(
                STORY_WEEKS[0].bg.0,
                STORY_WEEKS[0].bg.1,
                STORY_WEEKS[0].bg.2,
            ),
            Vec2::new(DESIGN_WIDTH, DESIGN_HEIGHT),
        ),
        Transform::from_xyz(0.0, 0.0, -30.0),
        StoryBackdrop,
        ScreenRoot,
    ));

    commands.spawn((
        Sprite {
            image: assets.story_titles[0].clone(),
            ..default()
        },
        Transform::from_xyz(0.0, 200.0, -20.0),
        StoryTitle,
        ScreenRoot,
    ));

    commands.spawn((
        Sprite {
            image: assets.story_difficulties[1].clone(),
            ..default()
        },
        Transform::from_xyz(0.0, 90.0, -19.0),
        StoryDifficultyChip,
        ScreenRoot,
    ));

    let prop_slots = [
        (-300.0, -260.0, 0.6),
        (20.0, -260.0, 0.5),
        (330.0, -260.0, 0.5),
    ];
    for (slot, (x, y, scale)) in prop_slots.iter().enumerate() {
        let stem = STORY_WEEKS[0].props[slot];
        commands.spawn((
            Sprite::default(),
            Transform::from_xyz(*x, *y, -18.0 + slot as f32),
            StoryProp {
                atlas: asset_server.load(format!("preload/images/storymenu/props/{stem}.xml")),
                foot: Vec2::new(*x, *y),
                scale: *scale,
                slot,
                frames: Vec::new(),
                index: 0,
                elapsed: 0.0,
                ready: false,
            },
            ScreenRoot,
        ));
    }

    // Left / right arrows (slot 3, single-frame idle clips).
    let left = screen_to_world(462.0, 270.0);
    let right = screen_to_world(818.0, 270.0);
    for (x, prefix) in [(left.x, "leftIdle"), (right.x, "rightIdle")] {
        commands.spawn((
            Sprite::default(),
            Transform::from_xyz(x, 56.0, -15.0),
            StoryProp {
                atlas: assets.story_arrows.clone(),
                foot: Vec2::new(x, 56.0),
                scale: 0.8,
                slot: 3,
                frames: Vec::new(),
                index: 0,
                elapsed: 0.0,
                ready: false,
            },
            StoryArrow(prefix),
            ScreenRoot,
        ));
    }

    commands
        .spawn(crate::widgets::screen_bundle_clear())
        .with_children(|root| {
            root.spawn(lime_text(
                "LEFT/RIGHT: WEEK   UP/DOWN: DIFFICULTY   ENTER: PLAY   ESC: BACK",
                &assets,
                18.0,
            ));
        });
}

/// A marker distinguishing arrow sprites (single-frame clips) from props.
#[derive(Component)]
pub struct StoryArrow(pub &'static str);

/// Fill in the atlas image/rect once decoded, and animate idle props.
pub fn animate_story(
    time: Res<Time>,
    atlases: Res<Assets<SparrowAtlas>>,
    mut props: Query<(&mut StoryProp, &mut Sprite, &mut Transform), Without<StoryArrow>>,
    mut arrows: Query<(&mut StoryProp, &mut Sprite, &StoryArrow)>,
) {
    let frame_time = 1.0 / 24.0;

    for (mut prop, mut sprite, mut transform) in &mut props {
        if !prop.ready {
            let Some(atlas) = atlases.get(&prop.atlas) else {
                continue;
            };
            prop.frames = atlas.clip("idle");
            if prop.frames.is_empty() {
                prop.frames = atlas.clip("confirm");
            }
            if prop.frames.is_empty() {
                continue;
            }
            sprite.image = atlas.texture.clone();
            prop.ready = true;
        }
        prop.elapsed += time.delta_secs();
        while prop.elapsed >= frame_time {
            prop.elapsed -= frame_time;
            prop.index += 1;
        }
        let frame = &prop.frames[prop.index % prop.frames.len()];
        sprite.rect = Some(frame.rect);
        sprite.custom_size = Some(frame.size * prop.scale);
        transform.translation.x = prop.foot.x;
        transform.translation.y = prop.foot.y + frame.size.y * prop.scale * 0.5;
    }

    for (mut arrow, mut sprite, marker) in &mut arrows {
        if !arrow.ready {
            let Some(atlas) = atlases.get(&arrow.atlas) else {
                continue;
            };
            arrow.frames = atlas.clip(marker.0);
            if arrow.frames.is_empty() {
                continue;
            }
            sprite.image = atlas.texture.clone();
            arrow.ready = true;
        }
        if let Some(frame) = arrow.frames.first() {
            sprite.rect = Some(frame.rect);
            sprite.custom_size = Some(frame.size);
            sprite.color = Color::WHITE;
        }
    }
}

/// Keyboard/mouse/touch navigation for the week select.
pub fn story_input(
    keys: Res<ButtonInput<KeyCode>>,
    mouse: Res<ButtonInput<MouseButton>>,
    touches: Res<Touches>,
    mut selection: ResMut<StorySelection>,
    mut song_selection: ResMut<SongSelection>,
    mut next: ResMut<NextState<AppState>>,
) {
    if keys.just_pressed(KeyCode::Escape) {
        next.set(AppState::MainMenu);
        return;
    }

    if keys.just_pressed(KeyCode::ArrowRight) || keys.just_pressed(KeyCode::KeyD) {
        selection.week = (selection.week + 1) % STORY_WEEKS.len();
    }
    if keys.just_pressed(KeyCode::ArrowLeft) || keys.just_pressed(KeyCode::KeyA) {
        selection.week = selection
            .week
            .checked_sub(1)
            .unwrap_or(STORY_WEEKS.len() - 1);
    }
    if keys.just_pressed(KeyCode::ArrowUp) || keys.just_pressed(KeyCode::KeyW) {
        selection.difficulty = selection.difficulty.checked_sub(1).unwrap_or(2);
    }
    if keys.just_pressed(KeyCode::ArrowDown) || keys.just_pressed(KeyCode::KeyS) {
        selection.difficulty = (selection.difficulty + 1) % 3;
    }

    // Touch/mouse: tap = next week (a coarse navigation fallback).
    if touches.iter_just_pressed().next().is_some() || mouse.just_pressed(MouseButton::Left) {
        selection.week = (selection.week + 1) % STORY_WEEKS.len();
    }

    if keys.just_pressed(KeyCode::Enter) || keys.just_pressed(KeyCode::Space) {
        confirm(&selection, &mut song_selection, &mut next);
    }
}

/// Rebuild the title, difficulty chip and props when the week changes.
#[allow(clippy::type_complexity)]
pub fn refresh_story(
    selection: Res<StorySelection>,
    assets: Res<GameAssets>,
    asset_server: Res<AssetServer>,
    mut backdrops: Query<
        &mut Sprite,
        (
            With<StoryBackdrop>,
            Without<StoryTitle>,
            Without<StoryDifficultyChip>,
        ),
    >,
    mut titles: Query<
        &mut Sprite,
        (
            With<StoryTitle>,
            Without<StoryBackdrop>,
            Without<StoryDifficultyChip>,
        ),
    >,
    mut chips: Query<
        &mut Sprite,
        (
            With<StoryDifficultyChip>,
            Without<StoryTitle>,
            Without<StoryBackdrop>,
        ),
    >,
    mut props: Query<&mut StoryProp, Without<StoryArrow>>,
) {
    if !selection.is_changed() {
        return;
    }
    let week = &STORY_WEEKS[selection.week];

    for mut sprite in &mut backdrops {
        sprite.color = Color::srgb_u8(week.bg.0, week.bg.1, week.bg.2);
    }
    for mut sprite in &mut titles {
        sprite.image = assets.story_titles[week.title].clone();
    }
    for mut chip in &mut chips {
        chip.image = assets.story_difficulties[selection.difficulty].clone();
    }
    for mut prop in &mut props {
        if prop.slot >= 3 {
            continue;
        }
        let Some(stem) = week.props.get(prop.slot) else {
            continue;
        };
        prop.atlas = asset_server.load(format!("preload/images/storymenu/props/{stem}.xml"));
        prop.ready = false;
        prop.frames.clear();
        prop.index = 0;
    }
}

fn confirm(
    selection: &StorySelection,
    song_selection: &mut SongSelection,
    next: &mut NextState<AppState>,
) {
    let week = &STORY_WEEKS[selection.week];
    let index = BUILTIN_SONGS
        .iter()
        .position(|entry| entry.id == week.first_song)
        .unwrap_or(0);
    song_selection.song_index = index;
    song_selection.difficulty = Difficulty::ALL[selection.difficulty];
    next.set(AppState::Play);
}
