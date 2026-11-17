//! In-game HUD: the original ``healthBar`` sprite with sliding icons, plus the
//! score / combo / judgement / countdown text and pause overlay.

use bevy::prelude::*;
use oxide_core::config::screen_to_world;
use oxide_core::theme::{TEXT_DIM, TEXT_PRIMARY, judgement_color};
use oxide_core::{DESIGN_HEIGHT, GameAssets};
use oxide_data::Difficulty;

use crate::play::PlayObject;
use crate::{PlayPhase, PlaySession};

#[derive(Component)]
pub struct HudRoot;

#[derive(Component)]
pub struct ScoreText;

#[derive(Component)]
pub struct ComboText;

#[derive(Component)]
pub struct JudgementText;

#[derive(Component)]
pub struct CountdownText;

/// The green fill inside the health bar (a coloured sprite whose width tracks
/// `health`).
#[derive(Component)]
pub struct HealthFill;

/// A character icon sitting at one end of the health bar.
#[derive(Component)]
pub struct HealthIcon {
    pub player: bool,
}

#[derive(Component)]
pub struct PauseOverlay;

fn hud_font(assets: &GameAssets, size: f32) -> TextFont {
    TextFont {
        font: assets.font_ui.clone().into(),
        font_size: FontSize::Px(size),
        ..default()
    }
}

fn text(text: impl Into<String>, assets: &GameAssets, size: f32, color: Color) -> impl Bundle {
    (Text::new(text), hud_font(assets, size), TextColor(color))
}

fn centered(top: Val) -> Node {
    Node {
        position_type: PositionType::Absolute,
        top,
        left: px(0.0),
        width: percent(100.0),
        justify_content: JustifyContent::Center,
        ..default()
    }
}

/// The health bar and icons are 2D sprites (so they can slide and bob), drawn
/// at the bottom of the screen exactly like the original `camHUD`.
fn health_bar_y() -> f32 {
    screen_to_world(0.0, DESIGN_HEIGHT * 0.9 + 9.5).y
}

pub fn spawn_hud(
    commands: &mut Commands,
    assets: &GameAssets,
    song_name: &str,
    difficulty: Difficulty,
) {
    // ---- 2D health bar -------------------------------------------------
    let bar_center = Vec2::new(0.0, health_bar_y());
    commands.spawn((
        Sprite {
            image: assets.health_bar.clone(),
            custom_size: Some(Vec2::new(601.0, 19.0)),
            ..default()
        },
        Transform::from_xyz(bar_center.x, bar_center.y, 12.0),
        PlayObject,
    ));

    // Green fill, 4px inset, anchored to the left of the bar.
    let fill_left = screen_to_world(640.0 - 300.5 + 4.0, 0.0).x;
    commands.spawn((
        Sprite::from_color(Color::srgb(0.31, 1.0, 0.25), Vec2::new(593.0, 11.0)),
        Transform::from_xyz(fill_left + 296.5, bar_center.y, 12.1),
        HealthFill,
        PlayObject,
    ));

    // Dad icon on the left, Boyfriend on the right, both bobbing on the beat.
    let icon_y = screen_to_world(0.0, DESIGN_HEIGHT * 0.9 - 75.0).y;
    for (player, x, image) in [
        (false, 340.0_f32, assets.icon_dad.clone()),
        (true, 940.0_f32, assets.icon_bf.clone()),
    ] {
        let center = screen_to_world(x, 0.0);
        commands.spawn((
            Sprite {
                image,
                custom_size: Some(Vec2::splat(150.0)),
                ..default()
            },
            Transform::from_xyz(center.x, icon_y, 12.2),
            HealthIcon { player },
            PlayObject,
        ));
    }

    // ---- UI text -------------------------------------------------------
    let title = format!(
        "{}   |   {}",
        song_name.to_uppercase(),
        difficulty.display_name()
    );
    let mut root = commands.spawn((
        Node {
            width: percent(100.0),
            height: percent(100.0),
            ..default()
        },
        HudRoot,
        PlayObject,
    ));
    root.with_children(|root| {
        // Song title, top center.
        root.spawn(centered(px(12.0))).with_children(|parent| {
            parent.spawn(text(title, assets, 24.0, TEXT_DIM));
        });

        // Score, below the health bar, right-aligned like the original.
        root.spawn((
            Node {
                position_type: PositionType::Absolute,
                top: px(DESIGN_HEIGHT * 0.9 + 30.0),
                right: px(300.0),
                ..default()
            },
            children![(text("Score: 0", assets, 22.0, TEXT_PRIMARY), ScoreText)],
        ));

        // Combo, center screen.
        root.spawn(centered(px(DESIGN_HEIGHT - 190.0)))
            .with_children(|parent| {
                parent.spawn((text("", assets, 44.0, TEXT_PRIMARY), ComboText));
            });

        // Judgement popup, above the combo.
        root.spawn(centered(percent(38.0))).with_children(|parent| {
            parent.spawn((text("", assets, 52.0, TEXT_PRIMARY), JudgementText));
        });

        // Countdown / loading text.
        root.spawn(centered(percent(24.0))).with_children(|parent| {
            parent.spawn((text("", assets, 92.0, TEXT_PRIMARY), CountdownText));
        });

        // Pause hint.
        root.spawn((
            Node {
                position_type: PositionType::Absolute,
                bottom: px(56.0),
                right: px(24.0),
                ..default()
            },
            children![text("ESC / P  PAUSE", assets, 18.0, TEXT_DIM)],
        ));

        // Pause overlay.
        let mut overlay = root.spawn((
            Node {
                position_type: PositionType::Absolute,
                width: percent(100.0),
                height: percent(100.0),
                flex_direction: FlexDirection::Column,
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                row_gap: px(18.0),
                display: Display::None,
                ..default()
            },
            BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.72)),
            PauseOverlay,
        ));
        overlay.with_children(|parent| {
            parent.spawn(text("PAUSED", assets, 84.0, TEXT_PRIMARY));
            parent.spawn(text("ESC / P  RESUME", assets, 26.0, TEXT_DIM));
            parent.spawn(text("Q  QUIT TO FREEPLAY", assets, 26.0, TEXT_DIM));
        });
    });
}

