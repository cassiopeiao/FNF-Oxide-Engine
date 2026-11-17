//! Shared visual language. All colors are `const` so they can be used inside
//! static sprite/UI definitions.

use bevy::prelude::*;

/// Page background behind menus.
pub const BG: Color = Color::srgb(0.035, 0.035, 0.05);
pub const PANEL: Color = Color::srgba(0.05, 0.05, 0.08, 0.86);
pub const PANEL_BORDER: Color = Color::srgba(1.0, 0.35, 0.7, 0.45);
pub const TEXT_PRIMARY: Color = Color::srgb(0.98, 0.97, 0.95);
pub const TEXT_DIM: Color = Color::srgb(0.62, 0.62, 0.7);
pub const ACCENT: Color = Color::srgb(1.0, 0.25, 0.62);
pub const ACCENT_DARK: Color = Color::srgb(0.55, 0.08, 0.33);
pub const SUCCESS: Color = Color::srgb(0.35, 1.0, 0.45);

/// FNF arrow colors: left (purple), down (cyan), up (green), right (red).
pub const LANE_COLORS: [Color; 4] = [
    Color::srgb(0.76, 0.29, 0.60),
    Color::srgb(0.0, 1.0, 1.0),
    Color::srgb(0.07, 0.98, 0.02),
    Color::srgb(0.98, 0.22, 0.25),
];

pub fn lane_color(lane: usize) -> Color {
    LANE_COLORS[lane % LANE_COLORS.len()]
}

/// Darkened version of a color, used for idle receptors.
pub fn dim(color: Color, factor: f32) -> Color {
    let srgba = color.to_srgba();
    Color::srgba(
        srgba.red * factor,
        srgba.green * factor,
        srgba.blue * factor,
        srgba.alpha,
    )
}

/// Judgement colors.
pub fn judgement_color(label: &str) -> Color {
    match label {
        "SICK!!" => Color::srgb(0.35, 0.95, 1.0),
        "GOOD" => Color::srgb(0.4, 1.0, 0.45),
        "BAD" => Color::srgb(1.0, 0.75, 0.2),
        "SHIT" => Color::srgb(1.0, 0.35, 0.2),
        _ => Color::srgb(1.0, 0.25, 0.3),
    }
}

/// Big rank letter color on the results screen.
pub fn rank_color(rank: &str) -> Color {
    match rank {
        "S+" | "S" => Color::srgb(1.0, 0.85, 0.2),
        "A" => Color::srgb(0.35, 1.0, 0.5),
        "B" => Color::srgb(0.4, 0.75, 1.0),
        "C" => Color::srgb(1.0, 0.7, 0.3),
        _ => Color::srgb(1.0, 0.35, 0.35),
    }
}
