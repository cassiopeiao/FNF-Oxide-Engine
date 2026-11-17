//! Freeplay song browser, laid out like the original `FreeplayState`.
//!
//! The original screen is a yellow "backing card" with the week's characters on
//! the right, a difficulty selector at the top left, a vertical list of song
//! capsules and a high-score / album-cover block on the right.

use bevy::picking::hover::Hovered;
use bevy::prelude::*;
use bevy::ui::Pressed;
use bevy::ui_widgets::{Activate, ActivateOnPress, Button, observe};
use oxide_core::{AppState, GameAssets, ScreenRoot};
use oxide_data::BUILTIN_SONGS;
use oxide_game::SongSelection;

const BG_YELLOW: Color = Color::srgb(0.976, 0.812, 0.32);
const CAPSULE_BG: Color = Color::srgb(0.07, 0.07, 0.12);
const CAPSULE_BG_SELECTED: Color = Color::srgb(0.16, 0.16, 0.24);
const CAPSULE_BORDER: Color = Color::srgba(0.55, 0.5, 0.42, 0.55);
const CAPSULE_BORDER_SELECTED: Color = Color::srgb(1.0, 0.86, 0.25);
const TEXT_WHITE: Color = Color::srgb(0.97, 0.97, 0.95);
const TEXT_GREY: Color = Color::srgb(0.45, 0.45, 0.45);
const TEXT_SUB: Color = Color::srgb(0.62, 0.62, 0.58);
const HIGHSCORE_ORANGE: Color = Color::srgb(1.0, 0.55, 0.1);
const HIGHSCORE_CYAN: Color = Color::srgb(0.55, 0.95, 1.0);

/// Which entry the cursor is on. `0` is the "Random" capsule, `1..` maps to
/// `BUILTIN_SONGS[index - 1]`.
#[derive(Resource, Debug)]
pub struct FreeplayCursor {
    pub index: usize,
    pub seed: u32,
}

impl Default for FreeplayCursor {
    fn default() -> Self {
        Self {
            index: 0,
            seed: 0x9E37_79B9,
        }
    }
}

impl FreeplayCursor {
    fn song_count() -> usize {
        BUILTIN_SONGS.len()
    }

    fn item_count() -> usize {
        Self::song_count() + 1
    }

    /// Return a pseudo-random song index (0-based).
    fn random_song(&mut self) -> usize {
        self.seed = self
            .seed
            .wrapping_mul(1_664_525)
            .wrapping_add(1_013_904_223);
        (self.seed as usize) % Self::song_count()
    }
}

/// One row in the scrolling capsule list. `index == 0` is the Random capsule.
#[derive(Component)]
pub struct Capsule {
    pub index: usize,
}

/// Marker on the inner container that scrolls with the cursor.
#[derive(Component)]
pub struct CapsuleList;

/// Marker on the difficulty word image that follows `SongSelection`.
#[derive(Component)]
pub struct DifficultyImage;

/// One of the two little difficulty arrows.
#[derive(Component)]
pub struct DiffArrow {
    pub delta: i32,
}

const LIST_LEFT: f32 = 392.0;
const LIST_TOP: f32 = 104.0;
const LIST_WIDTH: f32 = 476.0;
const LIST_HEIGHT: f32 = 548.0;
const ITEM_HEIGHT: f32 = 58.0;
const ITEM_PITCH: f32 = 70.0;
const SELECTED_ROW_TOP: f32 = 8.0;

