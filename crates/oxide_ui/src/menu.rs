//! Main menu: the original four Sparrow-atlas buttons stacked vertically over
//! the yellow ``menuBG`` backdrop.

use bevy::prelude::*;
use bevy::window::PrimaryWindow;
use oxide_core::config::{DESIGN_HEIGHT, DESIGN_WIDTH, screen_to_world};
use oxide_core::sparrow::{SparrowAtlas, SparrowFrame};
use oxide_core::theme::TEXT_DIM;
use oxide_core::{AppState, GameAssets, ScreenRoot};

/// Which menu item is currently highlighted.
#[derive(Resource)]
pub struct MenuSelection {
    pub index: usize,
}

/// Swallow the click that focused the window, so it cannot activate the
/// freshly spawned centre button on the frame MainMenu appears.
#[derive(Resource)]
pub struct MenuGate(Timer);

/// One hand-lettered menu button, animated between its ``idle`` and ``selected``
/// Sparrow clips.
#[derive(Component)]
pub struct MenuButtonVisual {
    atlas: Handle<SparrowAtlas>,
    center: Vec2,
    /// Roughly half the selected frame's size, used for mouse hit-testing.
    hitbox: Vec2,
    /// Which menu item this button activates (0..3).
    item: usize,
    idle: Vec<SparrowFrame>,
    selected: Vec<SparrowFrame>,
    frame: usize,
    elapsed: f32,
    ready: bool,
}

/// Index -> action. 0 storymode, 1 freeplay, 2 options, 3 credits.
const MENU_ITEMS: [(&str, &str); 4] = [
    ("storymode", "mainmenu/storymode"),
    ("freeplay", "mainmenu/freeplay"),
    ("options", "mainmenu/options"),
    ("credits", "mainmenu/credits"),
];

pub fn spawn_main_menu(mut commands: Commands, assets: Res<GameAssets>) {
    commands.insert_resource(MenuGate(Timer::from_seconds(0.25, TimerMode::Once)));
    commands.insert_resource(MenuSelection { index: 1 });

    // Backdrop, scaled 1.2x and centred like the original `setGraphicSize`.
    commands.spawn((
        Sprite {
            image: assets.menu_bg.clone(),
            custom_size: Some(Vec2::new(DESIGN_WIDTH * 1.2, DESIGN_HEIGHT * 1.2)),
            ..default()
        },
        Transform::from_xyz(0.0, 0.0, -30.0),
        ScreenRoot,
    ));

    // Four buttons, 160px apart vertically, centred horizontally.
    let spacing = 160.0;
    let top = (DESIGN_HEIGHT - spacing * (MENU_ITEMS.len() as f32 - 1.0)) * 0.5;
    let atlases = [
        assets.menu_storymode.clone(),
        assets.menu_freeplay.clone(),
        assets.menu_options.clone(),
        assets.menu_credits.clone(),
    ];
    for (index, atlas) in atlases.iter().enumerate() {
        let center = screen_to_world(DESIGN_WIDTH * 0.5, top + spacing * index as f32);
        commands.spawn((
            Sprite::default(),
            Transform::from_xyz(center.x, center.y, -20.0),
            MenuButtonVisual {
                atlas: atlas.clone(),
                center,
                hitbox: Vec2::new(420.0, 70.0),
                idle: Vec::new(),
                selected: Vec::new(),
                item: index,
                frame: 0,
                elapsed: 0.0,
                ready: false,
            },
            ScreenRoot,
        ));
    }

    // Version watermark, bottom-left, over a transparent UI layer.
    commands
        .spawn(crate::widgets::screen_bundle_clear())
        .with_children(|root| {
            root.spawn((
                Text::new(format!("v{}", env!("CARGO_PKG_VERSION"))),
                TextFont {
                    font: assets.font_ui.clone().into(),
                    font_size: FontSize::Px(18.0),
                    ..default()
                },
                TextColor(TEXT_DIM),
                Node {
                    position_type: PositionType::Absolute,
                    bottom: px(10.0),
                    left: px(12.0),
                    ..default()
                },
            ));
        });
}

