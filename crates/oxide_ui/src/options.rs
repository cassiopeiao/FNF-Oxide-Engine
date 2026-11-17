//! Options screen: volumes, downscroll, note speed, offset and ghost tapping.
//! Every change is clamped and written to disk immediately.

use bevy::picking::hover::Hovered;
use bevy::prelude::*;
use bevy::ui_widgets::{Activate, ActivateOnPress, Button, observe};
use oxide_core::theme::{PANEL, PANEL_BORDER, TEXT_DIM, TEXT_PRIMARY};
use oxide_core::{AppState, GameAssets, SaveStore, Settings};

use crate::widgets::{lime_text, menu_button, screen_bundle};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OptionField {
    Master,
    Music,
    Sfx,
    Speed,
    Offset,
    Downscroll,
    Ghost,
    Msaa,
}

#[derive(Component)]
pub struct OptionValue(pub OptionField);

#[derive(Debug, Clone, Copy)]
pub enum OptionAction {
    Master(i32),
    Music(i32),
    Sfx(i32),
    Speed(i32),
    Offset(i32),
    ToggleDownscroll,
    ToggleGhost,
    Msaa(i32),
    Reset,
}

fn apply(action: OptionAction, settings: &mut Settings) {
    match action {
        OptionAction::Master(delta) => settings.master_volume += delta as f32 * 0.1,
        OptionAction::Music(delta) => settings.music_volume += delta as f32 * 0.1,
        OptionAction::Sfx(delta) => settings.sfx_volume += delta as f32 * 0.1,
        OptionAction::Speed(delta) => settings.note_speed += delta as f32 * 0.1,
        OptionAction::Offset(delta) => settings.offset_ms += delta as f32 * 10.0,
        OptionAction::ToggleDownscroll => settings.downscroll = !settings.downscroll,
        OptionAction::ToggleGhost => settings.ghost_tapping = !settings.ghost_tapping,
        OptionAction::Msaa(delta) => settings.cycle_msaa(delta),
        OptionAction::Reset => *settings = Settings::default(),
    }
    settings.clamp();
}

fn value_text(field: OptionField, settings: &Settings) -> String {
    match field {
        OptionField::Master => format!("{:.0}%", settings.master_volume * 100.0),
        OptionField::Music => format!("{:.0}%", settings.music_volume * 100.0),
        OptionField::Sfx => format!("{:.0}%", settings.sfx_volume * 100.0),
        OptionField::Speed => format!("{:.1}x", settings.note_speed),
        OptionField::Offset => format!("{:+.0} ms", settings.offset_ms),
        OptionField::Downscroll => if settings.downscroll { "ON" } else { "OFF" }.to_owned(),
        OptionField::Ghost => if settings.ghost_tapping { "ON" } else { "OFF" }.to_owned(),
        OptionField::Msaa => settings.msaa_label().to_owned(),
    }
}

fn step_button(assets: &GameAssets, label: &'static str, action: OptionAction) -> impl Bundle {
    (
        Button,
        ActivateOnPress,
        Hovered::default(),
        Node {
            width: px(44.0),
            height: px(38.0),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            border: UiRect::all(px(2.0)),
            ..default()
        },
        BackgroundColor(PANEL),
        BorderColor::all(PANEL_BORDER),
        children![(
            Text::new(label),
            TextFont {
                font: assets.font_ui.clone().into(),
                font_size: FontSize::Px(22.0),
                ..default()
            },
            TextColor(TEXT_PRIMARY),
        )],
        observe(
            move |_: On<Activate>, mut settings: ResMut<Settings>, save: Res<SaveStore>| {
                apply(action, &mut settings);
                save.save(&settings);
            },
        ),
    )
}

fn toggle_button(assets: &GameAssets, action: OptionAction) -> impl Bundle {
    (
        Button,
        ActivateOnPress,
        Hovered::default(),
        Node {
            width: px(120.0),
            height: px(38.0),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            border: UiRect::all(px(2.0)),
            ..default()
        },
        BackgroundColor(PANEL),
        BorderColor::all(PANEL_BORDER),
        children![(
            Text::new("TOGGLE"),
            TextFont {
                font: assets.font_ui.clone().into(),
                font_size: FontSize::Px(15.0),
                ..default()
            },
            TextColor(TEXT_PRIMARY),
        )],
        observe(
            move |_: On<Activate>, mut settings: ResMut<Settings>, save: Res<SaveStore>| {
                apply(action, &mut settings);
                save.save(&settings);
            },
        ),
    )
}

