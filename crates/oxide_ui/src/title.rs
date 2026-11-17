//! Boot splash and title screen.
//!
//! The title screen is a faithful recreation of the original `TitleState`:
//! a pure-black backdrop, the animated ``logo bumpin`` Sparrow atlas, Girlfriend
//! dancing on the right and the blinking ``PRESS ENTER TO BEGIN`` text at the
//! bottom. Everything is a 2D sprite; there is no UI layer here.

use bevy::input::touch::Touches;
use bevy::prelude::*;
use oxide_core::config::screen_to_world;
use oxide_core::sparrow::{SparrowAtlas, SparrowFrame};
use oxide_core::theme::TEXT_DIM;
use oxide_core::{AppState, GameAssets, ScreenRoot};

use crate::widgets::{lime_text, screen_bundle};

/// The animated ``FUNKIN'`` logo at the top of the title screen.
#[derive(Component)]
pub struct LogoBump {
    atlas: Handle<SparrowAtlas>,
    /// World position of the sprite's top-left corner (so it never drifts while
    /// the individual bump frames change size).
    anchor: Vec2,
    frames: Vec<SparrowFrame>,
    index: usize,
    elapsed: f32,
    ready: bool,
}

/// Girlfriend's title-screen dance. The atlas has 30 frames; the original
/// alternates between the left-facing half (0..15) and right-facing half
/// (15..30) on each beat.
#[derive(Component)]
pub struct TitleDancer {
    atlas: Handle<SparrowAtlas>,
    center: Vec2,
    scale: f32,
    left: Vec<SparrowFrame>,
    right: Vec<SparrowFrame>,
    on_left: bool,
    index: usize,
    elapsed: f32,
    beat_timer: f32,
    ready: bool,
}

/// The ``PRESS ENTER TO BEGIN`` text, animated from the Animate atlas and
/// swapped to the confirm frame on press.
#[derive(Component)]
#[allow(dead_code)]
pub struct TitleText {
    atlas: Handle<SparrowAtlas>,
    idle: Vec<SparrowFrame>,
    confirm: Vec<SparrowFrame>,
    index: usize,
    elapsed: f32,
    confirming: bool,
    ready: bool,
}

/// Counts down the two-second flash-and-hold before entering the main menu.
#[derive(Resource)]
pub struct TitleAdvance {
    pub confirmed: bool,
    timer: Timer,
}

/// Roughly 102 BPM, matching the idle dance feel of the original.
const BEAT_PERIOD: f32 = 60.0 / 102.0;

pub fn spawn_boot(mut commands: Commands, assets: Res<GameAssets>) {
    commands
        .spawn(screen_bundle(assets.menu_bg.clone()))
        .with_children(|root| {
            root.spawn(lime_text("FNF: OXIDE ENGINE", &assets, 64.0));
            root.spawn(lime_text("LOADING ASSETS...", &assets, 26.0));
            root.spawn((
                Text::new(env!("CARGO_PKG_VERSION")),
                TextFont {
                    font: assets.font_ui.clone().into(),
                    font_size: FontSize::Px(18.0),
                    ..default()
                },
                TextColor(TEXT_DIM),
                Node {
                    margin: UiRect::top(px(28.0)),
                    ..default()
                },
            ));
        });
}

/// Leave Boot only once both fonts have been decoded, so the title never
/// flashes fallback text.
pub fn boot_tick(
    time: Res<Time>,
    fonts: Res<Assets<Font>>,
    atlases: Res<Assets<SparrowAtlas>>,
    assets: Res<GameAssets>,
    mut next: ResMut<NextState<AppState>>,
    mut waited: Local<f32>,
) {
    *waited += time.delta_secs();
    let fonts_ready =
        fonts.get(&assets.font_ui).is_some() && fonts.get(&assets.font_score).is_some();
    // `title-screen-text`, the logo and the dancer are baked/decode by loaders;
    // hold the splash until they are resident so the title never pops in late.
    let art_ready = atlases.get(&assets.title_text).is_some()
        && atlases.get(&assets.logo_bumpin).is_some()
        && atlases.get(&assets.gf_atlas).is_some();
    if (fonts_ready && art_ready) || *waited > 20.0 {
        next.set(AppState::Title);
    }
}

