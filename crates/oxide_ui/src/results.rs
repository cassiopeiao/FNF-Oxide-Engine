//! Results screen shown after a song ends or the player fails out.

use bevy::prelude::*;
use bevy::ui_widgets::{Activate, observe};
use oxide_core::theme::{ACCENT, PANEL, PANEL_BORDER, TEXT_DIM, TEXT_PRIMARY, rank_color};
use oxide_core::{AppState, GameAssets};
use oxide_game::LastResult;

use crate::widgets::{lime_text, menu_button, screen_bundle};

pub fn spawn_results(mut commands: Commands, assets: Res<GameAssets>, result: Res<LastResult>) {
    let rank_color = rank_color(result.rank);
    let failed = result.failed;

    commands
        .spawn(screen_bundle(assets.menu_desat.clone()))
        .with_children(|root| {
            root.spawn(lime_text(
                if failed { "FAILED" } else { "RESULTS" },
                &assets,
                40.0,
            ));

            root.spawn((
                Text::new(format!(
                    "{}  |  {}",
                    result.song_name.to_uppercase(),
                    result.difficulty.display_name()
                )),
                TextFont {
                    font: assets.font_ui.clone().into(),
                    font_size: FontSize::Px(20.0),
                    ..default()
                },
                TextColor(TEXT_DIM),
                Node {
                    margin: UiRect::bottom(px(10.0)),
                    ..default()
                },
            ));

            root.spawn((
                Text::new(result.rank),
                TextFont {
                    font: assets.font_score.clone().into(),
                    font_size: FontSize::Px(150.0),
                    ..default()
                },
                TextColor(rank_color),
                TextShadow {
                    offset: Vec2::new(5.0, 5.0),
                    color: Color::BLACK,
                },
            ));

            root.spawn((
                Text::new(format!(
                    "{:.2}%  |  {}/{} NOTES HIT",
                    result.accuracy,
                    result.counts.total().saturating_sub(result.counts.miss),
                    result.counts.total()
                )),
                TextFont {
                    font: assets.font_score.clone().into(),
                    font_size: FontSize::Px(24.0),
                    ..default()
                },
                TextColor(TEXT_PRIMARY),
                Node {
                    margin: UiRect::vertical(px(12.0)),
                    ..default()
                },
            ));

            // Stats panel.
            root.spawn((
                Node {
                    width: px(560.0),
                    flex_direction: FlexDirection::Column,
                    padding: UiRect::all(px(12.0)),
                    row_gap: px(4.0),
                    border: UiRect::all(px(2.0)),
                    ..default()
                },
                BackgroundColor(PANEL),
                BorderColor::all(PANEL_BORDER),
            ))
            .with_children(|panel| {
                let rows = [
                    ("SCORE", result.score.to_string()),
                    ("MAX COMBO", format!("{}x", result.max_combo)),
                    ("SICK!!", result.counts.sick.to_string()),
                    ("GOOD", result.counts.good.to_string()),
                    ("BAD", result.counts.bad.to_string()),
                    ("SHIT", result.counts.shit.to_string()),
                    ("MISS", result.counts.miss.to_string()),
                    ("ARTIST", result.artist.to_uppercase()),
                ];
                for (label, value) in rows {
                    panel
                        .spawn(Node {
                            width: percent(100.0),
                            justify_content: JustifyContent::SpaceBetween,
                            ..default()
                        })
                        .with_children(|row| {
                            row.spawn((
                                Text::new(label),
                                TextFont {
                                    font: assets.font_ui.clone().into(),
                                    font_size: FontSize::Px(18.0),
                                    ..default()
                                },
                                TextColor(TEXT_DIM),
                            ));
                            row.spawn((
                                Text::new(value),
                                TextFont {
                                    font: assets.font_score.clone().into(),
                                    font_size: FontSize::Px(20.0),
                                    ..default()
                                },
                                TextColor(TEXT_PRIMARY),
                            ));
                        });
                }
            });

            root.spawn(Node {
                margin: UiRect::top(px(16.0)),
                column_gap: px(14.0),
                ..default()
            })
            .with_children(|row| {
                row.spawn((
                    menu_button(&assets, "RETRY", 170.0, 48.0, 21.0),
                    observe(|_: On<Activate>, mut next: ResMut<NextState<AppState>>| {
                        next.set(AppState::Play);
                    }),
                ));
                row.spawn((
                    menu_button(&assets, "FREEPLAY", 200.0, 48.0, 21.0),
                    observe(|_: On<Activate>, mut next: ResMut<NextState<AppState>>| {
                        next.set(AppState::Freeplay);
                    }),
                ));
                row.spawn((
                    menu_button(&assets, "MENU", 150.0, 48.0, 21.0),
                    observe(|_: On<Activate>, mut next: ResMut<NextState<AppState>>| {
                        next.set(AppState::MainMenu);
                    }),
                ));
            });

            root.spawn((
                Text::new("ENTER  RETRY   |   ESC  FREEPLAY"),
                TextFont {
                    font: assets.font_ui.clone().into(),
                    font_size: FontSize::Px(17.0),
                    ..default()
                },
                TextColor(TEXT_DIM),
                Node {
                    margin: UiRect::top(px(12.0)),
                    ..default()
                },
            ));

            // Small accent strip.
            root.spawn((
                Node {
                    width: px(140.0),
                    height: px(4.0),
                    margin: UiRect::top(px(10.0)),
                    ..default()
                },
                BackgroundColor(ACCENT),
            ));
        });
}

pub fn results_keys(keys: Res<ButtonInput<KeyCode>>, mut next: ResMut<NextState<AppState>>) {
    if keys.just_pressed(KeyCode::Enter) || keys.just_pressed(KeyCode::Space) {
        next.set(AppState::Play);
    }
    if keys.just_pressed(KeyCode::Escape) {
        next.set(AppState::Freeplay);
    }
}