pub fn spawn_freeplay(mut commands: Commands, assets: Res<GameAssets>) {
    let text_font = assets.font_ui.clone();
    let score_font = assets.font_score.clone();

    commands
        .spawn((
            Node {
                width: percent(100.0),
                height: percent(100.0),
                position_type: PositionType::Relative,
                overflow: Overflow::clip(),
                ..default()
            },
            BackgroundColor(BG_YELLOW),
            ScreenRoot,
        ))
        .with_children(|root| {
            // Characters on the right: the original backing image is Dad (and
            // GF) on a purple stage.
            root.spawn((
                ImageNode {
                    image: assets.freeplay_bg.clone(),
                    image_mode: NodeImageMode::Stretch,
                    ..default()
                },
                Node {
                    position_type: PositionType::Absolute,
                    right: px(0.0),
                    top: px(0.0),
                    width: px(580.0),
                    height: percent(100.0),
                    ..default()
                },
            ));

            // Yellow backing card behind the song list.
            root.spawn((
                Node {
                    position_type: PositionType::Absolute,
                    left: px(356.0),
                    top: px(92.0),
                    width: px(536.0),
                    height: px(568.0),
                    border: UiRect::all(px(3.0)),
                    border_radius: BorderRadius::all(px(20.0)),
                    ..default()
                },
                BackgroundColor(Color::srgb(0.988, 0.847, 0.392)),
                BorderColor::all(Color::srgba(0.55, 0.42, 0.10, 0.35)),
            ));

            // Header row.
            root.spawn((
                Text::new("FREEPLAY"),
                TextFont {
                    font: text_font.clone().into(),
                    font_size: FontSize::Px(34.0),
                    ..default()
                },
                TextColor(TEXT_WHITE),
                Node {
                    position_type: PositionType::Absolute,
                    left: px(18.0),
                    top: px(44.0),
                    ..default()
                },
            ));
            root.spawn((
                Text::new("PRESS [TAB] TO CHANGE CHARACTERS"),
                TextFont {
                    font: text_font.clone().into(),
                    font_size: FontSize::Px(15.0),
                    ..default()
                },
                TextColor(TEXT_GREY),
                TextLayout { justify: Justify::Center, ..default() },
                Node {
                    position_type: PositionType::Absolute,
                    left: px(0.0),
                    width: percent(100.0),
                    top: px(16.0),
                    ..default()
                },
            ));
            root.spawn((
                Text::new("OFFICIAL OST"),
                TextFont {
                    font: text_font.clone().into(),
                    font_size: FontSize::Px(26.0),
                    ..default()
                },
                TextColor(TEXT_WHITE),
                Node {
                    position_type: PositionType::Absolute,
                    right: px(18.0),
                    top: px(6.0),
                    ..default()
                },
            ));

            // Difficulty selector: ◀ NORM ▶.
            root.spawn((
                ImageNode {
                    image: assets.fp_arrows.clone(),
                    rect: Some(Rect::new(4.0, 4.0, 52.0, 89.0)),
                    ..default()
                },
                Node {
                    position_type: PositionType::Absolute,
                    left: px(22.0),
                    top: px(98.0),
                    width: px(40.0),
                    height: px(70.0),
                    ..default()
                },
                Button,
                DiffArrow { delta: -1 },
                ActivateOnPress,
                Hovered::default(),
                observe(
                    |_: On<Activate>, mut selection: ResMut<SongSelection>| {
                        selection.difficulty = selection.difficulty.cycle(-1);
                    },
                ),
            ));
            root.spawn((
                ImageNode {
                    image: assets.fp_difficulties[1].clone(),
                    image_mode: NodeImageMode::Auto,
                    ..default()
                },
                Node {
                    position_type: PositionType::Absolute,
                    left: px(74.0),
                    top: px(96.0),
                    width: px(196.0),
                    height: px(66.0),
                    ..default()
                },
                DifficultyImage,
            ));
            root.spawn((
                ImageNode {
                    image: assets.fp_arrows.clone(),
                    rect: Some(Rect::new(56.0, 4.0, 103.0, 89.0)),
                    ..default()
                },
                Node {
                    position_type: PositionType::Absolute,
                    left: px(280.0),
                    top: px(98.0),
                    width: px(40.0),
                    height: px(70.0),
                    ..default()
                },
                Button,
                DiffArrow { delta: 1 },
                ActivateOnPress,
                Hovered::default(),
                observe(
                    |_: On<Activate>, mut selection: ResMut<SongSelection>| {
                        selection.difficulty = selection.difficulty.cycle(1);
                    },
                ),
            ));

            // Scrolling capsule list.
            root.spawn((
                Node {
                    position_type: PositionType::Absolute,
                    left: px(LIST_LEFT),
                    top: px(LIST_TOP),
                    width: px(LIST_WIDTH),
                    height: px(LIST_HEIGHT),
                    overflow: Overflow::clip(),
                    ..default()
                },
            ))
            .with_children(|clip| {
                clip.spawn((
                    Node {
                        position_type: PositionType::Absolute,
                        left: px(0.0),
                        top: px(SELECTED_ROW_TOP),
                        width: percent(100.0),
                        flex_direction: FlexDirection::Column,
                        row_gap: px(ITEM_PITCH - ITEM_HEIGHT),
                        ..default()
                    },
                    CapsuleList,
                ))
                .with_children(|list| {
                    for index in 0..FreeplayCursor::item_count() {
                        spawn_capsule(list, &assets, index);
                    }
                });
            });

            // High score block.
            root.spawn((
                Text::new("HIGHSCORE"),
                TextFont {
                    font: score_font.clone().into(),
                    font_size: FontSize::Px(30.0),
                    ..default()
                },
                TextColor(HIGHSCORE_ORANGE),
                Node {
                    position_type: PositionType::Absolute,
                    left: px(884.0),
                    top: px(62.0),
                    ..default()
                },
            ));
            root.spawn((
                Text::new("00000000"),
                TextFont {
                    font: score_font.clone().into(),
                    font_size: FontSize::Px(46.0),
                    ..default()
                },
                TextColor(HIGHSCORE_CYAN),
                Node {
                    position_type: PositionType::Absolute,
                    left: px(884.0),
                    top: px(100.0),
                    ..default()
                },
            ));
            root.spawn((
                Text::new("CLEARED 0%"),
                TextFont {
                    font: text_font.clone().into(),
                    font_size: FontSize::Px(13.0),
                    ..default()
                },
                TextColor(TEXT_WHITE),
                Node {
                    position_type: PositionType::Absolute,
                    right: px(20.0),
                    top: px(70.0),
                    ..default()
                },
            ));

            // Album cover.
            root.spawn((
                ImageNode {
                    image: assets.fp_album.clone(),
                    image_mode: NodeImageMode::Auto,
                    ..default()
                },
                Node {
                    position_type: PositionType::Absolute,
                    left: px(948.0),
                    top: px(250.0),
                    width: px(270.0),
                    height: px(270.0),
                    ..default()
                },
            ));
            root.spawn((
                ImageNode {
                    image: assets.fp_album_text.clone(),
                    image_mode: NodeImageMode::Auto,
                    rect: Some(Rect::new(0.0, 0.0, 234.0, 133.0)),
                    ..default()
                },
                Node {
                    position_type: PositionType::Absolute,
                    left: px(952.0),
                    top: px(492.0),
                    width: px(250.0),
                    height: px(142.0),
                    ..default()
                },
            ));

            // Footer hint.
            root.spawn((
                Text::new("UP/DOWN  SELECT   |   LEFT/RIGHT  DIFFICULTY   |   ENTER  PLAY   |   ESC  BACK"),
                TextFont {
                    font: text_font.clone().into(),
                    font_size: FontSize::Px(13.0),
                    ..default()
                },
                TextColor(TEXT_GREY),
                Node {
                    position_type: PositionType::Absolute,
                    left: px(0.0),
                    right: px(0.0),
                    bottom: px(8.0),
                    justify_content: JustifyContent::Center,
                    ..default()
                },
            ));
        });
}