pub fn spawn_title(mut commands: Commands, assets: Res<GameAssets>) {
    // Pure black backdrop, exactly like the original `TitleState`.
    commands.spawn((
        Sprite::from_color(Color::BLACK, Vec2::new(1280.0, 720.0)),
        Transform::from_xyz(0.0, 0.0, -30.0),
        ScreenRoot,
    ));

    // Logo. The original places its top-left at (-150, -100).
    let logo_anchor = screen_to_world(-150.0, -100.0);
    commands.spawn((
        Sprite::default(),
        Transform::from_xyz(logo_anchor.x, logo_anchor.y, -20.0),
        LogoBump {
            atlas: assets.logo_bumpin.clone(),
            anchor: logo_anchor,
            frames: Vec::new(),
            index: 0,
            elapsed: 0.0,
            ready: false,
        },
        ScreenRoot,
    ));

    // Girlfriend. The original places her top-left at (512, 50).
    let gf_center = screen_to_world(512.0 + 721.0 * 0.5, 50.0 + 662.0 * 0.5);
    commands.spawn((
        Sprite::default(),
        Transform::from_xyz(gf_center.x, gf_center.y, -19.0),
        TitleDancer {
            atlas: assets.gf_atlas.clone(),
            center: gf_center,
            scale: 1.0,
            left: Vec::new(),
            right: Vec::new(),
            on_left: true,
            index: 0,
            elapsed: 0.0,
            beat_timer: 0.0,
            ready: false,
        },
        ScreenRoot,
    ));

    // Prompt sprite, straight from the `title-screen-text` Animate atlas.
    // Original placement: top-left (100, 576) with a ~1004x80 frame.
    let prompt_center = screen_to_world(100.0 + 1004.0 * 0.5, 576.0 + 80.0 * 0.5);
    commands.spawn((
        Sprite::default(),
        Transform::from_xyz(prompt_center.x, prompt_center.y, -18.0),
        TitleText {
            atlas: assets.title_text.clone(),
            idle: Vec::new(),
            confirm: Vec::new(),
            index: 0,
            elapsed: 0.0,
            confirming: false,
            ready: false,
        },
        ScreenRoot,
    ));

    commands.insert_resource(TitleAdvance {
        confirmed: false,
        timer: Timer::from_seconds(2.0, TimerMode::Once),
    });
}

