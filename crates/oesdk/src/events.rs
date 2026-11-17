//! Plain-old-data payloads. These are `Copy` on purpose: they cross the wasm
//! boundary by value and must never borrow host memory.

/// Screen-space position, matching the engine's design resolution (1280x720).
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Vec2 {
    pub x: f32,
    pub y: f32,
}

impl Vec2 {
    pub const fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }
}

/// Which side of the stage a character belongs to.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Side {
    Player,
    Opponent,
}

/// Judgement buckets, ordered from best to worst.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Judgement {
    Sick,
    Good,
    Bad,
    Shit,
    Miss,
}

impl Judgement {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Sick => "sick",
            Self::Good => "good",
            Self::Bad => "bad",
            Self::Shit => "shit",
            Self::Miss => "miss",
        }
    }
}

/// A chart note, as the mod sees it.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Note {
    /// 0 = left, 1 = down, 2 = up, 3 = right.
    pub lane: u8,
    /// Absolute chart time in milliseconds.
    pub time_ms: f64,
    /// Hold length in milliseconds; `0.0` for a tap note.
    pub len_ms: f64,
    /// `true` when the note belongs to the player strumline.
    pub player: bool,
}

/// Payload for `on_note_hit`.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Hit {
    pub note: Note,
    pub judgement: Judgement,
    /// Signed timing error in ms: negative = early, positive = late.
    pub delta_ms: f64,
    pub combo: u32,
    pub score: u32,
    /// 0.0..=1.0.
    pub accuracy: f32,
}