pub fn spawn_options(mut commands: Commands, assets: Res<GameAssets>) {
    commands
        .spawn(screen_bundle(assets.menu_bg.clone()))
        .with_children(|root| {
            root.spawn(lime_text("OPTIONS", &assets, 50.0));
            root.spawn((
                Text::new("CHANGES SAVE AUTOMATICALLY"),
                TextFont {
                    font: assets.font_ui.clone().into(),
                    font_size: FontSize::Px(16.0),
                    ..default()
                },
                TextColor(TEXT_DIM),
                Node {
                    margin: UiRect::bottom(px(14.0)),
                    ..default()
                },
            ));

            root.spawn((
                Node {
                    width: px(900.0),
                    flex_direction: FlexDirection::Column,
                    align_items: AlignItems::Center,
                    padding: UiRect::all(px(18.0)),
                    row_gap: px(2.0),
                    border: UiRect::all(px(2.0)),
                    ..default()
                },
                BackgroundColor(Color::srgba(0.03, 0.03, 0.05, 0.82)),
                BorderColor::all(PANEL_BORDER),
            ))
            .with_children(|panel| {
                // Sliders.
                let sliders: [(&str, OptionField, OptionAction, OptionAction); 6] = [
                    (
                        "MASTER VOLUME",
                        OptionField::Master,
                        OptionAction::Master(-1),
                        OptionAction::Master(1),
                    ),
                    (
                        "MUSIC VOLUME",
                        OptionField::Music,
                        OptionAction::Music(-1),
                        OptionAction::Music(1),
                    ),
                    (
                        "SFX VOLUME",
                        OptionField::Sfx,
                        OptionAction::Sfx(-1),
                        OptionAction::Sfx(1),
                    ),
                    (
                        "NOTE SPEED",
                        OptionField::Speed,
                        OptionAction::Speed(-1),
                        OptionAction::Speed(1),
                    ),
                    (
                        "INPUT OFFSET",
                        OptionField::Offset,
                        OptionAction::Offset(-1),
                        OptionAction::Offset(1),
                    ),
                    (
                        "ANTI-ALIASING",
                        OptionField::Msaa,
                        OptionAction::Msaa(-1),
                        OptionAction::Msaa(1),
                    ),
                ];

                for (label, field, decrement, increment) in sliders {
                    panel
                        .spawn(Node {
                            width: px(760.0),
                            height: px(48.0),
                            align_items: AlignItems::Center,
                            justify_content: JustifyContent::SpaceBetween,
                            ..default()
                        })
                        .with_children(|row| {
                            row.spawn((
                                Text::new(label),
                                TextFont {
                                    font: assets.font_ui.clone().into(),
                                    font_size: FontSize::Px(22.0),
                                    ..default()
                                },
                                TextColor(TEXT_DIM),
                            ));
                            row.spawn(Node {
                                align_items: AlignItems::Center,
                                column_gap: px(12.0),
                                ..default()
                            })
                            .with_children(|controls| {
                                controls.spawn(step_button(&assets, "-", decrement));
                                controls
                                    .spawn(Node {
                                        width: px(150.0),
                                        justify_content: JustifyContent::Center,
                                        ..default()
                                    })
                                    .with_children(|value| {
                                        value.spawn((
                                            Text::new(value_text(field, &Settings::default())),
                                            TextFont {
                                                font: assets.font_score.clone().into(),
                                                font_size: FontSize::Px(22.0),
                                                ..default()
                                            },
                                            TextColor(TEXT_PRIMARY),
                                            OptionValue(field),
                                        ));
                                    });
                                controls.spawn(step_button(&assets, "+", increment));
                            });
                        });
                }

                // Toggles.
                for (label, field, action) in [
                    (
                        "DOWNSCROLL",
                        OptionField::Downscroll,
                        OptionAction::ToggleDownscroll,
                    ),
                    (
                        "GHOST TAPPING",
                        OptionField::Ghost,
                        OptionAction::ToggleGhost,
                    ),
                ] {
                    panel
                        .spawn(Node {
                            width: px(760.0),
                            height: px(48.0),
                            align_items: AlignItems::Center,
                            justify_content: JustifyContent::SpaceBetween,
                            ..default()
                        })
                        .with_children(|row| {
                            row.spawn((
                                Text::new(label),
                                TextFont {
                                    font: assets.font_ui.clone().into(),
                                    font_size: FontSize::Px(22.0),
                                    ..default()
                                },
                                TextColor(TEXT_DIM),
                            ));
                            row.spawn(Node {
                                align_items: AlignItems::Center,
                                column_gap: px(12.0),
                                ..default()
                            })
                            .with_children(|controls| {
                                controls
                                    .spawn(Node {
                                        width: px(150.0),
                                        justify_content: JustifyContent::Center,
                                        ..default()
                                    })
                                    .with_children(|value| {
                                        value.spawn((
                                            Text::new(value_text(field, &Settings::default())),
                                            TextFont {
                                                font: assets.font_score.clone().into(),
                                                font_size: FontSize::Px(22.0),
                                                ..default()
                                            },
                                            TextColor(TEXT_PRIMARY),
                                            OptionValue(field),
                                        ));
                                    });
                                controls.spawn(toggle_button(&assets, action));
                            });
                        });
                }
            });

            root.spawn(Node {
                margin: UiRect::top(px(20.0)),
                column_gap: px(14.0),
                ..default()
            })
            .with_children(|row| {
                row.spawn((
                    menu_button(&assets, "RESET DEFAULTS", 240.0, 48.0, 20.0),
                    observe(
                        |_: On<Activate>, mut settings: ResMut<Settings>, save: Res<SaveStore>| {
                            apply(OptionAction::Reset, &mut settings);
                            save.save(&settings);
                        },
                    ),
                ));
                row.spawn((
                    menu_button(&assets, "BACK", 180.0, 48.0, 20.0),
                    observe(|_: On<Activate>, mut next: ResMut<NextState<AppState>>| {
                        next.set(AppState::MainMenu);
                    }),
                ));
            });

            root.spawn((
                Text::new("ESC  BACK"),
                TextFont {
                    font: assets.font_ui.clone().into(),
                    font_size: FontSize::Px(17.0),
                    ..default()
                },
                TextColor(TEXT_DIM),
                Node {
                    margin: UiRect::top(px(18.0)),
                    ..default()
                },
            ));
        });
}

pub fn refresh_values(settings: Res<Settings>, mut values: Query<(&OptionValue, &mut Text)>) {
    if !settings.is_changed() {
        return;
    }
    for (value, mut text) in &mut values {
        text.0 = value_text(value.0, &settings);
    }
}

pub fn options_keys(keys: Res<ButtonInput<KeyCode>>, mut next: ResMut<NextState<AppState>>) {
    if keys.just_pressed(KeyCode::Escape) {
        next.set(AppState::MainMenu);
    }
}