/// Advance the logo and Girlfriend once their atlases have decoded.
#[allow(clippy::type_complexity)]
pub fn animate_title(
    time: Res<Time>,
    atlases: Res<Assets<SparrowAtlas>>,
    mut logo_query: Query<(&mut LogoBump, &mut Sprite, &mut Transform), Without<TitleDancer>>,
    mut dancer_query: Query<(&mut TitleDancer, &mut Sprite, &mut Transform), Without<LogoBump>>,
    mut text_query: Query<(&mut TitleText, &mut Sprite), (Without<TitleDancer>, Without<LogoBump>)>,
) {
    let frame_time = 1.0 / 24.0;

    for (mut logo, mut sprite, mut transform) in &mut logo_query {
        if !logo.ready {
            let Some(atlas) = atlases.get(&logo.atlas) else {
                continue;
            };
            logo.frames = atlas.clip("logo bumpin");
            if logo.frames.is_empty() {
                continue;
            }
            sprite.image = atlas.texture.clone();
            logo.ready = true;
        }
        if logo.frames.is_empty() {
            continue;
        }
        logo.elapsed += time.delta_secs();
        while logo.elapsed >= frame_time {
            logo.elapsed -= frame_time;
            logo.index = (logo.index + 1) % logo.frames.len();
        }
        let frame = &logo.frames[logo.index];
        sprite.rect = Some(frame.rect);
        sprite.custom_size = Some(frame.size);
        // Keep the sprite's top-left pinned, so the bump animation never drifts.
        transform.translation.x = logo.anchor.x + frame.source_size.x * 0.5;
        transform.translation.y = logo.anchor.y - frame.source_size.y * 0.5;
    }

    for (mut dancer, mut sprite, mut transform) in &mut dancer_query {
        if !dancer.ready {
            let Some(atlas) = atlases.get(&dancer.atlas) else {
                continue;
            };
            let all = atlas.clip("gfDance");
            dancer.left = all[..15.min(all.len())].to_vec();
            dancer.right = all[15.min(all.len())..].to_vec();
            if dancer.left.is_empty() && dancer.right.is_empty() {
                continue;
            }
            sprite.image = atlas.texture.clone();
            dancer.ready = true;
        }
        let frames = if dancer.on_left {
            &dancer.left
        } else {
            &dancer.right
        };
        if frames.is_empty() {
            continue;
        }
        // Advance timing and animation state before borrowing the clip, so the
        // borrow checker can keep the frame slice alive until use.
        dancer.elapsed += time.delta_secs();
        dancer.beat_timer += time.delta_secs();
        if dancer.beat_timer >= BEAT_PERIOD {
            dancer.beat_timer -= BEAT_PERIOD;
            dancer.on_left = !dancer.on_left;
            dancer.index = 0;
        }
        while dancer.elapsed >= frame_time {
            dancer.elapsed -= frame_time;
            dancer.index += 1;
        }
        let frames = if dancer.on_left {
            &dancer.left
        } else {
            &dancer.right
        };
        let frame = &frames[dancer.index % frames.len()];
        sprite.rect = Some(frame.rect);
        sprite.custom_size = Some(frame.size * dancer.scale);
        transform.translation.x = dancer.center.x;
        transform.translation.y = dancer.center.y;
    }

    for (mut text, mut sprite) in &mut text_query {
        if !text.ready {
            let Some(atlas) = atlases.get(&text.atlas) else {
                continue;
            };
            text.idle = atlas.clip("Idle");
            text.confirm = atlas.clip("Confirm");
            if text.idle.is_empty() && text.confirm.is_empty() {
                continue;
            }
            sprite.image = atlas.texture.clone();
            text.index = 2;
            text.ready = true;
        }
        let confirming = text.confirming;
        let count = if confirming {
            text.confirm.len()
        } else {
            text.idle.len()
        };
        if count == 0 {
            continue;
        }
        text.elapsed += time.delta_secs();
        while text.elapsed >= frame_time {
            text.elapsed -= frame_time;
            text.index += 1;
        }
        let index = text.index % count;
        let frame = if confirming {
            text.confirm[index].clone()
        } else {
            text.idle[index].clone()
        };
        sprite.rect = Some(frame.rect);
        sprite.custom_size = Some(frame.size);
        sprite.color = Color::WHITE;
    }
}

/// Swap the prompt to the confirm frame and, after a short hold, enter the
/// main menu.
pub fn title_advance(
    time: Res<Time>,
    mut advance: ResMut<TitleAdvance>,
    mut prompts: Query<&mut TitleText>,
    mut next: ResMut<NextState<AppState>>,
) {
    if advance.confirmed {
        for mut title_text in &mut prompts {
            title_text.confirming = true;
        }
        advance.timer.tick(time.delta());
        if advance.timer.just_finished() {
            next.set(AppState::MainMenu);
        }
    }
}

pub fn title_input(
    time: Res<Time>,
    keys: Res<ButtonInput<KeyCode>>,
    mouse: Res<ButtonInput<MouseButton>>,
    touches: Res<Touches>,
    mut advance: ResMut<TitleAdvance>,
) {
    // Ignore the click that may have focused the window during startup, and do
    // not auto-advance while a development screenshot is being captured.
    if time.elapsed_secs() < 2.0 || std::env::var("FNF_OXIDE_SCREENSHOT").is_ok() {
        return;
    }
    if advance.confirmed {
        return;
    }
    let touched = touches.iter_just_pressed().next().is_some();
    if keys.just_pressed(KeyCode::Enter)
        || keys.just_pressed(KeyCode::Space)
        || mouse.just_pressed(MouseButton::Left)
        || touched
    {
        advance.confirmed = true;
    }
}
