//! Game-wide constants: design resolution, lane geometry and judgement
//! windows. Values intentionally match Friday Night Funkin' closely.

use bevy::prelude::*;

/// The game is authored at 1280x720 and letterboxed/scaled from there.
pub const DESIGN_WIDTH: f32 = 1280.0;
pub const DESIGN_HEIGHT: f32 = 720.0;

/// Four lanes per side.
pub const LANE_COUNT: usize = 4;

/// Receptor and note sprite size, matching the original `Strumline.STRUMLINE_SIZE`.
pub const STRUM_SIZE: f32 = 104.0;
/// Distance between consecutive lane centres, matching `NOTE_SPACING`.
pub const LANE_SPACING: f32 = 112.0;
/// Legacy alias kept for the note-head size (the source uses the same 104px sheet).
pub const NOTE_SIZE: f32 = STRUM_SIZE;

/// Screen-space offsets used to place the two strumlines (original
/// `Constants.STRUMLINE_X_OFFSET` / `STRUMLINE_Y_OFFSET`).
pub const STRUMLINE_X_OFFSET: f32 = 48.0;
pub const STRUMLINE_Y_OFFSET: f32 = 24.0;

/// Per-receptor offset within a strumline, matching the original
/// `Strumline.INITIAL_OFFSET = -0.275 * STRUMLINE_SIZE`.
pub const STRUMLINE_INITIAL_OFFSET: f32 = -0.275 * STRUM_SIZE;

/// Original note / receptor visual scale (from `notestyles/funkin.json`).
pub const NOTE_SCALE: f32 = 0.7;

/// `noteStrumline.png` frame size (the receptor is a 154x157 graphic centred
/// inside this 232x236 frame). The original positions the *frame* and lets the
/// trimmed graphic sit centred inside it, which is why the visible arrow ends
/// up 27px in from the frame's left edge.
pub const STRUM_FRAME_WIDTH: f32 = 232.0;
pub const STRUM_FRAME_HEIGHT: f32 = 236.0;
/// Visible receptor graphic size at `NOTE_SCALE` (154*0.7 x 157*0.7).
pub const STRUM_VISIBLE_WIDTH: f32 = 154.0 * NOTE_SCALE;
pub const STRUM_VISIBLE_HEIGHT: f32 = 157.0 * NOTE_SCALE;
/// Sustain trail width: `NOTE_hold_assets.png` is 416px for 8 notes, so one
/// column is 416/8 * scale = 36.4px (`SustainTrail.graphicWidth`).
pub const HOLD_WIDTH: f32 = 416.0 / 8.0 * NOTE_SCALE;

/// Convert a top-left-origin, y-down screen coordinate (the original game's
/// 1280x720 space) into Bevy world space (centred origin, y up).
pub fn screen_to_world(x: f32, y: f32) -> Vec2 {
    Vec2::new(x - DESIGN_WIDTH * 0.5, DESIGN_HEIGHT * 0.5 - y)
}

/// Screen-space centre of one receptor, exactly like the original:
/// `strumline.x + getXPos(lane) + INITIAL_OFFSET + frameWidth * scale / 2`.
///
/// The opponent strumline keeps the original `x = 48`; the player strumline is
/// its mirror image about the vertical centre line so the two sides are
/// symmetric (as requested) while each keeps the LEFT/DOWN/UP/RIGHT lane order.
fn strum_centre_x(player: bool, lane: usize) -> f32 {
    let half_frame = STRUM_FRAME_WIDTH * NOTE_SCALE * 0.5;
    let opponent = |lane: usize| {
        STRUMLINE_X_OFFSET + lane as f32 * LANE_SPACING + STRUMLINE_INITIAL_OFFSET + half_frame
    };
    if player {
        // Mirror the whole strumline box about the screen centre so both sides
        // are symmetric *without* reversing the LEFT/DOWN/UP/RIGHT lane order
        // (the original keeps both groups in the same order).
        let group_left = STRUMLINE_X_OFFSET + STRUMLINE_INITIAL_OFFSET;
        let group_width = (LANE_COUNT - 1) as f32 * LANE_SPACING + STRUM_FRAME_WIDTH * NOTE_SCALE;
        let mirrored_left = DESIGN_WIDTH - group_left - group_width;
        mirrored_left + lane as f32 * LANE_SPACING + half_frame
    } else {
        opponent(lane)
    }
}

/// World X of the centre of one lane's receptor / notes.
pub fn strum_x(player: bool, lane: usize) -> f32 {
    strum_centre_x(player, lane) - DESIGN_WIDTH * 0.5
}

/// World Y of the receptor centre, depending on scroll direction.
///
/// The original sets `strumline.y = STRUMLINE_Y_OFFSET` (24) as the *top* of
/// the 236px-tall receptor frame; the visible 157px graphic is centred inside
/// it, so the arrow centre lands at `24 + 236*0.7/2 = 106.6`.
pub fn strum_y(downscroll: bool) -> f32 {
    let frame_height = STRUM_FRAME_HEIGHT * NOTE_SCALE;
    let fnf_y = if downscroll {
        DESIGN_HEIGHT - STRUMLINE_Y_OFFSET - frame_height * 0.5
    } else {
        STRUMLINE_Y_OFFSET + frame_height * 0.5
    };
    screen_to_world(0.0, fnf_y).y
}

/// World position of a lane receptor centre.
pub fn lane_pos(player: bool, lane: usize, downscroll: bool) -> Vec2 {
    Vec2::new(strum_x(player, lane), strum_y(downscroll))
}

/// Base scroll speed in world units per millisecond (before the chart's
/// `scrollSpeed` multiplier).
pub const PX_PER_MS: f64 = 0.45;

/// How long a note is visible before it must be hit, in milliseconds.
pub const NOTE_APPROACH_MS: f64 = 1100.0;

/// Countdown length before the instrumental starts.
pub const COUNTDOWN_MS: f64 = 2600.0;

pub const HIT_WINDOW_SICK: f64 = 45.0;
pub const HIT_WINDOW_GOOD: f64 = 90.0;
pub const HIT_WINDOW_BAD: f64 = 135.0;
pub const HIT_WINDOW_SHIT: f64 = 161.0;
pub const MISS_WINDOW: f64 = HIT_WINDOW_SHIT;

/// Extra time after the final note before the results screen appears.
pub const SONG_END_PADDING_MS: f64 = 2200.0;