fn spawn_capsule(parent: &mut ChildSpawnerCommands, assets: &GameAssets, index: usize) {
    let entry = if index == 0 {
        None
    } else {
        BUILTIN_SONGS.get(index - 1)
    };

    let icon = match entry {
        Some(entry) if entry.id == "tutorial" => assets.icon_gf.clone(),
        Some(_) => assets.icon_dad.clone(),
        None => assets.icon_bf.clone(),
    };

    let title = entry
        .map(|e| e.song_name.to_uppercase())
        .unwrap_or_else(|| "RANDOM".into());
    let number = if index == 0 {
        String::new()
    } else {
        format!("{:02}", index)
    };
    let sub = entry
        .map(|e| {
            format!(
                "BPM {:.0}   ...   {} DIFFICULTY",
                e.bpm,
                e.stage.to_uppercase()
            )
        })
        .unwrap_or_else(|| "SURPRISE ME".into());

    parent
        .spawn((
            Button,
            Capsule { index },
            ActivateOnPress,
            Hovered::default(),
            Node {
                width: percent(100.0),
                height: px(ITEM_HEIGHT),
                align_items: AlignItems::Center,
                padding: UiRect::horizontal(px(10.0)),
                border: UiRect::all(px(2.0)),
                border_radius: BorderRadius::all(px(12.0)),
                ..default()
            },
            BackgroundColor(CAPSULE_BG),
            BorderColor::all(CAPSULE_BORDER),
            observe(
                move |_: On<Activate>,
                      mut cursor: ResMut<FreeplayCursor>,
                      mut next: ResMut<NextState<AppState>>,
                      mut selection: ResMut<SongSelection>| {
                    let was_selected = cursor.index == index;
                    cursor.index = index;
                    if was_selected {
                        selection.song_index = if index == 0 {
                            cursor.random_song()
                        } else {
                            index - 1
                        };
                        next.set(AppState::Play);
                    }
                },
            ),
        ))
        .with_children(|item| {
            item.spawn((
                ImageNode {
                    image: icon,
                    image_mode: NodeImageMode::Auto,
                    ..default()
                },
                Node {
                    width: px(44.0),
                    height: px(44.0),
                    margin: UiRect::right(px(10.0)),
                    ..default()
                },
            ));
            item.spawn((Node {
                flex_direction: FlexDirection::Column,
                flex_grow: 1.0,
                ..default()
            },))
                .with_children(|col| {
                    col.spawn((
                        Text::new(title),
                        TextFont {
                            font: assets.font_ui.clone().into(),
                            font_size: FontSize::Px(24.0),
                            ..default()
                        },
                        TextColor(TEXT_WHITE),
                    ));
                    col.spawn((
                        Text::new(sub),
                        TextFont {
                            font: assets.font_ui.clone().into(),
                            font_size: FontSize::Px(11.0),
                            ..default()
                        },
                        TextColor(TEXT_SUB),
                    ));
                });
            item.spawn((
                Text::new(number),
                TextFont {
                    font: assets.font_score.clone().into(),
                    font_size: FontSize::Px(30.0),
                    ..default()
                },
                TextColor(TEXT_SUB),
            ));
        });
}