/// Load clips once their atlases decode, then play the highlighted animation.
pub fn animate_menu_buttons(
    time: Res<Time>,
    selection: Res<MenuSelection>,
    atlases: Res<Assets<SparrowAtlas>>,
    mut buttons: Query<(&mut MenuButtonVisual, &mut Sprite, &mut Transform)>,
) {
    let frame_time = 1.0 / 24.0;
    for (mut button, mut sprite, mut transform) in &mut buttons {
        if !button.ready {
            let Some(atlas) = atlases.get(&button.atlas) else {
                continue;
            };
            let name = MENU_ITEMS[button.item].0;
            button.idle = atlas.clip(&format!("{name} idle"));
            button.selected = atlas.clip(&format!("{name} selected"));
            if button.idle.is_empty() && button.selected.is_empty() {
                continue;
            }
            sprite.image = atlas.texture.clone();
            button.ready = true;
        }
        let is_selected = selection.index == button.item;
        button.elapsed += time.delta_secs();
        while button.elapsed >= frame_time {
            button.elapsed -= frame_time;
            button.frame += 1;
        }
        let frames = if is_selected && !button.selected.is_empty() {
            &button.selected
        } else {
            &button.idle
        };
        if frames.is_empty() {
            continue;
        }
        let frame = &frames[button.frame % frames.len()];
        sprite.rect = Some(frame.rect);
        sprite.custom_size = Some(frame.size);
        transform.translation.x = button.center.x;
        transform.translation.y = button.center.y;
    }
}

/// Keyboard navigation plus mouse hover/click.
pub fn main_menu_input(
    keys: Res<ButtonInput<KeyCode>>,
    mouse: Res<ButtonInput<MouseButton>>,
    windows: Query<&Window, With<PrimaryWindow>>,
    buttons: Query<(&MenuButtonVisual, &Sprite)>,
    mut selection: ResMut<MenuSelection>,
    gate: Res<MenuGate>,
    mut next: ResMut<NextState<AppState>>,
) {
    if keys.just_pressed(KeyCode::ArrowUp) || keys.just_pressed(KeyCode::KeyW) {
        selection.index = selection.index.saturating_sub(1);
    }
    if keys.just_pressed(KeyCode::ArrowDown) || keys.just_pressed(KeyCode::KeyS) {
        selection.index = (selection.index + 1).min(MENU_ITEMS.len() - 1);
    }
    if keys.just_pressed(KeyCode::Escape) {
        next.set(AppState::Title);
        return;
    }

    let Ok(window) = windows.single() else {
        return;
    };

    // Mouse hover selects; a click activates the hovered item.
    if let Some(cursor) = window.cursor_position() {
        let world = cursor_to_world(window, cursor);
        for (button, sprite) in &buttons {
            let Some(size) = sprite.custom_size else {
                continue;
            };
            let half = size * 0.5;
            let in_x = (world.x - button.center.x).abs() <= half.x.max(button.hitbox.x);
            let in_y = (world.y - button.center.y).abs() <= half.y.max(button.hitbox.y);
            if in_x && in_y {
                if selection.index != button.item {
                    selection.index = button.item;
                }
                if mouse.just_pressed(MouseButton::Left) && gate.0.is_finished() {
                    activate(button.item, &mut next);
                    return;
                }
            }
        }
    }

    if gate.0.is_finished()
        && (keys.just_pressed(KeyCode::Enter)
            || keys.just_pressed(KeyCode::Space)
            || keys.just_pressed(KeyCode::KeyF))
    {
        activate(selection.index, &mut next);
    }
}

fn activate(index: usize, next: &mut NextState<AppState>) {
    match index {
        // Story mode opens the week select; Freeplay lists every song.
        0 => next.set(AppState::StoryMenu),
        1 => next.set(AppState::Freeplay),
        2 => next.set(AppState::Options),
        // Credits has no backing screen yet; stay put.
        _ => {}
    }
}

fn cursor_to_world(window: &Window, cursor: Vec2) -> Vec2 {
    let width = window.width().max(1.0);
    let height = window.height().max(1.0);
    let aspect = width / height;
    let view_height = DESIGN_HEIGHT;
    let view_width = view_height * aspect;
    let x = (cursor.x / width - 0.5) * view_width;
    let y = (0.5 - cursor.y / height) * view_height;
    Vec2::new(x, y)
}

pub fn tick_menu_gate(time: Res<Time>, mut gate: ResMut<MenuGate>) {
    gate.0.tick(time.delta());
}