#[allow(clippy::type_complexity)]
pub fn update_hud(
    session: Res<PlaySession>,
    mut texts: ParamSet<(
        Query<&mut Text, With<ScoreText>>,
        Query<&mut Text, With<ComboText>>,
        Query<(&mut Text, &mut TextColor), With<JudgementText>>,
        Query<&mut Text, With<CountdownText>>,
    )>,
    mut health: Query<(&mut Sprite, &mut Transform), With<HealthFill>>,
    mut overlay: Query<&mut Node, (With<PauseOverlay>, Without<HealthFill>)>,
) {
    if let Ok(mut text) = texts.p0().single_mut() {
        text.0 = format!("Score: {}", session.score);
    }
    if let Ok(mut text) = texts.p1().single_mut() {
        text.0 = if session.combo >= 3 {
            format!("{} COMBO", session.combo)
        } else {
            String::new()
        };
    }
    if let Ok((mut text, mut color)) = texts.p2().single_mut() {
        match session.popup {
            Some((judgement, remaining)) if remaining > 0.0 => {
                text.0 = judgement.label().to_owned();
                color.0 = judgement_color(judgement.label());
            }
            _ => {
                text.0.clear();
            }
        }
    }
    if let Ok(mut text) = texts.p3().single_mut() {
        text.0 = match session.phase {
            PlayPhase::Loading => "LOADING...".to_owned(),
            PlayPhase::Countdown => {
                let remaining = (-session.conductor.position_ms() / 1000.0).ceil() as i64;
                if remaining > 0 {
                    remaining.to_string()
                } else {
                    "GO!".to_owned()
                }
            }
            PlayPhase::Playing if session.conductor.position_ms() < 420.0 => "GO!".to_owned(),
            _ => String::new(),
        };
    }
    if let Ok((mut fill, mut transform)) = health.single_mut() {
        let ratio = session.health.clamp(0.0, 100.0) / 100.0;
        let width = 593.0 * ratio;
        fill.custom_size = Some(Vec2::new(width.max(0.0), 11.0));
        // Green when healthy, red when nearly empty (like the original flash).
        fill.color = if ratio < 0.2 {
            Color::srgb(1.0, 0.25, 0.25)
        } else {
            Color::srgb(0.31, 1.0, 0.25)
        };
        let left = screen_to_world(640.0 - 300.5 + 4.0, 0.0).x;
        transform.translation.x = left + width * 0.5;
    }
    if let Ok(mut node) = overlay.single_mut() {
        node.display = if session.phase == PlayPhase::Paused {
            Display::Flex
        } else {
            Display::None
        };
    }
}

/// Bob the character icons once per beat, like `HealthIcon.onStepHit`.
pub fn bob_health_icons(
    session: Res<PlaySession>,
    mut icons: Query<(&HealthIcon, &mut Transform)>,
) {
    let beat = session.conductor.beat();
    let pulse = (beat * std::f64::consts::PI).sin().abs() as f32;
    let scale = 1.0 + pulse * 0.18;
    for (_icon, mut transform) in &mut icons {
        transform.scale = Vec3::splat(scale);
    }
}
