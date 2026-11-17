//! Top-left FPS + memory readout, mirroring the original `FunkinDebugDisplay`
//! in its "Simple" mode: white monospace text at (10, 10) showing the current
//! frame rate and process memory (current / peak).

use bevy::prelude::*;
use oxide_core::GameAssets;

/// Refresh at most a few times per second so the text does not flicker.
const UPDATE_INTERVAL: f32 = 0.25;

#[derive(Component)]
pub struct DebugOverlay;

#[derive(Resource, Default)]
pub struct DebugStats {
    frames: u32,
    elapsed: f32,
}

pub fn spawn_debug_overlay(mut commands: Commands, assets: Res<GameAssets>) {
    commands.spawn((
        Node {
            position_type: PositionType::Absolute,
            top: px(10.0),
            left: px(10.0),
            padding: UiRect::axes(px(4.0), px(2.0)),
            ..default()
        },
        BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.35)),
        bevy::ui::GlobalZIndex(1000),
        children![(
            Text::new("FPS: 0\nMEM: --"),
            TextFont {
                font: assets.font_ui.clone().into(),
                font_size: FontSize::Px(16.0),
                ..default()
            },
            TextColor(Color::WHITE),
            DebugOverlay,
        )],
    ));
    commands.init_resource::<DebugStats>();
}

pub fn update_debug_overlay(
    time: Res<Time>,
    mut stats: ResMut<DebugStats>,
    mut texts: Query<&mut Text, With<DebugOverlay>>,
) {
    stats.frames += 1;
    stats.elapsed += time.delta_secs();
    if stats.elapsed < UPDATE_INTERVAL {
        return;
    }
    let fps = (stats.frames as f32 / stats.elapsed).round() as u32;
    stats.frames = 0;
    stats.elapsed = 0.0;

    let (current, peak) = process_memory_bytes();
    let line = format!(
        "FPS: {}\nMEM: {} / {}",
        fps,
        format_bytes(current).to_lowercase(),
        format_bytes(peak).to_lowercase(),
    );
    if let Ok(mut text) = texts.single_mut() {
        text.0 = line;
    }
}

/// Resident set size (current and peak) in bytes, read from the Linux procfs.
/// Falls back to zero elsewhere, where the readout shows `--`.
fn process_memory_bytes() -> (f64, f64) {
    #[cfg(any(target_os = "linux", target_os = "android"))]
    {
        let mut current = 0.0_f64;
        let mut peak = 0.0_f64;
        if let Ok(status) = std::fs::read_to_string("/proc/self/status") {
            for line in status.lines() {
                if let Some(rest) = line.strip_prefix("VmRSS:") {
                    current = parse_kb(rest) * 1024.0;
                } else if let Some(rest) = line.strip_prefix("VmHWM:") {
                    peak = parse_kb(rest) * 1024.0;
                }
            }
        }
        (current, peak)
    }
    #[cfg(not(any(target_os = "linux", target_os = "android")))]
    {
        (0.0, 0.0)
    }
}

#[cfg(any(target_os = "linux", target_os = "android"))]
fn parse_kb(rest: &str) -> f64 {
    rest.split_whitespace()
        .next()
        .and_then(|value| value.parse::<f64>().ok())
        .unwrap_or(0.0)
}

/// Format a byte count like `FlxStringUtil.formatBytes` ("117.74MB").
fn format_bytes(bytes: f64) -> String {
    let units = ["Bytes", "kB", "MB", "GB", "TB", "PB"];
    let mut value = bytes;
    let mut unit = 0;
    while value >= 1024.0 && unit < units.len() - 1 {
        value /= 1024.0;
        unit += 1;
    }
    format!("{value:.2}{}", units[unit])
}