/// Arrow keys / WASD move the cursor; left/right or Q/E change difficulty.
pub fn freeplay_input(
    keys: Res<ButtonInput<KeyCode>>,
    mut cursor: ResMut<FreeplayCursor>,
    mut selection: ResMut<SongSelection>,
    mut next: ResMut<NextState<AppState>>,
) {
    let count = FreeplayCursor::item_count();
    let down = keys.any_just_pressed([KeyCode::ArrowDown, KeyCode::KeyS]);
    let up = keys.any_just_pressed([KeyCode::ArrowUp, KeyCode::KeyW]);
    if down {
        cursor.index = (cursor.index + 1) % count;
    }
    if up {
        cursor.index = (cursor.index + count - 1) % count;
    }

    if keys.any_just_pressed([KeyCode::ArrowLeft, KeyCode::KeyA, KeyCode::KeyQ]) {
        selection.difficulty = selection.difficulty.cycle(-1);
    }
    if keys.any_just_pressed([KeyCode::ArrowRight, KeyCode::KeyD, KeyCode::KeyE]) {
        selection.difficulty = selection.difficulty.cycle(1);
    }

    if keys.any_just_pressed([KeyCode::Enter, KeyCode::Space]) {
        if cursor.index == 0 {
            selection.song_index = cursor.random_song();
        } else {
            selection.song_index = cursor.index - 1;
        }
        next.set(AppState::Play);
    }
    if keys.just_pressed(KeyCode::Escape) {
        next.set(AppState::MainMenu);
    }
}

/// Slide the capsule list so the selected row sits at a fixed screen height.
pub fn scroll_capsules(
    cursor: Res<FreeplayCursor>,
    mut lists: Query<&mut Node, With<CapsuleList>>,
) {
    let target = px(SELECTED_ROW_TOP - cursor.index as f32 * ITEM_PITCH);
    for mut node in &mut lists {
        if node.top != target {
            node.top = target;
        }
    }
}

/// Highlight the selected capsule and dim the rest.
#[allow(clippy::type_complexity)]
pub fn style_capsules(
    cursor: Res<FreeplayCursor>,
    mut capsules: Query<(
        &Capsule,
        &Hovered,
        Has<Pressed>,
        &mut BackgroundColor,
        &mut BorderColor,
    )>,
) {
    for (capsule, hovered, pressed, mut background, mut border) in &mut capsules {
        let selected = capsule.index == cursor.index;
        background.0 = if selected {
            CAPSULE_BG_SELECTED
        } else if hovered.get() || pressed {
            Color::srgba(0.12, 0.12, 0.18, 0.94)
        } else {
            CAPSULE_BG
        };
        *border = BorderColor::all(if selected {
            CAPSULE_BORDER_SELECTED
        } else {
            CAPSULE_BORDER
        });
    }
}

/// Swap the difficulty word image when the difficulty changes.
pub fn refresh_difficulty(
    selection: Res<SongSelection>,
    assets: Res<GameAssets>,
    mut image: Query<&mut ImageNode, With<DifficultyImage>>,
) {
    if !selection.is_changed() {
        return;
    }
    let idx = selection.difficulty as usize;
    for mut node in &mut image {
        if let Some(handle) = assets.fp_difficulties.get(idx) {
            node.image = handle.clone();
        }
    }
}
