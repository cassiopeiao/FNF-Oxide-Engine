//! Shared menu widgets and the single button style pass.

use bevy::picking::hover::Hovered;
use bevy::prelude::*;
use bevy::ui::Pressed;
use bevy::ui_widgets::{ActivateOnPress, Button};
use oxide_core::theme::{ACCENT, BG, PANEL_BORDER, TEXT_PRIMARY};
use oxide_core::{GameAssets, ScreenRoot};

#[derive(Component)]
pub struct MenuButton;

/// A menu button whose label is baked into an image instead of text.
#[derive(Component)]
pub struct ImageMenuButton;

/// Standard glassy menu button. Attach an `observe` bundle to handle clicks.
pub fn menu_button(
    assets: &GameAssets,
    label: impl Into<String>,
    width: f32,
    height: f32,
    font_size: f32,
) -> impl Bundle {
    (
        Button,
        MenuButton,
        ActivateOnPress,
        Hovered::default(),
        Node {
            width: px(width),
            height: px(height),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            border: UiRect::all(px(2.0)),
            border_radius: BorderRadius::all(px(6.0)),
            ..default()
        },
        BackgroundColor(Color::srgba(0.06, 0.06, 0.1, 0.9)),
        BorderColor::all(PANEL_BORDER),
        children![(
            Text::new(label.into()),
            TextFont {
                font: assets.font_ui.clone().into(),
                font_size: FontSize::Px(font_size),
                ..default()
            },
            TextColor(TEXT_PRIMARY),
        )],
    )
}

/// Transparent full-screen column. Use when the background is drawn by 2D
/// sprites (the title screen) rather than by a UI image.
pub fn screen_bundle_clear() -> impl Bundle {
    (
        Node {
            width: percent(100.0),
            height: percent(100.0),
            flex_direction: FlexDirection::Column,
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
            ..default()
        },
        BackgroundColor(Color::NONE),
        ScreenRoot,
    )
}

/// Full-screen background used by every menu.
pub fn screen_bundle(image: Handle<Image>) -> impl Bundle {
    (
        Node {
            width: percent(100.0),
            height: percent(100.0),
            flex_direction: FlexDirection::Column,
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
            ..default()
        },
        ImageNode {
            image,
            color: Color::srgba(0.55, 0.55, 0.68, 1.0),
            image_mode: NodeImageMode::Stretch,
            ..default()
        },
        BackgroundColor(BG),
        ScreenRoot,
    )
}

/// Button carrying the original game's hand-lettered label artwork.
pub fn image_menu_button(image: Handle<Image>, width: f32, height: f32) -> impl Bundle {
    (
        Button,
        MenuButton,
        ImageMenuButton,
        ActivateOnPress,
        Hovered::default(),
        Node {
            width: px(width),
            height: px(height),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            margin: UiRect::vertical(px(4.0)),
            ..default()
        },
        ImageNode::new(image),
        BackgroundColor(Color::NONE),
    )
}

pub fn lime_text(text: impl Into<String>, assets: &GameAssets, size: f32) -> impl Bundle {
    (
        Text::new(text.into()),
        TextFont {
            font: assets.font_ui.clone().into(),
            font_size: FontSize::Px(size),
            ..default()
        },
        TextColor(Color::srgb(0.85, 0.85, 0.9)),
    )
}

#[allow(clippy::type_complexity)]
pub fn button_style(
    mut buttons: Query<
        (
            &Hovered,
            Has<Pressed>,
            &mut BackgroundColor,
            &mut BorderColor,
        ),
        (With<MenuButton>, Without<ImageMenuButton>),
    >,
) {
    for (hovered, pressed, mut background, mut border) in &mut buttons {
        let (fill, outline) = if pressed && hovered.get() {
            (Color::srgba(0.65, 0.12, 0.4, 0.95), ACCENT)
        } else if hovered.get() {
            (Color::srgba(0.3, 0.08, 0.22, 0.95), ACCENT)
        } else {
            (Color::srgba(0.06, 0.06, 0.1, 0.9), PANEL_BORDER)
        };
        background.0 = fill;
        *border = BorderColor::all(outline);
    }
}

/// Tint the hand-lettered buttons on hover/press.
pub fn image_button_style(
    mut buttons: Query<(&Hovered, Has<Pressed>, &mut ImageNode), With<ImageMenuButton>>,
) {
    for (hovered, pressed, mut image) in &mut buttons {
        image.color = if pressed && hovered.get() {
            Color::srgb(0.72, 0.32, 0.52)
        } else if hovered.get() {
            Color::WHITE
        } else {
            Color::srgb(0.82, 0.82, 0.86)
        };
    }
}
