//! Character handles.
//!
//! v-slice hard-codes one BF and one Dad per stage. Oxide supports *N*
//! characters per side so a future versus mode can put several players on the
//! same stage; the SDK therefore addresses characters by name or by slot
//! instead of by a global "boyfriend"/"opponent" singleton.

use crate::events::Side;

/// Stable, opaque identifier for a character on stage.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct CharacterId(pub u32);

impl CharacterId {
    pub const fn raw(self) -> u32 {
        self.0
    }
}

/// A character's static description, as reported by the host.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CharacterInfo {
    pub id: CharacterId,
    pub name: String,
    pub side: Side,
    /// Zero-based slot within its side (player 0, player 1, ...).
    pub slot: u8,
}
